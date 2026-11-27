//! Replays the operations of tools/reference_cpa_certificates.py (done there by the real Python code) and checks
//! that the Rust version writes the same bytes. Also the PEM export of a KeyInfo, the general fields and the
//! transports, cert ids and security ids per party of every test CPA.

use std::fs;
use std::path::{Path, PathBuf};

use cpa_core::certificates::{key_info_from_pem, key_info_to_pem, replace_key_info};
use cpa_core::cpa::{self, GeneralField};
use cpa_core::xml::{Document, NodeId};
use cpa_core::DS_NAMESPACE;
use serde_json::Value;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn reference(name: &str) -> PathBuf {
    manifest().join("tests/reference/cpa_certificates").join(name)
}

fn load_cpa(name: &str) -> Document {
    let path = manifest().join("../../application/tests/resources/cpas").join(name);
    Document::parse_bytes(&fs::read(path).unwrap()).unwrap()
}

fn json(name: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(reference(name)).unwrap()).unwrap()
}

/// The element at a path of element-child indexes from the root.
fn resolve(doc: &Document, path: &Value) -> NodeId {
    let mut node = doc.root().unwrap();
    for index in path.as_array().unwrap() {
        node = doc.child_elements(node).nth(index.as_u64().unwrap() as usize).unwrap();
    }
    node
}

fn path_of(doc: &Document, node: NodeId) -> Vec<usize> {
    let mut path = Vec::new();
    let mut node = node;
    while let Some(parent) = doc.parent_element(node) {
        path.insert(0, doc.child_elements(parent).position(|child| child == node).unwrap());
        node = parent;
    }
    path
}

fn text<'a>(op: &'a Value, key: &str) -> &'a str {
    op[key].as_str().unwrap_or_else(|| panic!("{key} missing in {op}"))
}

fn general_field(op: &Value) -> GeneralField {
    let index = op["index"].as_u64().map(|i| i as usize);
    match (text(op, "field"), index) {
        ("CpaId", _) => GeneralField::CpaId,
        ("Status", _) => GeneralField::Status,
        ("PartyName", Some(i)) => GeneralField::PartyName(i),
        ("PartyId", Some(i)) => GeneralField::PartyId(i),
        ("PartyIdType", Some(i)) => GeneralField::PartyIdType(i),
        ("StartDate", _) => GeneralField::StartDate,
        ("EndDate", _) => GeneralField::EndDate,
        other => panic!("unknown field {other:?}"),
    }
}

fn apply(doc: &mut Document, op: &Value) {
    let element = || op.get("element");
    match text(op, "op") {
        "set_general" => cpa::set_general(doc, general_field(op), text(op, "value")).unwrap(),
        "add_comment" => {
            cpa::add_comment(doc, text(op, "text"), text(op, "lang"));
        }
        "set_comment" => {
            let comment = cpa::comments(doc)[op["index"].as_u64().unwrap() as usize];
            cpa::set_comment(doc, comment, text(op, "text")).unwrap();
        }
        "delete_comment" => {
            let comment = cpa::comments(doc)[op["index"].as_u64().unwrap() as usize];
            cpa::delete_comment(doc, comment).unwrap();
        }
        "replace_key_info" => {
            let certificate = resolve(doc, element().unwrap());
            let old = doc.find(certificate, Some(DS_NAMESPACE), "KeyInfo");
            let pem = fs::read_to_string(manifest().join("..").join(text(op, "pem"))).unwrap();
            let new = key_info_from_pem(doc, &pem).unwrap();
            replace_key_info(doc, certificate, old, new);
        }
        "set_cpa_attribute" => {
            let node = resolve(doc, element().unwrap());
            cpa::set_cpa_attribute(doc, node, text(op, "local"), text(op, "value")).unwrap();
        }
        "set_element_text" => {
            let node = resolve(doc, element().unwrap());
            cpa::set_element_text(doc, node, text(op, "value")).unwrap();
        }
        "clone_element" => {
            let node = resolve(doc, element().unwrap());
            cpa::clone_element(doc, node).unwrap();
        }
        "editor_set_attribute" | "editor_add_attribute" => {
            let node = resolve(doc, element().unwrap());
            cpa::set_attribute_by_name(doc, node, text(op, "name"), text(op, "value")).unwrap();
        }
        "editor_add_element" => {
            let node = resolve(doc, element().unwrap());
            cpa::add_element(doc, node, text(op, "name"), text(op, "value")).unwrap();
        }
        "editor_delete_element" => {
            let node = resolve(doc, element().unwrap());
            assert!(cpa::delete_element(doc, node));
        }
        "editor_delete_attribute" => {
            let node = resolve(doc, element().unwrap());
            cpa::delete_attribute_by_name(doc, node, text(op, "name")).unwrap();
        }
        other => panic!("unknown operation {other}"),
    }
}

fn assert_same(name: &str, written: &str, expected: &str) {
    if written != expected {
        let position = written.chars().zip(expected.chars()).take_while(|(a, b)| a == b).count();
        let start = position.saturating_sub(100);
        panic!(
            "{name} differs at char {position}\n rust:   {:?}\n python: {:?}",
            written.chars().skip(start).take(220).collect::<String>(),
            expected.chars().skip(start).take(220).collect::<String>()
        );
    }
}

#[test]
fn operations_write_the_same_bytes_as_python() {
    let cases = json("cases.json");
    let cases = cases.as_array().unwrap();
    assert!(cases.len() >= 10, "only {} cases", cases.len());
    for case in cases {
        let name = text(case, "name");
        let mut doc = load_cpa(text(case, "source"));
        for op in case["ops"].as_array().unwrap() {
            apply(&mut doc, op);
        }
        let expected = fs::read_to_string(reference(text(case, "expected"))).unwrap();
        assert_same(name, &doc.to_xml(), &expected);
    }
}

#[test]
fn key_info_to_pem_matches_python() {
    let reference_json = json("key_info_full_test_cpa.json");
    let doc = load_cpa(text(&reference_json, "source"));
    let key_info = resolve(&doc, &reference_json["key_info"]);
    let expected = fs::read_to_string(reference("key_info_full_test_cpa.pem")).unwrap();
    assert_eq!(key_info_to_pem(&doc, Some(key_info)).unwrap(), expected);
}

#[test]
fn general_fields_and_party_lists_match_python() {
    let report = json("certificates.json");
    let mut checked = 0;
    for (name, cpa_report) in report["cpas"].as_object().unwrap() {
        let doc = load_cpa(name);
        if let Some(general) = cpa_report["general"].as_array() {
            let fields = cpa::read_general(&doc);
            let actual = [
                &fields.cpa_id,
                &fields.status,
                &fields.party_names[0],
                &fields.party_names[1],
                &fields.party_ids[0],
                &fields.party_ids[1],
                &fields.party_id_types[0],
                &fields.party_id_types[1],
                &fields.start_date,
                &fields.end_date,
            ];
            let expected: Vec<&str> = general.iter().map(|v| v.as_str().unwrap()).collect();
            assert_eq!(actual.to_vec(), expected, "{name}");
            checked += 1;
        }
        if let Some(parties) = cpa_report["parties"].as_array() {
            let party_infos = cpa::party_infos(&doc);
            for (party, expected) in party_infos.iter().zip(parties) {
                let transports: Vec<Vec<usize>> =
                    cpa::transports(&doc, *party).into_iter().map(|t| path_of(&doc, t)).collect();
                let expected_transports: Vec<Vec<usize>> = serde_json::from_value(expected["transports"].clone()).unwrap();
                assert_eq!(transports, expected_transports, "{name}");
                let cert_ids: Vec<String> = serde_json::from_value(expected["cert_ids"].clone()).unwrap();
                assert_eq!(cpa::cert_ids(&doc, *party), cert_ids, "{name}");
                let security_ids: Vec<String> = serde_json::from_value(expected["security_ids"].clone()).unwrap();
                assert_eq!(cpa::security_ids(&doc, *party), security_ids, "{name}");
            }
        }
    }
    assert!(checked >= 8, "only {checked} CPAs checked");
}
