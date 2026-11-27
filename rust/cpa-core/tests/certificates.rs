//! The Python certificate tests ported to Rust: TestCertificateExpiry, TestCertificateDetails, TestCertificateUsage,
//! TestDownloadCertificate, TestGeneratePartnerKeyInfoCertidList, TestCreateX509Data, TestCreateRSAKeyValueElements,
//! TestCreateSubElements, TestReplaceXmlObject, TestCertificates (open_certificate) and TestLoadFunctionCertificates.
//! Python generated its certificates in the tests; here they come from tests/reference/cpa_certificates/fixtures.json
//! (made by tools/reference_cpa_certificates.py with the same names and dates).

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use chrono::{DateTime, TimeZone, Utc};
use cpa_core::certificates::{
    certificate_entries, certificate_expiries, certificate_usages, describe_reference, end_date_from_certificates,
    end_date_message, first_expiring_certificate, key_info_certificates, key_info_from_pem, key_info_to_pem,
    pem_file_name, replace_key_info, CertificateType,
};
use cpa_core::cpa::{party_infos, read_general};
use cpa_core::xml::{Document, NodeId};
use cpa_core::{CPA_NAMESPACE, DS_NAMESPACE};
use serde_json::Value;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn reference(name: &str) -> PathBuf {
    manifest().join("tests/reference/cpa_certificates").join(name)
}

fn fixtures() -> Value {
    serde_json::from_str(&fs::read_to_string(reference("fixtures.json")).unwrap()).unwrap()
}

/// The base64 DER of a generated certificate.
fn encode(name: &str) -> String {
    fixtures()[name]["der_base64"].as_str().unwrap().to_string()
}

fn utc(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
}

/// A minimal CPA like `generate_cpa` of TestCertificateExpiry: per party a list of (certId, X509Certificate texts).
fn generate_cpa(parties: &[Vec<(&str, Vec<String>)>]) -> Document {
    let mut xml = format!(
        "<tns:CollaborationProtocolAgreement xmlns:tns=\"{CPA_NAMESPACE}\" xmlns:ds=\"{DS_NAMESPACE}\">\
         <tns:Start>2020-01-01T00:00:00Z</tns:Start><tns:End>2099-01-01T00:00:00Z</tns:End>"
    );
    for certificates in parties {
        xml.push_str("<tns:PartyInfo>");
        for (cert_id, chain) in certificates {
            xml.push_str(&format!("<tns:Certificate tns:certId=\"{cert_id}\"><ds:KeyInfo>"));
            for text in chain {
                xml.push_str(&format!("<ds:X509Data><ds:X509Certificate>{text}</ds:X509Certificate></ds:X509Data>"));
            }
            xml.push_str("</ds:KeyInfo></tns:Certificate>");
        }
        xml.push_str("</tns:PartyInfo>");
    }
    xml.push_str("</tns:CollaborationProtocolAgreement>");
    Document::parse(&xml).unwrap()
}

fn chain() -> Vec<String> {
    vec![encode("leaf"), encode("intermediate"), encode("root")]
}

fn end_text(doc: &Document) -> String {
    doc.text(doc.find(doc.root().unwrap(), Some(CPA_NAMESPACE), "End").unwrap()).unwrap()
}

fn key_infos(doc: &Document) -> Vec<NodeId> {
    doc.iter_named(doc.root().unwrap(), Some(DS_NAMESPACE), "KeyInfo").collect()
}

// ---------------------------------------------------------------------------------------------------------------
// TestCertificateExpiry
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn certificate_type_classifies_chain() {
    let doc = generate_cpa(&[vec![("A_cert", chain())], vec![]]);
    let (details, _) = key_info_certificates(&doc, key_infos(&doc).first().copied(), Utc::now());
    let kinds: Vec<CertificateType> = details.iter().map(|d| d.kind).collect();
    assert_eq!(kinds, [CertificateType::Leaf, CertificateType::Intermediate, CertificateType::Root]);
    assert_eq!(CertificateType::Intermediate.to_string(), "intermediate");
}

#[test]
fn get_certificate_expiries_reads_whole_chain() {
    let doc = generate_cpa(&[vec![("A_cert", chain())], vec![("B_cert", chain()[..1].to_vec())]]);
    let (expiries, errors) = certificate_expiries(&doc);
    assert!(errors.is_empty());
    let mut found: Vec<(String, String)> =
        expiries.iter().map(|e| (e.cert_id.clone().unwrap(), e.kind.to_string())).collect();
    found.sort();
    let mut expected: Vec<(String, String)> =
        [("A_cert", "leaf"), ("A_cert", "intermediate"), ("A_cert", "root"), ("B_cert", "leaf")]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
    expected.sort();
    assert_eq!(found, expected);
}

#[test]
fn get_certificate_expiries_sorted_on_expiry() {
    let doc = generate_cpa(&[vec![("A_cert", chain())], vec![]]);
    let names: Vec<String> = certificate_expiries(&doc).0.into_iter().map(|e| e.common_name).collect();
    assert_eq!(names, ["Intermediate", "Leaf", "Root"]);
}

#[test]
fn certificate_common_name() {
    let doc = generate_cpa(&[vec![("A_cert", vec![encode("leaf"), encode("no_cn")])], vec![]]);
    let (details, _) = key_info_certificates(&doc, key_infos(&doc).first().copied(), Utc::now());
    assert_eq!(details[0].common_name, "Leaf");
    // Without common name: the full subject
    assert_eq!(details[1].common_name, "O=Only Org");
}

#[test]
fn get_certificate_expiries_skips_unreadable_certificate() {
    let doc = generate_cpa(&[vec![("A_cert", vec!["bm90IGEgY2VydA==".to_string(), encode("leaf")])], vec![]]);
    let (expiries, errors) = certificate_expiries(&doc);
    assert_eq!(expiries.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("Could not read a certificate of certId 'A_cert': "), "{}", errors[0]);
}

#[test]
fn get_first_expiring_certificate_returns_earliest() {
    let doc = generate_cpa(&[vec![("A_cert", chain()[..1].to_vec())], vec![("B_cert", chain())]]);
    let first = first_expiring_certificate(&doc).unwrap();
    assert_eq!(first.kind, CertificateType::Intermediate);
    assert_eq!(first.cert_id.as_deref(), Some("B_cert"));
    assert_eq!(first.subject, "CN=Intermediate");
    assert_eq!(first.not_after, utc(2030, 6, 15, 12, 30, 5));
}

#[test]
fn get_first_expiring_certificate_without_certificates() {
    assert!(first_expiring_certificate(&generate_cpa(&[vec![], vec![]])).is_none());
}

// ---------------------------------------------------------------------------------------------------------------
// TestSetEndDateFromCertificates
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn set_end_date_from_certificates_uses_first_expiring() {
    let mut doc = generate_cpa(&[vec![("A_cert", chain())], vec![]]);
    let (end_date, expiry, _) = end_date_from_certificates(&mut doc).unwrap();
    assert_eq!(end_date, "2030-06-15T12:30:05Z");
    assert_eq!(end_text(&doc), "2030-06-15T12:30:05Z");
    let message = end_date_message(&end_date, &expiry);
    assert!(message.contains("intermediate certificate"), "{message}");
    assert_eq!(
        message,
        "End date set to 2030-06-15T12:30:05Z: expiry of intermediate certificate 'CN=Intermediate' (certId A_cert)"
    );
    assert!(expiry.not_after > Utc::now(), "fixture should not be expired yet");
}

#[test]
fn set_end_date_from_certificates_lists_certificates_in_debug() {
    let mut doc = generate_cpa(&[vec![("A_cert", chain())], vec![]]);
    let (_, _, debug_lines) = end_date_from_certificates(&mut doc).unwrap();
    assert_eq!(
        debug_lines,
        [
            "Certificate 1/3: Intermediate expires 2030-06-15T12:30:05Z (intermediate, certId A_cert)",
            "Certificate 2/3: Leaf expires 2035-01-01T00:00:00Z (leaf, certId A_cert)",
            "Certificate 3/3: Root expires 2040-01-01T00:00:00Z (root, certId A_cert)",
        ]
    );
}

#[test]
fn set_end_date_from_certificates_without_certificates() {
    let mut doc = generate_cpa(&[vec![], vec![]]);
    assert_eq!(
        end_date_from_certificates(&mut doc).unwrap_err(),
        "No certificates found in the CPA, end date not changed"
    );
    assert_eq!(end_text(&doc), "2099-01-01T00:00:00Z");
}

// ---------------------------------------------------------------------------------------------------------------
// TestCertificateDetails
// ---------------------------------------------------------------------------------------------------------------

fn details_cpa() -> Document {
    generate_cpa(&[
        vec![("A_cert", vec![encode("leaf_of_root"), encode("root")])],
        vec![("B_old", vec![encode("old")]), ("B_empty", vec![])],
    ])
}

#[test]
fn get_key_info_certificates_reads_details() {
    let doc = details_cpa();
    let (details, errors) = key_info_certificates(&doc, Some(key_infos(&doc)[0]), Utc::now());
    assert!(errors.is_empty());
    let (leaf, root) = (&details[0], &details[1]);
    assert_eq!(
        (leaf.kind, leaf.common_name.as_str(), leaf.subject.as_str(), leaf.issuer.as_str()),
        (CertificateType::Leaf, "Leaf", "CN=Leaf", "CN=Root")
    );
    assert_eq!(leaf.serial_number, fixtures()["leaf_of_root"]["serial"].as_str().unwrap());
    assert_eq!(leaf.serial_number, "F0E1D2C3B4A5968778695A4B3C2D1E0F");
    assert_eq!(leaf.not_before, utc(2020, 1, 1, 0, 0, 0));
    assert_eq!(leaf.not_after, utc(2035, 6, 15, 12, 30, 5));
    assert_eq!((root.kind, root.common_name.as_str()), (CertificateType::Root, "Root"));
}

#[test]
fn get_key_info_certificates_marks_expired() {
    let doc = details_cpa();
    let (details, _) = key_info_certificates(&doc, Some(key_infos(&doc)[0]), utc(2036, 1, 1, 0, 0, 0));
    let expired: Vec<bool> = details.iter().map(|d| d.expired).collect();
    assert_eq!(expired, [true, false]);
}

#[test]
fn get_key_info_certificates_without_key_info() {
    let doc = details_cpa();
    let (details, errors) = key_info_certificates(&doc, None, Utc::now());
    assert!(details.is_empty() && errors.is_empty());
}

#[test]
fn load_shows_details_of_certificates() {
    // The logic behind the rows of the Certificates tab: per certId its KeyInfo and the certificates in it
    let doc = details_cpa();
    let parties = party_infos(&doc);
    let (entries, errors) = certificate_entries(&doc, parties[1]);
    assert!(errors.is_empty());
    let ids: Vec<&str> = entries.iter().map(|e| e.cert_id.as_str()).collect();
    assert_eq!(ids, ["B_old", "B_empty"]);
    let (empty, _) = key_info_certificates(&doc, entries[1].key_info, Utc::now());
    assert!(empty.is_empty());
}

#[test]
fn load_marks_expired_certificate() {
    let doc = details_cpa();
    let (entries, _) = certificate_entries(&doc, party_infos(&doc)[1]);
    let (details, _) = key_info_certificates(&doc, entries[0].key_info, Utc::now());
    assert_eq!(details[0].common_name, "Old");
    assert_eq!(details[0].not_after, utc(2021, 1, 1, 0, 0, 0));
    assert!(details[0].expired);
}

#[test]
fn open_certificate_replaces_certificate_details() {
    let mut doc = details_cpa();
    let (entries, _) = certificate_entries(&doc, party_infos(&doc)[1]);
    let entry = entries[0].clone();
    let pem = fs::read_to_string(reference("new_rsa.pem")).unwrap();
    let new_key_info = key_info_from_pem(&mut doc, &pem).unwrap();
    replace_key_info(&mut doc, entry.certificate, entry.key_info, new_key_info);
    let (entries, _) = certificate_entries(&doc, party_infos(&doc)[1]);
    assert_eq!(entries[0].key_info, Some(new_key_info));
    assert!(!doc.is_attached(entry.key_info.unwrap()));
    let (details, _) = key_info_certificates(&doc, entries[0].key_info, Utc::now());
    assert_eq!(details.len(), 1);
    assert_eq!(
        (details[0].kind, details[0].common_name.as_str(), details[0].not_after),
        (CertificateType::Leaf, "New", utc(2045, 1, 1, 0, 0, 0))
    );
}

#[test]
fn open_certificate_replaces_empty_key_info() {
    // Deviation from Python: an old KeyInfo without children is replaced too
    let mut doc = details_cpa();
    let (entries, _) = certificate_entries(&doc, party_infos(&doc)[1]);
    let entry = entries[1].clone();
    let pem = fs::read_to_string(reference("new_rsa.pem")).unwrap();
    let new_key_info = key_info_from_pem(&mut doc, &pem).unwrap();
    replace_key_info(&mut doc, entry.certificate, entry.key_info, new_key_info);
    let key_info_children: Vec<NodeId> = doc.find_all(entry.certificate, Some(DS_NAMESPACE), "KeyInfo").collect();
    assert_eq!(key_info_children, [new_key_info]);
}

// ---------------------------------------------------------------------------------------------------------------
// TestCertificateUsage
// ---------------------------------------------------------------------------------------------------------------

const USAGE_CPA: &str = r#"<tns:CollaborationProtocolAgreement xmlns:tns="http://www.oasis-open.org/committees/ebxml-cppa/schema/cpp-cpa-2_0.xsd">
  <tns:PartyInfo tns:partyName="A">
    <tns:CollaborationRole>
      <tns:ApplicationCertificateRef tns:certId="A_app"/>
    </tns:CollaborationRole>
    <tns:Certificate tns:certId="A_client"/>
    <tns:Certificate tns:certId="A_app"/>
    <tns:Certificate tns:certId="A_unused"/>
    <tns:SecurityDetails tns:securityId="A_security">
      <tns:TrustAnchors>
        <tns:AnchorCertificateRef tns:certId="A_client"/>
      </tns:TrustAnchors>
    </tns:SecurityDetails>
    <tns:Transport tns:transportId="A_transport">
      <tns:TransportSender>
        <!-- comment between the elements -->
        <tns:TransportClientSecurity>
          <tns:ClientCertificateRef tns:certId="A_client"/>
        </tns:TransportClientSecurity>
      </tns:TransportSender>
    </tns:Transport>
  </tns:PartyInfo>
</tns:CollaborationProtocolAgreement>"#;

fn find(doc: &Document, local: &str) -> NodeId {
    doc.iter_named(doc.root().unwrap(), Some(CPA_NAMESPACE), local).next().unwrap()
}

#[test]
fn describe_reference_names_element_with_id() {
    let doc = Document::parse(USAGE_CPA).unwrap();
    assert_eq!(
        describe_reference(&doc, find(&doc, "ClientCertificateRef")),
        "ClientCertificateRef in Transport 'A_transport'"
    );
    assert_eq!(
        describe_reference(&doc, find(&doc, "AnchorCertificateRef")),
        "AnchorCertificateRef in SecurityDetails 'A_security'"
    );
}

#[test]
fn describe_reference_falls_back_to_parent() {
    let doc = Document::parse(USAGE_CPA).unwrap();
    assert_eq!(
        describe_reference(&doc, find(&doc, "ApplicationCertificateRef")),
        "ApplicationCertificateRef in CollaborationRole"
    );
    assert_eq!(describe_reference(&doc, doc.root().unwrap()), "CollaborationProtocolAgreement");
}

#[test]
fn get_certificate_usages_collects_references() {
    let doc = Document::parse(USAGE_CPA).unwrap();
    let usages = certificate_usages(&doc);
    assert_eq!(usages.len(), 2);
    assert_eq!(usages["A_app"], ["ApplicationCertificateRef in CollaborationRole"]);
    assert_eq!(
        usages["A_client"],
        ["AnchorCertificateRef in SecurityDetails 'A_security'", "ClientCertificateRef in Transport 'A_transport'"]
    );
}

#[test]
fn get_certificate_usages_ignores_certificate_element() {
    let doc = Document::parse(USAGE_CPA).unwrap();
    assert!(!certificate_usages(&doc).contains_key("A_unused"));
}

#[test]
fn get_certificate_usages_of_full_cpa() {
    let path = manifest().join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let doc = Document::parse_bytes(&fs::read(path).unwrap()).unwrap();
    assert_eq!(
        certificate_usages(&doc)["BROKER_ClientCert"],
        [
            "AnchorCertificateRef in SecurityDetails 'BROKER_TransportSecurity'",
            "ClientCertificateRef in Transport 'BROKER_T_HTTPS'"
        ]
    );
}

// ---------------------------------------------------------------------------------------------------------------
// TestDownloadCertificate
// ---------------------------------------------------------------------------------------------------------------

fn download_cpa() -> Document {
    generate_cpa(&[vec![("A/cert 1", vec![encode("leaf"), encode("root")])], vec![("B_cert", vec![])]])
}

fn pem_of(base64_der: &str) -> String {
    let lines: Vec<&str> =
        base64_der.as_bytes().chunks(64).map(|chunk| std::str::from_utf8(chunk).unwrap()).collect();
    format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n", lines.join("\n"))
}

#[test]
fn key_info_to_pem_returns_chain_in_order() {
    let doc = download_cpa();
    let pem = key_info_to_pem(&doc, Some(key_infos(&doc)[0])).unwrap();
    assert_eq!(pem, format!("{}{}", pem_of(&encode("leaf")), pem_of(&encode("root"))));
    assert_eq!(pem.matches("BEGIN CERTIFICATE").count(), 2);
}

#[test]
fn key_info_to_pem_without_certificates() {
    let doc = download_cpa();
    assert_eq!(key_info_to_pem(&doc, None).unwrap(), "");
    assert_eq!(key_info_to_pem(&doc, Some(key_infos(&doc)[1])).unwrap(), "");
}

#[test]
fn save_certificate_file_name() {
    assert_eq!(pem_file_name(Some("A/cert 1")), "A_cert_1.pem");
    assert_eq!(pem_file_name(Some("BROKER_Client.Cert-1")), "BROKER_Client.Cert-1.pem");
    assert_eq!(pem_file_name(None), "certificate.pem");
    assert_eq!(pem_file_name(Some("")), "certificate.pem");
}

#[test]
fn key_info_to_pem_reports_unreadable_certificate() {
    let doc = generate_cpa(&[vec![("A", vec!["bm90IGEgY2VydA==".to_string()])], vec![]]);
    assert!(key_info_to_pem(&doc, Some(key_infos(&doc)[0])).is_err());
}

// ---------------------------------------------------------------------------------------------------------------
// TestGeneratePartnerKeyInfoCertidList and TestLoadFunctionCertificates
// ---------------------------------------------------------------------------------------------------------------

fn party(xml_inside: &str) -> (Document, NodeId) {
    let doc = Document::parse(&format!(
        "<tns:PartyInfo xmlns:tns=\"{CPA_NAMESPACE}\" xmlns:ds=\"{DS_NAMESPACE}\">{xml_inside}</tns:PartyInfo>"
    ))
    .unwrap();
    let root = doc.root().unwrap();
    (doc, root)
}

#[test]
fn generate_partner_keyinfo_certid_list_with_valid_data() {
    let (doc, party_info) =
        party("<tns:Certificate tns:certId=\"123\"><ds:KeyInfo><ds:KeyValue/></ds:KeyInfo></tns:Certificate>");
    let (entries, errors) = certificate_entries(&doc, party_info);
    assert!(errors.is_empty());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].cert_id, "123");
    assert!(entries[0].key_info.is_some());
}

#[test]
fn generate_partner_keyinfo_certid_list_with_missing_cert_id() {
    let (doc, party_info) = party("<tns:Certificate/>");
    let (entries, errors) = certificate_entries(&doc, party_info);
    assert!(entries.is_empty());
    assert_eq!(errors, ["Certificate ID attribute not found for certificate element in CPA please check the CPA"]);
}

#[test]
fn generate_partner_keyinfo_certid_list_with_no_key_info() {
    let (doc, party_info) = party("<tns:Certificate tns:certId=\"123\"/>");
    let (entries, errors) = certificate_entries(&doc, party_info);
    assert_eq!(entries[0].cert_id, "123");
    assert_eq!(entries[0].key_info, None);
    assert_eq!(errors, ["KeyInfo not found for certificate ID: 123"]);
}

#[test]
fn generate_partner_keyinfo_certid_list_follows_python_dict() {
    let (doc, party_info) = party(
        "<tns:Certificate tns:certId=\"a\"/><tns:Certificate tns:certId=\"\"/>\
         <tns:Certificate tns:certId=\"b\"><ds:KeyInfo/></tns:Certificate>\
         <tns:Certificate tns:certId=\"a\"><ds:KeyInfo/></tns:Certificate>",
    );
    let (entries, errors) = certificate_entries(&doc, party_info);
    let ids: Vec<&str> = entries.iter().map(|e| e.cert_id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
    // The second "a" replaced the first, at the position of the first
    assert!(entries[0].key_info.is_some());
    assert_eq!(errors, ["KeyInfo not found for certificate ID: a"]);
}

#[test]
fn load_logs_error_when_no_certificates_found() {
    let doc = generate_cpa(&[vec![], vec![]]);
    for party_info in party_infos(&doc) {
        assert!(certificate_entries(&doc, party_info).0.is_empty());
    }
}

#[test]
fn load_populates_tree_with_certificates() {
    let doc = generate_cpa(&[vec![("A_cert", chain())], vec![("B_cert", vec![])]]);
    let parties = party_infos(&doc);
    assert_eq!(certificate_entries(&doc, parties[0]).0.len(), 1);
    assert_eq!(certificate_entries(&doc, parties[1]).0.len(), 1);
}

// ---------------------------------------------------------------------------------------------------------------
// open_certificate: TestCertificates, TestCreateX509Data, TestCreateRSAKeyValueElements, TestCreateSubElements
// ---------------------------------------------------------------------------------------------------------------

fn child_names(doc: &Document, node: NodeId) -> Vec<String> {
    doc.child_elements(node).map(|child| doc.name(child).unwrap().clark()).collect()
}

#[test]
fn open_certificate_valid_file_builds_key_info() {
    let mut doc = Document::parse("<a/>").unwrap();
    let pem = fs::read_to_string(reference("new_rsa.pem")).unwrap();
    let key_info = key_info_from_pem(&mut doc, &pem).unwrap();
    let ds = |local: &str| format!("{{{DS_NAMESPACE}}}{local}");
    assert_eq!(doc.name(key_info).unwrap().clark(), ds("KeyInfo"));
    assert_eq!(child_names(&doc, key_info), [ds("KeyValue"), ds("X509Data")]);
    let key_value = doc.find(key_info, Some(DS_NAMESPACE), "KeyValue").unwrap();
    let rsa_key_value = doc.find(key_value, Some(DS_NAMESPACE), "RSAKeyValue").unwrap();
    assert_eq!(child_names(&doc, rsa_key_value), [ds("Modulus"), ds("Exponent")]);
    let exponent = doc.find(rsa_key_value, Some(DS_NAMESPACE), "Exponent").unwrap();
    assert_eq!(doc.text(exponent).unwrap(), "AQAB");
    let modulus = doc.find(rsa_key_value, Some(DS_NAMESPACE), "Modulus").unwrap();
    let modulus_bytes = base64::engine::general_purpose::STANDARD.decode(doc.text(modulus).unwrap()).unwrap();
    assert_eq!(modulus_bytes.len(), 256, "2048-bit modulus without leading zero byte");
    // X509DataType is a sequence: X509IssuerSerial before X509SubjectName and X509Certificate
    let x509_data = doc.find(key_info, Some(DS_NAMESPACE), "X509Data").unwrap();
    assert_eq!(child_names(&doc, x509_data), [ds("X509IssuerSerial"), ds("X509SubjectName"), ds("X509Certificate")]);
    let issuer_serial = doc.find(x509_data, Some(DS_NAMESPACE), "X509IssuerSerial").unwrap();
    assert_eq!(child_names(&doc, issuer_serial), [ds("X509IssuerName"), ds("X509SerialNumber")]);
    assert_eq!(
        doc.element_to_xml(issuer_serial),
        format!(
            "<ds:X509IssuerSerial xmlns:ds=\"{DS_NAMESPACE}\"><ds:X509IssuerName>CN=New</ds:X509IssuerName>\
             <ds:X509SerialNumber>9223372036854775807</ds:X509SerialNumber></ds:X509IssuerSerial>"
        )
    );
}

#[test]
fn open_certificate_invalid_certificate() {
    let mut doc = Document::parse("<a/>").unwrap();
    let error = key_info_from_pem(&mut doc, "invalid_certificate_data").unwrap_err();
    assert!(error.starts_with("Unable to load PEM file."), "{error}");
    let error = key_info_from_pem(&mut doc, "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----").unwrap_err();
    assert_eq!(error, "Valid PEM but no BEGIN CERTIFICATE/END CERTIFICATE delimiters. Are you sure this is a certificate?");
    assert!(key_info_from_pem(&mut doc, "").is_err());
}

#[test]
fn open_certificate_needs_rsa_leaf() {
    let mut doc = Document::parse("<a/>").unwrap();
    let error = key_info_from_pem(&mut doc, &pem_of(&encode("leaf"))).unwrap_err();
    assert_eq!(error, "'EllipticCurvePublicNumbers' object has no attribute 'n'");
}

#[test]
fn open_certificate_full_chain_keeps_order() {
    let path = manifest().join("../../application/tests/resources/certs/leaf/certs/Leaf Certificate 1_full_chain.pem");
    let mut doc = Document::parse("<a/>").unwrap();
    let key_info = key_info_from_pem(&mut doc, &fs::read_to_string(path).unwrap()).unwrap();
    let (details, errors) = key_info_certificates(&doc, Some(key_info), Utc::now());
    assert!(errors.is_empty());
    let names: Vec<&str> = details.iter().map(|d| d.common_name.as_str()).collect();
    assert_eq!(names.len(), 3);
    assert_eq!(details[2].kind, CertificateType::Root);
    // The intermediate of this test chain has no basicConstraints CA, so it counts as a leaf (as in Python)
    assert_eq!(details[1].kind, CertificateType::Leaf);
}

// ---------------------------------------------------------------------------------------------------------------
// TestReplaceXmlObject
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn replace_xml_object_appends_new_element_when_xml_element_is_none() {
    let mut doc = Document::parse(&format!(
        "<tns:Certificate xmlns:tns=\"{CPA_NAMESPACE}\" xmlns:ds=\"{DS_NAMESPACE}\" tns:certId=\"c\">\n</tns:Certificate>"
    ))
    .unwrap();
    let certificate = doc.root().unwrap();
    let pem = fs::read_to_string(reference("new_rsa.pem")).unwrap();
    let new = key_info_from_pem(&mut doc, &pem).unwrap();
    replace_key_info(&mut doc, certificate, None, new);
    assert_eq!(doc.child_elements(certificate).collect::<Vec<_>>(), [new]);
    // ds is in scope: the new KeyInfo does not declare it again
    assert!(doc.element(new).unwrap().namespaces.is_empty());
    assert!(doc.to_xml().contains("\n<ds:KeyInfo><ds:KeyValue>"));
}

#[test]
fn replace_xml_object_replaces_existing_element_with_new_element() {
    let mut doc = Document::parse(&format!(
        "<tns:Certificate xmlns:tns=\"{CPA_NAMESPACE}\" tns:certId=\"c\">\n  \
         <ds:KeyInfo xmlns:ds=\"{DS_NAMESPACE}\"><ds:KeyName>old</ds:KeyName></ds:KeyInfo>\n</tns:Certificate>"
    ))
    .unwrap();
    let certificate = doc.root().unwrap();
    let old = doc.find(certificate, Some(DS_NAMESPACE), "KeyInfo").unwrap();
    let pem = fs::read_to_string(reference("new_rsa.pem")).unwrap();
    let new = key_info_from_pem(&mut doc, &pem).unwrap();
    replace_key_info(&mut doc, certificate, Some(old), new);
    assert_eq!(doc.child_elements(certificate).collect::<Vec<_>>(), [new]);
    // ds is not in scope at the Certificate: the new KeyInfo declares it; the tail of the old one is gone
    let xml = doc.to_xml();
    assert!(xml.contains(&format!("c\">\n  <ds:KeyInfo xmlns:ds=\"{DS_NAMESPACE}\"><ds:KeyValue>")), "{xml}");
    assert!(xml.ends_with("</ds:KeyInfo></tns:Certificate>"));
}

#[test]
fn read_general_of_generated_cpa() {
    // generate_cpa has no Status, cpaid or PartyId: every field but the dates is empty
    let fields = read_general(&generate_cpa(&[vec![], vec![]]));
    assert_eq!(fields.cpa_id, "");
    assert_eq!(fields.party_ids, ["", ""]);
    assert_eq!(fields.start_date, "2020-01-01T00:00:00Z");
}

/// Stripping removes every certificate and every reference to one, turns https endpoints into http, and keeps the
/// CPA valid: the dangling AnchorCertificateRef of the test CPA goes with its TrustAnchors.
#[test]
fn strip_certificates_removes_certificates_and_references() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let mut doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let report = cpa_core::certificates::strip_certificates(&mut doc);

    assert_eq!(report.cert_ids.len(), 4, "{report:?}");
    assert!(report.removed.iter().any(|(name, _)| *name == "TransportClientSecurity"), "{report:?}");
    assert!(report.removed.iter().any(|(name, _)| *name == "TrustAnchors"), "{report:?}");
    assert!(!report.endpoints.is_empty());
    assert!(report.endpoints.iter().all(|(_, old, new)| old.starts_with("https://") && new.starts_with("http://")));

    let xml = doc.to_xml();
    for gone in ["<tns:Certificate ", "CertificateRef", "TransportServerSecurity", "TransportClientSecurity", "https://party"] {
        assert!(!xml.contains(gone), "{gone} is still in the CPA");
    }
    assert!(xml.contains("<tns:SecurityDetails"), "SecurityDetails stays for its securityId references");
    let errors = cpa_core::validation::validate_document(&doc).unwrap();
    assert!(errors.is_empty(), "{errors:?}");
    // A second time changes nothing
    assert_eq!(cpa_core::certificates::strip_certificates(&mut doc), Default::default());
}

#[test]
fn add_certificate_after_stripping() {
    use cpa_core::certificates::{add_certificate, certificate_entries, check_new_cert_id, new_cert_id, strip_certificates};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let mut doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(&path).unwrap()).unwrap();
    // The PEM chain of a certId in the test CPA
    let party = cpa_core::cpa::party_infos(&doc)[0];
    let (entries, _) = certificate_entries(&doc, party);
    let pem = cpa_core::certificates::key_info_to_pem(&doc, entries[0].key_info).unwrap();
    assert!(check_new_cert_id(&doc, &entries[0].cert_id).is_some(), "a certId in use is refused");

    strip_certificates(&mut doc);
    let cert_id = new_cert_id(&doc, 0);
    assert_eq!(cert_id, "Broker_Cert");
    let before = doc.to_xml();
    assert!(add_certificate(&mut doc, 0, "with space", &pem).is_err());
    assert!(add_certificate(&mut doc, 0, &cert_id, "no pem").is_err());
    assert_eq!(doc.to_xml(), before, "nothing changes on an error");

    add_certificate(&mut doc, 0, &cert_id, &pem).unwrap();
    let party = cpa_core::cpa::party_infos(&doc)[0];
    let (entries, errors) = certificate_entries(&doc, party);
    assert!(errors.is_empty());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].cert_id, "Broker_Cert");
    assert_eq!(cpa_core::certificates::key_info_to_pem(&doc, entries[0].key_info).unwrap(), pem);
    assert_eq!(new_cert_id(&doc, 0), "Broker_Cert2");
    let xml = doc.to_xml();
    assert!(xml.contains("</tns:CollaborationRole>\n        <tns:Certificate tns:certId=\"Broker_Cert\">\n            <ds:KeyInfo>\n                <ds:KeyValue>"));
    assert!(xml.contains("</ds:KeyInfo>\n        </tns:Certificate>\n        <tns:SecurityDetails"));
    assert!(cpa_core::validation::validate_document(&doc).unwrap().is_empty());
}

#[test]
fn link_certificate_restores_tls_after_stripping() {
    use cpa_core::certificates::{add_certificate, certificate_entries, link_certificate, strip_certificates, CertificateUse};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let mut doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let party = cpa_core::cpa::party_infos(&doc)[0];
    let pem = cpa_core::certificates::key_info_to_pem(&doc, certificate_entries(&doc, party).0[0].key_info).unwrap();
    strip_certificates(&mut doc);
    add_certificate(&mut doc, 0, "Broker_Cert", &pem).unwrap();

    let uses = CertificateUse { tls_server: true, tls_client: true, trust_anchor: true };
    let report = link_certificate(&mut doc, 0, "Broker_Cert", uses).unwrap();
    assert_eq!(
        report.endpoints,
        vec![(
            "BROKER_T_HTTPS".to_string(),
            "http://party-a.example.com/exchange/PARTNER_A".to_string(),
            "https://party-a.example.com/exchange/PARTNER_A".to_string()
        )]
    );
    assert!(report.changes.contains(&"ServerCertificateRef of transport BROKER_T_HTTPS set to Broker_Cert".to_string()));
    let xml = doc.to_xml();
    for expected in [
        "<tns:SecurityDetails tns:securityId=\"BROKER_TransportSecurity\">\n            <tns:TrustAnchors>\n                <tns:AnchorCertificateRef tns:certId=\"Broker_Cert\"/>\n            </tns:TrustAnchors>\n        </tns:SecurityDetails>",
        "<tns:TransportClientSecurity>\n                    <tns:TransportSecurityProtocol tns:version=\"1.2\">TLS</tns:TransportSecurityProtocol>\n                    <tns:ClientCertificateRef tns:certId=\"Broker_Cert\"/>\n                    <tns:ServerSecurityDetailsRef tns:securityId=\"BROKER_TransportSecurity\"/>\n                </tns:TransportClientSecurity>\n            </tns:TransportSender>",
        "tns:type=\"allPurpose\"/>\n                <tns:TransportServerSecurity>\n                    <tns:TransportSecurityProtocol tns:version=\"1.2\">TLS</tns:TransportSecurityProtocol>\n                    <tns:ServerCertificateRef tns:certId=\"Broker_Cert\"/>\n                    <tns:ClientSecurityDetailsRef tns:securityId=\"BROKER_TransportSecurity\"/>\n                </tns:TransportServerSecurity>\n            </tns:TransportReceiver>",
    ] {
        assert!(xml.contains(expected), "missing:\n{expected}");
    }
    assert!(cpa_core::validation::validate_document(&doc).unwrap().is_empty());
    // The certificate is now in use
    assert!(cpa_core::certificates::certificate_usages(&doc)["Broker_Cert"].len() >= 3);

    // Linking again changes nothing
    let before = doc.to_xml();
    let again = link_certificate(&mut doc, 0, "Broker_Cert", uses).unwrap();
    assert!(again.endpoints.is_empty());
    assert_eq!(doc.to_xml(), before);
}

#[test]
fn link_certificate_in_a_cpa_without_security_details() {
    use cpa_core::certificates::{add_certificate, certificate_entries, link_certificate, CertificateUse};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas");
    let source = cpa_core::xml::Document::parse_bytes(&std::fs::read(dir.join("full_test_cpa.xml")).unwrap()).unwrap();
    let party = cpa_core::cpa::party_infos(&source)[0];
    let pem = cpa_core::certificates::key_info_to_pem(&source, certificate_entries(&source, party).0[0].key_info).unwrap();
    let mut doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(dir.join("full_cpa_p.xml")).unwrap()).unwrap();
    let cert_id = cpa_core::certificates::new_cert_id(&doc, 0);
    add_certificate(&mut doc, 0, &cert_id, &pem).unwrap();
    let uses = CertificateUse { tls_server: true, tls_client: true, trust_anchor: true };
    let report = link_certificate(&mut doc, 0, &cert_id, uses).unwrap();
    assert!(report.changes.iter().any(|c| c.starts_with("SecurityDetails ")), "{:?}", report.changes);
    let errors = cpa_core::validation::validate_document(&doc).unwrap();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn replace_one_expired_certificate_of_a_chain() {
    use cpa_core::certificates::{certificate_entries, key_info_certificates, replace_chain_certificate};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renewed = std::fs::read_to_string(root.join("tests/reference/cpa_certificates/renewed_leaf.pem")).unwrap();
    let full_chain = std::fs::read_to_string(
        root.join("../../application/tests/resources/certs/leaf/certs/Leaf Certificate 2_full_chain.pem"),
    )
    .unwrap();
    let path = root.join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let mut doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let party = cpa_core::cpa::party_infos(&doc)[0];
    let key_info = certificate_entries(&doc, party).0[0].key_info.unwrap();
    let now = chrono::Utc::now();
    let before = key_info_certificates(&doc, Some(key_info), now).0;
    let errors_before = cpa_core::validation::validate_document(&doc).unwrap();
    assert_eq!(before.len(), 3);
    assert!(before[0].expired);

    // A whole chain is refused, nothing changes
    let xml = doc.to_xml();
    assert!(replace_chain_certificate(&mut doc, key_info, 0, &full_chain).unwrap_err().contains("3 certificates"));
    assert_eq!(doc.to_xml(), xml);

    // The renewed leaf: same subject and issuer, so no warnings; intermediate and root stay
    let report = replace_chain_certificate(&mut doc, key_info, 0, &renewed).unwrap();
    assert_eq!(report.role, "Leaf");
    assert_eq!(report.new_name, "Leaf Certificate 2");
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let after = key_info_certificates(&doc, Some(key_info), now).0;
    assert_eq!(after.len(), 3);
    assert!(!after[0].expired);
    assert_eq!(after[0].serial_number, "1092");
    assert_eq!(after[1], before[1]);
    assert_eq!(after[2], before[2]);
    assert_eq!(cpa_core::validation::validate_document(&doc).unwrap(), errors_before);

    // The leaf in place of the intermediate: warned about
    let report = replace_chain_certificate(&mut doc, key_info, 1, &renewed).unwrap();
    assert_eq!(report.role, "Intermediate");
    assert!(report.warnings.iter().any(|w| w.starts_with("The subject changes")), "{:?}", report.warnings);
}
