//! Compares the certificate details, expiry lists, usages and certificate entries of every test CPA, the PEM files
//! of the test resources, the generated fixture certificates and describe_cpa_date with what the Python version
//! gives (tools/reference_cpa_certificates.py).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{FixedOffset, Utc};
use cpa_core::certificates::{
    certificate_entries, certificate_expiries, certificate_usages, key_info_certificates, key_info_from_pem,
    CertificateDetails,
};
use cpa_core::cpa::party_infos;
use cpa_core::dates::{describe_cpa_date, format_cpa_date};
use cpa_core::xml::{Document, NodeId};
use cpa_core::DS_NAMESPACE;
use serde_json::Value;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn reference(name: &str) -> PathBuf {
    manifest().join("tests/reference/cpa_certificates").join(name)
}

fn json(name: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(reference(name)).unwrap()).unwrap()
}

fn load_cpa(name: &str) -> Document {
    let path = manifest().join("../../application/tests/resources/cpas").join(name);
    Document::parse_bytes(&fs::read(path).unwrap()).unwrap()
}

fn s(value: &Value) -> &str {
    value.as_str().unwrap_or_else(|| panic!("not a string: {value}"))
}

fn assert_details(context: &str, actual: &CertificateDetails, expected: &Value) {
    assert_eq!(actual.kind.to_string(), s(&expected["type"]), "{context}: type");
    assert_eq!(actual.common_name, s(&expected["common_name"]), "{context}: common name");
    assert_eq!(actual.subject, s(&expected["subject"]), "{context}: subject");
    assert_eq!(actual.issuer, s(&expected["issuer"]), "{context}: issuer");
    assert_eq!(actual.serial_number, s(&expected["serial"]), "{context}: serial");
    assert_eq!(format_cpa_date(actual.not_before), s(&expected["not_before"]), "{context}: not before");
    assert_eq!(format_cpa_date(actual.not_after), s(&expected["not_after"]), "{context}: not after");
}

/// A KeyInfo with one X509Certificate per base64 text.
fn key_info_document(certificates: &[&str]) -> (Document, NodeId) {
    let mut xml = format!("<ds:KeyInfo xmlns:ds=\"{DS_NAMESPACE}\">");
    for certificate in certificates {
        xml.push_str(&format!("<ds:X509Data><ds:X509Certificate>{certificate}</ds:X509Certificate></ds:X509Data>"));
    }
    xml.push_str("</ds:KeyInfo>");
    let doc = Document::parse(&xml).unwrap();
    let root = doc.root().unwrap();
    (doc, root)
}

#[test]
fn certificates_of_test_cpas_match_python() {
    let report = json("certificates.json");
    let now = Utc::now();
    let mut checked = 0;
    for (name, cpa_report) in report["cpas"].as_object().unwrap() {
        let doc = load_cpa(name);
        let root = doc.root().unwrap();
        let x509_certificates: Vec<NodeId> = doc.iter_named(root, Some(DS_NAMESPACE), "X509Certificate").collect();
        let expected = cpa_report["certificates"].as_array().unwrap();
        assert_eq!(x509_certificates.len(), expected.len(), "{name}");
        for (index, (&node, expected)) in x509_certificates.iter().zip(expected).enumerate() {
            let (details, errors) = key_info_certificates(&doc, Some(node), now);
            if expected.get("error").is_some() {
                assert!(details.is_empty() && errors.len() == 1, "{name} #{index}");
            } else {
                assert!(errors.is_empty(), "{name} #{index}: {errors:?}");
                assert_details(&format!("{name} #{index}"), &details[0], expected);
                checked += 1;
            }
        }

        let (expiries, errors) = certificate_expiries(&doc);
        assert_eq!(errors.len() as u64, cpa_report["expiry_errors"].as_u64().unwrap(), "{name}");
        let actual: Vec<Vec<String>> = expiries
            .iter()
            .map(|e| {
                vec![
                    e.cert_id.clone().unwrap_or_default(),
                    e.kind.to_string(),
                    e.common_name.clone(),
                    e.subject.clone(),
                    format_cpa_date(e.not_after),
                ]
            })
            .collect();
        let expected: Vec<Vec<String>> = serde_json::from_value(cpa_report["expiries"].clone()).unwrap();
        assert_eq!(actual, expected, "{name}: expiries");

        let usages: HashMap<String, Vec<String>> = serde_json::from_value(cpa_report["usages"].clone()).unwrap();
        assert_eq!(certificate_usages(&doc), usages, "{name}: usages");

        let entries = cpa_report["entries"].as_array().unwrap();
        let parties = party_infos(&doc);
        assert_eq!(parties.len(), entries.len(), "{name}");
        for (party, expected) in parties.iter().zip(entries) {
            let (actual, errors) = certificate_entries(&doc, *party);
            let cert_ids: Vec<String> = actual.iter().map(|e| e.cert_id.clone()).collect();
            let has_key_info: Vec<bool> = actual.iter().map(|e| e.key_info.is_some()).collect();
            assert_eq!(cert_ids, serde_json::from_value::<Vec<String>>(expected["cert_ids"].clone()).unwrap());
            assert_eq!(has_key_info, serde_json::from_value::<Vec<bool>>(expected["has_key_info"].clone()).unwrap());
            assert_eq!(errors, serde_json::from_value::<Vec<String>>(expected["errors"].clone()).unwrap());
        }
    }
    assert!(checked >= 20, "only {checked} certificates checked");
}

#[test]
fn pem_files_and_fixtures_match_python() {
    let report = json("certificates.json");
    let fixtures = json("fixtures.json");
    let mut sets: Vec<(String, Vec<Value>)> = report["pem_files"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, certificates)| (name.clone(), certificates.as_array().unwrap().clone()))
        .collect();
    sets.extend(fixtures.as_object().unwrap().iter().map(|(name, details)| (name.clone(), vec![details.clone()])));
    assert!(sets.len() >= 15);
    for (name, expected) in sets {
        let texts: Vec<&str> = expected.iter().map(|e| s(&e["der_base64"])).collect();
        let (doc, key_info) = key_info_document(&texts);
        let (details, errors) = key_info_certificates(&doc, Some(key_info), Utc::now());
        assert!(errors.is_empty(), "{name}: {errors:?}");
        assert_eq!(details.len(), expected.len(), "{name}");
        for (index, (actual, expected)) in details.iter().zip(&expected).enumerate() {
            assert_details(&format!("{name} #{index}"), actual, expected);
        }
    }
}

#[test]
fn key_info_from_pem_writes_python_names_and_decimal_serials() {
    let report = json("certificates.json");
    let fixtures = json("fixtures.json");
    let cases = [
        (
            manifest().join("../../application/tests/resources/certs/leaf/certs/Leaf Certificate 1_full_chain.pem"),
            report["pem_files"]["leaf/certs/Leaf Certificate 1_full_chain.pem"].clone(),
        ),
        (reference("new_rsa.pem"), Value::Array(vec![fixtures["new"].clone()])),
    ];
    for (path, expected) in cases {
        let mut doc = Document::parse("<a/>").unwrap();
        let key_info = key_info_from_pem(&mut doc, &fs::read_to_string(&path).unwrap()).unwrap();
        let x509_data: Vec<NodeId> = doc.find_all(key_info, Some(DS_NAMESPACE), "X509Data").collect();
        let expected = expected.as_array().unwrap();
        assert_eq!(x509_data.len(), expected.len());
        for (data, expected) in x509_data.iter().zip(expected) {
            let issuer_serial = doc.find(*data, Some(DS_NAMESPACE), "X509IssuerSerial").unwrap();
            let child = |parent: NodeId, local: &str| doc.text(doc.find(parent, Some(DS_NAMESPACE), local).unwrap());
            assert_eq!(child(issuer_serial, "X509IssuerName").unwrap(), s(&expected["issuer"]));
            assert_eq!(child(issuer_serial, "X509SerialNumber").unwrap(), s(&expected["serial_decimal"]));
            assert_eq!(child(*data, "X509SubjectName").unwrap(), s(&expected["subject"]));
            assert_eq!(child(*data, "X509Certificate").unwrap(), s(&expected["der_base64"]));
        }
    }
}

#[test]
fn describe_cpa_date_matches_python() {
    let report = json("certificates.json");
    let dates = report["dates"].as_array().unwrap();
    assert!(dates.len() > 200);
    for entry in dates {
        let text = s(&entry[0]);
        let offset = FixedOffset::east_opt(entry[1].as_i64().unwrap() as i32).unwrap();
        assert_eq!(describe_cpa_date(text, Some(offset)), s(&entry[2]), "{text:?} at offset {offset}");
    }
}
