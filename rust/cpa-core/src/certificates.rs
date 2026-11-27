//! Certificates of a CPA without user interface: the details of every X509Certificate, the expiry dates, which
//! elements refer to a certificate, the PEM export of a KeyInfo and a new KeyInfo from a PEM chain.
//! Port of `helper_classes/certificateExpiry.py`, `certificateUsage.py` and the logic of
//! `app_classes/certificates_tab.py` and `general_tab.set_end_date_from_certificates`.
//!
//! Names are written like Python cryptography's `Name.rfc4514_string()`: RDNs in reverse order, the short names
//! CN, L, ST, O, OU, C, STREET, DC and UID (others as dotted OID), RFC 4514 escaping and `+` between the attributes
//! of a multi-valued RDN.

use std::collections::HashMap;
use std::fmt;

use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use base64::Engine;
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use x509_parser::prelude::{FromDer, X509Certificate, X509Name};
use x509_parser::public_key::PublicKey;

use crate::cpa::{set_general, strip_redundant_namespaces, GeneralField};
use crate::dates::format_cpa_date;
use crate::xml::{Document, NodeId};
use crate::{CPA_NAMESPACE, DS_NAMESPACE};

/// The position of a certificate in its chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CertificateType {
    Leaf,
    Intermediate,
    Root,
}

impl fmt::Display for CertificateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CertificateType::Leaf => "leaf",
            CertificateType::Intermediate => "intermediate",
            CertificateType::Root => "root",
        })
    }
}

/// The details of one certificate of a KeyInfo, as the Certificates tab shows them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateDetails {
    pub kind: CertificateType,
    pub common_name: String,
    pub subject: String,
    pub issuer: String,
    /// Serial number in uppercase hexadecimal, without leading zeros.
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub expired: bool,
}

/// The expiry date of one certificate in the CPA.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateExpiry {
    pub not_after: DateTime<Utc>,
    pub kind: CertificateType,
    pub common_name: String,
    pub subject: String,
    /// The certId of the Certificate element; None when it has none.
    pub cert_id: Option<String>,
}

impl CertificateExpiry {
    /// The certId as the Python version printed it: `None` when there is none.
    pub fn cert_id_text(&self) -> &str {
        self.cert_id.as_deref().unwrap_or("None")
    }
}

/// A Certificate element of a party with its KeyInfo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateEntry {
    pub cert_id: String,
    pub certificate: NodeId,
    pub key_info: Option<NodeId>,
}

// ----------------------------------------------------------------------------------------------------------------
// Reading one certificate
// ----------------------------------------------------------------------------------------------------------------

/// A value of a name attribute: text, or bytes for a BIT STRING (written as `#hex`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum NameValue {
    Text(String),
    Bytes(Vec<u8>),
}

/// A name as a list of RDNs, each a list of (dotted OID, value).
type Name = Vec<Vec<(String, NameValue)>>;

/// What the CPA Editor needs of a certificate.
struct ParsedCertificate {
    kind: CertificateType,
    common_name: String,
    subject: String,
    issuer: String,
    serial: BigInt,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    der: Vec<u8>,
    /// Modulus and exponent (minimal big-endian bytes) of an RSA key, otherwise the error of the Python version.
    rsa: Result<(Vec<u8>, Vec<u8>), String>,
}

impl ParsedCertificate {
    fn details(&self, now: DateTime<Utc>) -> CertificateDetails {
        CertificateDetails {
            kind: self.kind,
            common_name: self.common_name.clone(),
            subject: self.subject.clone(),
            issuer: self.issuer.clone(),
            serial_number: self.serial.to_str_radix(16).to_uppercase(),
            not_before: self.not_before,
            not_after: self.not_after,
            expired: self.not_after < now,
        }
    }
}

const OID_COMMON_NAME: &str = "2.5.4.3";

/// Short names of cryptography's `_NAMEOID_TO_NAME`.
const SHORT_NAMES: &[(&str, &str)] = &[
    ("2.5.4.3", "CN"),
    ("2.5.4.7", "L"),
    ("2.5.4.8", "ST"),
    ("2.5.4.10", "O"),
    ("2.5.4.11", "OU"),
    ("2.5.4.6", "C"),
    ("2.5.4.9", "STREET"),
    ("0.9.2342.19200300.100.1.25", "DC"),
    ("0.9.2342.19200300.100.1.1", "UID"),
];

/// Reads a DER certificate like cryptography's `load_der_x509_certificate`.
fn parse_der_certificate(der: &[u8]) -> Result<ParsedCertificate, String> {
    let (rest, certificate) = X509Certificate::from_der(der).map_err(|e| format!("error parsing asn1 value: {e}"))?;
    if !rest.is_empty() {
        return Err("error parsing asn1 value: ParseError { kind: ExtraData }".to_string());
    }
    let subject = decode_name(certificate.subject())?;
    let issuer = decode_name(certificate.issuer())?;
    let is_ca = match certificate.basic_constraints() {
        Ok(Some(extension)) => extension.value.ca,
        Ok(None) => false,
        Err(e) => return Err(e.to_string()),
    };
    let kind = if !is_ca {
        CertificateType::Leaf
    } else if names_equal(&subject, &issuer) {
        CertificateType::Root
    } else {
        CertificateType::Intermediate
    };
    let common_name = subject
        .iter()
        .flatten()
        .find(|(oid, _)| oid == OID_COMMON_NAME)
        .map(|(_, value)| match value {
            NameValue::Text(text) => text.clone(),
            NameValue::Bytes(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        })
        .unwrap_or_else(|| rfc4514_string(&subject));
    let validity = certificate.validity();
    let timestamp = |seconds: i64| {
        DateTime::<Utc>::from_timestamp(seconds, 0).ok_or_else(|| "certificate date out of range".to_string())
    };
    let rsa = match certificate.public_key().parsed() {
        Ok(PublicKey::RSA(key)) => Ok((minimal_bytes(key.modulus), minimal_bytes(key.exponent))),
        Ok(PublicKey::EC(_)) => Err("'EllipticCurvePublicNumbers' object has no attribute 'n'".to_string()),
        Ok(PublicKey::DSA(_)) => Err("'DSAPublicNumbers' object has no attribute 'n'".to_string()),
        Ok(_) => Err("The public key of the certificate is not an RSA key".to_string()),
        Err(e) => Err(e.to_string()),
    };
    Ok(ParsedCertificate {
        kind,
        common_name,
        subject: rfc4514_string(&subject),
        issuer: rfc4514_string(&issuer),
        serial: BigInt::from_signed_bytes_be(certificate.raw_serial()),
        not_before: timestamp(validity.not_before.timestamp())?,
        not_after: timestamp(validity.not_after.timestamp())?,
        der: der.to_vec(),
        rsa,
    })
}

fn minimal_bytes(bytes: &[u8]) -> Vec<u8> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

/// Decodes the attribute values like cryptography: BMPString as UTF-16, UniversalString as UTF-32, BIT STRING as
/// bytes, every other type as UTF-8.
fn decode_name(name: &X509Name) -> Result<Name, String> {
    let mut rdns = Vec::new();
    for rdn in name.iter() {
        let mut attributes = Vec::new();
        for attribute in rdn.iter() {
            let oid = attribute.attr_type().to_id_string();
            let value = attribute.attr_value();
            let data = value.data;
            let decoded = match value.header.tag().0 {
                3 => NameValue::Bytes(data.to_vec()),
                30 => {
                    if data.len() % 2 != 0 {
                        return Err("Parsing name attribute failed".to_string());
                    }
                    let units: Vec<u16> = data.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
                    NameValue::Text(String::from_utf16(&units).map_err(|_| "Parsing name attribute failed")?)
                }
                28 => {
                    if data.len() % 4 != 0 {
                        return Err("Parsing name attribute failed".to_string());
                    }
                    let text: Option<String> = data
                        .chunks(4)
                        .map(|c| char::from_u32(u32::from_be_bytes([c[0], c[1], c[2], c[3]])))
                        .collect();
                    NameValue::Text(text.ok_or("Parsing name attribute failed")?)
                }
                _ => NameValue::Text(
                    std::str::from_utf8(data).map_err(|_| "Parsing name attribute failed")?.to_string(),
                ),
            };
            attributes.push((oid, decoded));
        }
        rdns.push(attributes);
    }
    Ok(rdns)
}

/// cryptography's `Name.__eq__`: the same RDNs in the same order, each RDN compared as a set.
fn names_equal(a: &Name, b: &Name) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| x.iter().all(|v| y.contains(v)) && y.iter().all(|v| x.contains(v)))
}

/// cryptography's `Name.rfc4514_string()`.
fn rfc4514_string(name: &Name) -> String {
    name.iter()
        .rev()
        .map(|rdn| {
            rdn.iter()
                .map(|(oid, value)| {
                    let short = SHORT_NAMES.iter().find(|(o, _)| o == oid).map(|(_, s)| *s).unwrap_or(oid.as_str());
                    format!("{short}={}", escape_dn_value(value))
                })
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// cryptography's `_escape_dn_value`.
fn escape_dn_value(value: &NameValue) -> String {
    let text = match value {
        NameValue::Bytes(bytes) if bytes.is_empty() => return String::new(),
        NameValue::Bytes(bytes) => return format!("#{}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()),
        NameValue::Text(text) => text,
    };
    if text.is_empty() {
        return String::new();
    }
    let mut value = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('+', "\\+")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('\0', "\\00");
    if value.starts_with('#') || (value.starts_with(' ') && value.chars().count() > 1) {
        value.insert(0, '\\');
    }
    if value.ends_with(' ') {
        value.pop();
        value.push_str("\\ ");
    }
    value
}

/// Python's `base64.b64decode` without validation: characters outside the base64 alphabet are skipped.
fn decode_base64_lenient(text: &str) -> Result<Vec<u8>, String> {
    let filtered: String =
        text.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=')).collect();
    let engine = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new()
            .with_decode_allow_trailing_bits(true)
            .with_decode_padding_mode(DecodePaddingMode::RequireCanonical),
    );
    engine.decode(filtered.as_bytes()).map_err(|e| format!("Invalid base64: {e}"))
}

fn read_x509_certificate(doc: &Document, x509_certificate: NodeId) -> Result<ParsedCertificate, String> {
    let der = decode_base64_lenient(&doc.text(x509_certificate).unwrap_or_default())?;
    parse_der_certificate(&der)
}

fn x509_certificates(doc: &Document, element: NodeId) -> Vec<NodeId> {
    doc.iter_named(element, Some(DS_NAMESPACE), "X509Certificate").collect()
}

// ----------------------------------------------------------------------------------------------------------------
// Certificates of a KeyInfo and of the whole CPA
// ----------------------------------------------------------------------------------------------------------------

/// The details of every X509Certificate of a KeyInfo, in the order of the KeyInfo. A certificate that cannot be
/// read is left out and reported as `Could not read a certificate: ...`.
pub fn key_info_certificates(
    doc: &Document,
    key_info: Option<NodeId>,
    now: DateTime<Utc>,
) -> (Vec<CertificateDetails>, Vec<String>) {
    let mut certificates = Vec::new();
    let mut errors = Vec::new();
    let Some(key_info) = key_info else { return (certificates, errors) };
    for x509_certificate in x509_certificates(doc, key_info) {
        match read_x509_certificate(doc, x509_certificate) {
            Ok(certificate) => certificates.push(certificate.details(now)),
            Err(e) => errors.push(format!("Could not read a certificate: {e}")),
        }
    }
    (certificates, errors)
}

/// The expiry date of every X509Certificate (leaf, intermediate and root) in the CPA, sorted on the expiry date so
/// the certificate that expires first comes first (stable for equal dates). A certificate that cannot be read is
/// reported as `Could not read a certificate of certId '...': ...`.
pub fn certificate_expiries(doc: &Document) -> (Vec<CertificateExpiry>, Vec<String>) {
    let mut expiries = Vec::new();
    let mut errors = Vec::new();
    let Some(root) = doc.root() else { return (expiries, errors) };
    for certificate_element in doc.iter_named(root, Some(CPA_NAMESPACE), "Certificate") {
        let cert_id = doc.attribute(certificate_element, Some(CPA_NAMESPACE), "certId").map(str::to_string);
        for x509_certificate in x509_certificates(doc, certificate_element) {
            match read_x509_certificate(doc, x509_certificate) {
                Ok(certificate) => expiries.push(CertificateExpiry {
                    not_after: certificate.not_after,
                    kind: certificate.kind,
                    common_name: certificate.common_name,
                    subject: certificate.subject,
                    cert_id: cert_id.clone(),
                }),
                Err(e) => errors.push(format!(
                    "Could not read a certificate of certId '{}': {e}",
                    cert_id.as_deref().unwrap_or("None")
                )),
            }
        }
    }
    expiries.sort_by_key(|expiry| expiry.not_after);
    (expiries, errors)
}

/// The certificate that expires first, None without certificates.
pub fn first_expiring_certificate(doc: &Document) -> Option<CertificateExpiry> {
    certificate_expiries(doc).0.into_iter().next()
}

/// Attributes that identify the element a certificate reference belongs to.
const CONTEXT_ID_ATTRIBUTES: [&str; 4] = ["transportId", "docExchangeId", "securityId", "channelId"];

/// Where a certificate reference is, for example `ClientCertificateRef in Transport 'PartnerA_Transport'`: the
/// nearest ancestor with an id, otherwise the parent.
pub fn describe_reference(doc: &Document, reference: NodeId) -> String {
    let name = doc.local_name(reference).to_string();
    let parent = doc.parent_element(reference);
    let mut ancestor = parent;
    while let Some(element) = ancestor {
        for attribute in CONTEXT_ID_ATTRIBUTES {
            if let Some(context_id) = doc.attribute(element, Some(CPA_NAMESPACE), attribute) {
                if !context_id.is_empty() {
                    return format!("{name} in {} '{context_id}'", doc.local_name(element));
                }
            }
        }
        ancestor = doc.parent_element(element);
    }
    match parent {
        Some(parent) => format!("{name} in {}", doc.local_name(parent)),
        None => name,
    }
}

/// Per certId the elements that refer to that certificate (every element with a certId other than Certificate),
/// described with [`describe_reference`] in the order of the CPA. A certId without references is not in the map.
pub fn certificate_usages(doc: &Document) -> HashMap<String, Vec<String>> {
    let mut usages: HashMap<String, Vec<String>> = HashMap::new();
    let Some(root) = doc.root() else { return usages };
    for element in doc.descendants(root) {
        if !doc.is_element(element) || doc.is_named(element, Some(CPA_NAMESPACE), "Certificate") {
            continue;
        }
        if let Some(cert_id) = doc.attribute(element, Some(CPA_NAMESPACE), "certId") {
            if !cert_id.is_empty() {
                usages.entry(cert_id.to_string()).or_default().push(describe_reference(doc, element));
            }
        }
    }
    usages
}

/// The Certificate elements of a party with their KeyInfo, like `generate_partner_keyinfo_certid_list`:
/// a Certificate without certId is reported and skipped, one with an empty certId is skipped silently, one
/// without KeyInfo is kept with `key_info: None` and reported. A certId that occurs twice keeps the position of
/// the first and the elements of the last (Python dict).
pub fn certificate_entries(doc: &Document, party_info: NodeId) -> (Vec<CertificateEntry>, Vec<String>) {
    let mut entries: Vec<CertificateEntry> = Vec::new();
    let mut errors = Vec::new();
    for certificate in doc.find_all(party_info, Some(CPA_NAMESPACE), "Certificate") {
        let Some(cert_id) = doc.attribute(certificate, Some(CPA_NAMESPACE), "certId") else {
            errors.push(
                "Certificate ID attribute not found for certificate element in CPA please check the CPA".to_string(),
            );
            continue;
        };
        if cert_id.is_empty() {
            continue;
        }
        let key_info = doc.find(certificate, Some(DS_NAMESPACE), "KeyInfo");
        if key_info.is_none() {
            errors.push(format!("KeyInfo not found for certificate ID: {cert_id}"));
        }
        let entry = CertificateEntry { cert_id: cert_id.to_string(), certificate, key_info };
        match entries.iter_mut().find(|existing| existing.cert_id == cert_id) {
            Some(existing) => *existing = entry,
            None => entries.push(entry),
        }
    }
    (entries, errors)
}

// ----------------------------------------------------------------------------------------------------------------
// PEM
// ----------------------------------------------------------------------------------------------------------------

/// The certificates of a KeyInfo in PEM format, in the order of the KeyInfo (leaf first); empty without KeyInfo
/// or certificates. Error when a certificate cannot be read (the tab logs `Error while downloading certificate:`).
pub fn key_info_to_pem(doc: &Document, key_info: Option<NodeId>) -> Result<String, String> {
    let mut pem = String::new();
    let Some(key_info) = key_info else { return Ok(pem) };
    for x509_certificate in x509_certificates(doc, key_info) {
        let certificate = read_x509_certificate(doc, x509_certificate)?;
        pem.push_str(&der_to_pem(&certificate.der));
    }
    Ok(pem)
}

/// A certificate in PEM, as cryptography writes it: lines of 64 characters.
fn der_to_pem(der: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for line in encoded.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

const PEM_FAQ: &str = "Unable to load PEM file. See https://cryptography.io/en/latest/faq/#why-can-t-i-import-my-pem-file \
                       for more details.";

/// The DER of the first certificate in a PEM text, like cryptography's `load_pem_x509_certificate`.
fn first_pem_certificate(text: &str) -> Result<Vec<u8>, String> {
    let mut found_block = false;
    let mut position = 0;
    while let Some(start) = text[position..].find("-----BEGIN ") {
        let label_start = position + start + "-----BEGIN ".len();
        let Some(label_length) = text[label_start..].find("-----") else { break };
        let label = &text[label_start..label_start + label_length];
        if label.contains('\n') {
            position = label_start;
            continue;
        }
        let body_start = label_start + label_length + "-----".len();
        let end_marker = format!("-----END {label}-----");
        let Some(body_length) = text[body_start..].find(&end_marker) else { break };
        found_block = true;
        position = body_start + body_length + end_marker.len();
        if label != "CERTIFICATE" && label != "X509 CERTIFICATE" {
            continue;
        }
        let body: String = text[body_start..body_start + body_length]
            .lines()
            .filter(|line| !line.contains(':'))
            .flat_map(|line| line.chars())
            .filter(|c| !c.is_whitespace())
            .collect();
        return base64::engine::general_purpose::STANDARD
            .decode(body.as_bytes())
            .map_err(|e| format!("{PEM_FAQ} InvalidData({e})"));
    }
    if found_block {
        Err("Valid PEM but no BEGIN CERTIFICATE/END CERTIFICATE delimiters. Are you sure this is a certificate?"
            .to_string())
    } else {
        Err(format!("{PEM_FAQ} MalformedFraming"))
    }
}

/// Builds a new KeyInfo (not yet in the document) from a PEM file with a certificate chain, leaf first, like the
/// "Upload Certificate" menu: a KeyValue with the RSA key of the leaf, then per certificate an X509Data with
/// X509IssuerSerial (issuer name and decimal serial number), X509SubjectName and X509Certificate. The KeyInfo
/// declares the prefix `ds`. Error (the tab logs `Error while uploading certificate:`) when a certificate cannot be
/// read, the leaf has no RSA key, or the text holds no certificate at all.
pub fn key_info_from_pem(doc: &mut Document, pem: &str) -> Result<NodeId, String> {
    let chain = pem.trim();
    let parts: Vec<String> = chain
        .split("-----END CERTIFICATE-----")
        .filter(|part| !part.is_empty())
        .map(|part| format!("{part}-----END CERTIFICATE-----"))
        .collect();
    if parts.is_empty() {
        return Err("No certificate found in the file".to_string());
    }
    let mut certificates = Vec::new();
    for part in &parts {
        certificates.push(parse_der_certificate(&first_pem_certificate(part)?)?);
    }
    let (modulus, exponent) = certificates[0].rsa.clone()?;

    let key_info = doc.create_element(Some(DS_NAMESPACE), "KeyInfo");
    doc.declare_namespace(key_info, Some("ds"), DS_NAMESPACE);
    let standard = base64::engine::general_purpose::STANDARD;
    let key_value = doc.append_element(key_info, Some(DS_NAMESPACE), "KeyValue");
    let rsa_key_value = doc.append_element(key_value, Some(DS_NAMESPACE), "RSAKeyValue");
    for (name, bytes) in [("Modulus", &modulus), ("Exponent", &exponent)] {
        let element = doc.append_element(rsa_key_value, Some(DS_NAMESPACE), name);
        doc.set_text(element, Some(&standard.encode(bytes)));
    }
    for certificate in &certificates {
        let x509_data = doc.append_element(key_info, Some(DS_NAMESPACE), "X509Data");
        let issuer_serial = doc.append_element(x509_data, Some(DS_NAMESPACE), "X509IssuerSerial");
        let issuer_name = doc.append_element(issuer_serial, Some(DS_NAMESPACE), "X509IssuerName");
        doc.set_text(issuer_name, Some(&certificate.issuer));
        let serial_number = doc.append_element(issuer_serial, Some(DS_NAMESPACE), "X509SerialNumber");
        doc.set_text(serial_number, Some(&certificate.serial.to_string()));
        let subject_name = doc.append_element(x509_data, Some(DS_NAMESPACE), "X509SubjectName");
        doc.set_text(subject_name, Some(&certificate.subject));
        let x509_certificate = doc.append_element(x509_data, Some(DS_NAMESPACE), "X509Certificate");
        doc.set_text(x509_certificate, Some(&standard.encode(&certificate.der)));
    }
    Ok(key_info)
}

/// Puts a new KeyInfo in a Certificate element: the old KeyInfo (if any) is removed with its tail and the new one
/// is added at the end of the Certificate, like `replace_xml_object`. A `ds` declaration of the new KeyInfo that
/// is already in scope is dropped, as lxml does.
///
/// The Python version left an old KeyInfo without children in place (lxml elements without children are false);
/// this always replaces it.
pub fn replace_key_info(doc: &mut Document, certificate: NodeId, old: Option<NodeId>, new: NodeId) {
    if let Some(old) = old {
        if doc.parent(old) == Some(certificate) {
            doc.remove_with_tail(old);
        }
    }
    doc.append_child(certificate, new);
    strip_redundant_namespaces(doc, new);
}

/// The "Set from certificates" button: sets the end date of the CPA to the expiry date of the certificate that
/// expires first (leaf, intermediate and root all count). Returns the date, that certificate and the debug lines
/// that list every certificate in order of expiry.
///
/// Error `No certificates found in the CPA, end date not changed` without readable certificates.
pub fn end_date_from_certificates(doc: &mut Document) -> Result<(String, CertificateExpiry, Vec<String>), String> {
    let (expiries, _) = certificate_expiries(doc);
    let Some(first_expiring) = expiries.first().cloned() else {
        return Err("No certificates found in the CPA, end date not changed".to_string());
    };
    let debug_lines = expiries
        .iter()
        .enumerate()
        .map(|(index, expiry)| {
            format!(
                "Certificate {}/{}: {} expires {} ({}, certId {})",
                index + 1,
                expiries.len(),
                expiry.common_name,
                format_cpa_date(expiry.not_after),
                expiry.kind,
                expiry.cert_id_text()
            )
        })
        .collect();
    let end_date = format_cpa_date(first_expiring.not_after);
    set_general(doc, GeneralField::EndDate, &end_date)?;
    Ok((end_date, first_expiring, debug_lines))
}

/// The log line of the "Set from certificates" button after the end date was set.
pub fn end_date_message(end_date: &str, expiry: &CertificateExpiry) -> String {
    format!(
        "End date set to {end_date}: expiry of {} certificate '{}' (certId {})",
        expiry.kind,
        expiry.subject,
        expiry.cert_id_text()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> NameValue {
        NameValue::Text(value.to_string())
    }

    #[test]
    fn escapes_like_cryptography() {
        assert_eq!(escape_dn_value(&text("a,b+c\"d\\e;f<g>h")), "a\\,b\\+c\\\"d\\\\e\\;f\\<g\\>h");
        assert_eq!(escape_dn_value(&text("#x")), "\\#x");
        assert_eq!(escape_dn_value(&text(" x ")), "\\ x\\ ");
        assert_eq!(escape_dn_value(&text(" ")), "\\ ");
        assert_eq!(escape_dn_value(&text("")), "");
        assert_eq!(escape_dn_value(&NameValue::Bytes(vec![0, 0xab])), "#00ab");
    }

    #[test]
    fn rfc4514_reverses_rdns_and_joins_multi_valued() {
        let name: Name = vec![
            vec![("2.5.4.6".to_string(), text("NL"))],
            vec![("2.5.4.10".to_string(), text("Org")), ("1.2.3.4".to_string(), text("x"))],
            vec![(OID_COMMON_NAME.to_string(), text("Leaf"))],
        ];
        assert_eq!(rfc4514_string(&name), "CN=Leaf,O=Org+1.2.3.4=x,C=NL");
    }

    #[test]
    fn lenient_base64_skips_whitespace() {
        assert_eq!(decode_base64_lenient("bm90IGEg\n  Y2VydA==").unwrap(), b"not a cert");
        assert!(decode_base64_lenient("bm90IGEgY2VydA").is_err());
    }
}
