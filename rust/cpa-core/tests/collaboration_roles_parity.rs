//! Differential test against the Python version: every case replays the operations of
//! tools/reference_collaboration_roles.py on the same test CPA and must save byte for byte what lxml saved
//! (tests/reference/collaboration_roles/<case>.xml). Keep the cases in step with the script.

use std::fs;
use std::path::{Path, PathBuf};

use cpa_core::collaboration_roles::*;
use cpa_core::xml::{Document, NodeId};
use cpa_core::CPA_NAMESPACE;

const NS: Option<&str> = Some(CPA_NAMESPACE);

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load(source: &str) -> Document {
    let path = manifest().join("../../application/tests/resources/cpas").join(source);
    Document::parse_bytes(&fs::read(path).unwrap()).unwrap()
}

/// Compares with the reference file and shows the first differing lines.
fn check(case: &str, document: &Document) {
    let reference = manifest().join("tests/reference/collaboration_roles").join(format!("{case}.xml"));
    let expected = fs::read_to_string(&reference).unwrap_or_else(|e| panic!("{}: {e}", reference.display()));
    let written = document.to_xml();
    if written == expected {
        return;
    }
    let written_lines: Vec<&str> = written.split('\n').collect();
    let expected_lines: Vec<&str> = expected.split('\n').collect();
    let first = written_lines.iter().zip(&expected_lines).take_while(|(a, b)| a == b).count();
    let mut report = format!("{case}: differs from lxml from line {}\n", first + 1);
    let start = first.saturating_sub(2);
    for (label, lines) in [("rust", &written_lines), ("lxml", &expected_lines)] {
        report.push_str(&format!("--- {label}\n"));
        for (number, line) in lines.iter().enumerate().skip(start).take(8) {
            report.push_str(&format!("{:>5} | {line:?}\n", number + 1));
        }
    }
    panic!("{report}");
}

fn roles(document: &Document, party_index: usize) -> Vec<NodeId> {
    document.find_all(party_infos(document)[party_index], NS, "CollaborationRole").collect()
}

fn root(document: &Document) -> NodeId {
    document.root().unwrap()
}

fn action(document: &Document, binding_id: &str) -> NodeId {
    find_action_by_binding_id(document, root(document), binding_id).unwrap()
}

fn chars(pairs: &[(&str, &str)]) -> Characteristics {
    pairs.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

fn new_role(values: [&str; 8]) -> NewCollaborationRole {
    NewCollaborationRole {
        process_name: values[0].into(),
        process_version: values[1].into(),
        process_href: values[2].into(),
        process_uuid: values[3].into(),
        role_name: values[4].into(),
        role_href: values[5].into(),
        service_name: values[6].into(),
        service_type: values[7].into(),
    }
}

/// Adds an action with the suggested id, channel and characteristics, like the GUI does (add_suggested).
fn add_suggested(
    document: &mut Document,
    role: NodeId,
    direction: Direction,
    action_name: &str,
    mirror_role: Option<NodeId>,
) {
    let binding_id = suggest_binding_id(document, role, direction, action_name, &[]);
    let mirror = mirror_role.map(|mirror_role| {
        let opposite = direction.opposite();
        Mirror {
            collaboration_role: mirror_role,
            binding_id: suggest_binding_id(
                document,
                mirror_role,
                opposite,
                action_name,
                std::slice::from_ref(&binding_id),
            ),
            channel_id: suggest_channel_id(document, mirror_role, opposite),
        }
    });
    let packaging = document.find(root(document), NS, "Packaging").unwrap();
    let package_id = document.attribute(packaging, NS, "id").unwrap().to_string();
    let channel_id = suggest_channel_id(document, role, direction);
    let characteristics = suggest_characteristics(document, role);
    add_action(
        document,
        role,
        direction,
        action_name,
        &package_id,
        &channel_id,
        &characteristics,
        &binding_id,
        mirror.as_ref(),
    )
    .unwrap();
}

#[test]
fn add_send_with_mirror() {
    let mut document = load("full_test_cpa.xml");
    let role = roles(&document, 0)[0];
    let counterpart = find_counterpart_collaboration_role(&document, role);
    assert!(counterpart.is_some());
    add_suggested(&mut document, role, Direction::Send, "NewAction", counterpart);
    check("add_send_with_mirror", &document);
}

#[test]
fn add_receive_with_mirror_tab_indent() {
    let mut document = load("test_cpa.xml");
    let role = roles(&document, 0)[1];
    let counterpart = find_counterpart_collaboration_role(&document, role);
    assert!(counterpart.is_some());
    add_suggested(&mut document, role, Direction::Receive, "NewAction", counterpart);
    check("add_receive_with_mirror_tab_indent", &document);
}

#[test]
fn add_receive_after_existing() {
    let mut document = load("full_cpa_p.xml");
    let role = roles(&document, 1)[0];
    add_suggested(&mut document, role, Direction::Receive, "Extra Receive", None);
    check("add_receive_after_existing", &document);
}

#[test]
fn add_receive_first_in_service_binding() {
    let mut document = load("full_cpa_p.xml");
    let party_info = party_infos(&document)[0];
    let role = add_collaboration_role(
        &mut document,
        party_info,
        &new_role(["Process", "1.0", "http://example.com/spec", "", "Role", "http://example.com/spec", "Empty", ""]),
    )
    .unwrap();
    let channel_id = suggest_channel_id(&document, role, Direction::Receive);
    add_action(
        &mut document,
        role,
        Direction::Receive,
        "First",
        "defaultPackage_Profile",
        &channel_id,
        &chars(&[("isAuthenticated", "none"), ("timeToPerform", "PT5M")]),
        "first_receive",
        None,
    )
    .unwrap();
    check("add_receive_first_in_service_binding", &document);
}

#[test]
fn add_send_after_comment() {
    let mut document = load("full_test_cpa.xml");
    let role = roles(&document, 0)[0];
    let binding = document.find(role, NS, "ServiceBinding").unwrap();
    // lxml service_binding.insert(1, Comment(' note ')): directly before the second child, after the tail of the
    // first
    let comment = document.create_comment(" note ");
    let second = document.child_elements(binding).nth(1).unwrap();
    document.insert_before(second, comment);
    add_suggested(&mut document, role, Direction::Send, "AfterComment", None);
    check("add_send_after_comment", &document);
}

#[test]
fn update_characteristics() {
    let mut document = load("full_cpa_p.xml");
    let element = action(&document, "dummybroker-ontvangen");
    update_action(
        &mut document,
        element,
        " Renamed ",
        "defaultPackage_Profile",
        "DC_T_Besteffort_S__R_PARTNERA",
        &chars(&[
            ("isAuthenticated", "persistent"),
            ("timeToPerform", ""),
            ("isTamperProof", " none "),
            ("custom", "x"),
        ]),
    )
    .unwrap();
    check("update_characteristics", &document);
}

#[test]
fn update_without_characteristics_and_channel() {
    let mut document = load("full_cpa_p.xml");
    let element = action(&document, "dummybroker-verzonden");
    let binding = document.find(element, NS, "ThisPartyActionBinding").unwrap();
    for name in ["BusinessTransactionCharacteristics", "ChannelId"] {
        let child = document.find(binding, NS, name).unwrap();
        document.remove_with_tail(child);
    }
    update_action(
        &mut document,
        element,
        "Verzonden",
        "defaultPackage_Profile",
        "new_channel",
        &chars(&[("isConfidential", "none")]),
    )
    .unwrap();
    check("update_without_characteristics_and_channel", &document);
}

#[test]
fn delete_action_only() {
    let mut document = load("full_test_cpa.xml");
    let element = action(&document, "BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1");
    assert_eq!(delete_action(&mut document, element, false).len(), 1);
    check("delete_action_only", &document);
}

#[test]
fn delete_action_with_counterpart() {
    let mut document = load("full_test_cpa.xml");
    let element = action(&document, "BROKER_S1_Ketenpartner_BrokerX_R_ProcessVerzoek_PV_0.1");
    assert_eq!(
        delete_action(&mut document, element, true),
        [
            "BROKER_S1_Ketenpartner_BrokerX_R_ProcessVerzoek_PV_0.1",
            "Ketenpartner_S1_Ketenpartner_BrokerX_S_ProcessVerzoek_PV_0.1"
        ]
    );
    check("delete_action_with_counterpart", &document);
}

#[test]
fn delete_last_and_only_child() {
    let mut document = load("full_test_cpa.xml");
    let role = roles(&document, 0)[0];
    for binding_id in [
        "BROKER_S1_Ketenpartner_BrokerX_R_ProcessVerzoek_PV_0.1",
        "BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1",
    ] {
        let element = action(&document, binding_id);
        delete_action(&mut document, element, false);
    }
    let binding = document.find(role, NS, "ServiceBinding").unwrap();
    let service = document.find(binding, NS, "Service").unwrap();
    remove_element(&mut document, service);
    check("delete_last_and_only_child", &document);
}

#[test]
fn add_collaboration_role_with_uuid() {
    let mut document = load("full_cpa_p.xml");
    let party_info = party_infos(&document)[0];
    add_collaboration_role(
        &mut document,
        party_info,
        &new_role([
            " NewProcess ",
            "2.0",
            "http://example.com/process",
            "urn:uuid:1234",
            "NewRole",
            "http://example.com/role",
            "NewService",
            "urn:example:type",
        ]),
    )
    .unwrap();
    check("add_collaboration_role_with_uuid", &document);
}

#[test]
fn add_collaboration_role_mixed_indent() {
    let mut document = load("test_cpa.xml");
    let party_info = party_infos(&document)[0];
    add_collaboration_role(&mut document, party_info, &new_role(["P", "1", "href", "", "R", "href", "S", ""])).unwrap();
    check("add_collaboration_role_mixed_indent", &document);
}

#[test]
fn delete_collaboration_role_second() {
    let mut document = load("full_test_cpa.xml");
    let role = roles(&document, 0)[1];
    delete_collaboration_role(&mut document, role).unwrap();
    check("delete_collaboration_role_second", &document);
}

#[test]
fn delete_collaboration_role_first() {
    let mut document = load("full_test_cpa.xml");
    let role = roles(&document, 1)[0];
    delete_collaboration_role(&mut document, role).unwrap();
    check("delete_collaboration_role_first", &document);
}

#[test]
fn every_reference_file_has_a_case() {
    let source = fs::read_to_string(manifest().join("tests/collaboration_roles_parity.rs")).unwrap();
    let folder = manifest().join("tests/reference/collaboration_roles");
    let mut count = 0;
    for entry in fs::read_dir(&folder).unwrap() {
        let path = entry.unwrap().path();
        let case = Path::new(&path).file_stem().unwrap().to_str().unwrap().to_string();
        assert!(source.contains(&format!("check(\"{case}\"")), "no Rust case for {case}");
        count += 1;
    }
    assert!(count >= 14, "only {count} reference files");
}
