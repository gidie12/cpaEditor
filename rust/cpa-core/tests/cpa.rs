//! The logic of TestGeneralTab, TestCommentTab, TestSetEndDateFromCertificates (start date), the transport tab and
//! the XML editor, ported to Rust, plus the cases the Python version handled in its own way.

use std::fs;
use std::path::Path;

use chrono::{NaiveDateTime, Utc};
use cpa_core::cpa::{
    self, add_comment, add_element, clone_element, comments, delete_attribute_by_name, delete_comment, delete_element,
    read_general, set_attribute_by_name, set_comment, set_general, GeneralField,
    CPA_STATUS_OPTIONS, XML_INCOMPATIBLE_MESSAGE,
};
use cpa_core::validation::validate_document;
use cpa_core::xml::{Document, NodeId};
use cpa_core::{CPA_DATE_FORMAT, CPA_NAMESPACE};

fn load_cpa(name: &str) -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name);
    Document::parse_bytes(&fs::read(path).unwrap()).unwrap()
}

fn minimal_cpa(party_infos: &str) -> Document {
    Document::parse(&format!(
        "<tns:CollaborationProtocolAgreement xmlns:tns=\"{CPA_NAMESPACE}\" tns:cpaid=\"CPA123\">\n  \
         <tns:Status tns:value=\"proposed\"/>\n  <tns:Start>2023-01-01</tns:Start>\n  <tns:End>2023-12-31</tns:End>\n  \
         {party_infos}\n</tns:CollaborationProtocolAgreement>"
    ))
    .unwrap()
}

const PARTIES: &str = "<tns:PartyInfo tns:partyName=\"PartnerA\"><tns:PartyId tns:type=\"TypeA\">PartnerA_ID</tns:PartyId></tns:PartyInfo>\n  \
                       <tns:PartyInfo tns:partyName=\"PartnerB\"><tns:PartyId tns:type=\"TypeB\">PartnerB_ID</tns:PartyId></tns:PartyInfo>";

fn element_text(doc: &Document, local: &str) -> String {
    doc.text(doc.find(doc.root().unwrap(), Some(CPA_NAMESPACE), local).unwrap()).unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------------------------
// TestGeneralTab
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn creates_all_fields_correctly() {
    let fields = read_general(&minimal_cpa(PARTIES));
    assert_eq!(fields.cpa_id, "CPA123");
    assert_eq!(fields.status, "proposed");
    assert_eq!(fields.party_ids, ["PartnerA_ID", "PartnerB_ID"]);
    assert_eq!(fields.party_names, ["PartnerA", "PartnerB"]);
    assert_eq!(fields.start_date, "2023-01-01");
    assert_eq!(fields.end_date, "2023-12-31");
    assert_eq!(fields.party_id_types, ["TypeA", "TypeB"]);
}

#[test]
fn handles_empty_fields_gracefully() {
    let doc = Document::parse(&format!("<tns:CollaborationProtocolAgreement xmlns:tns=\"{CPA_NAMESPACE}\"/>")).unwrap();
    assert_eq!(read_general(&doc), cpa::GeneralFields::default());
}

#[test]
fn party_without_party_id_empties_both_partners() {
    // CPAParser.get_partners stops at the first missing value and only shows pairs
    let doc = minimal_cpa(
        "<tns:PartyInfo tns:partyName=\"A\"><tns:PartyId tns:type=\"T\">A_ID</tns:PartyId></tns:PartyInfo>\
         <tns:PartyInfo tns:partyName=\"B\"/>",
    );
    let fields = read_general(&doc);
    assert_eq!(fields.party_ids, ["", ""]);
    assert_eq!(fields.party_names, ["", ""]);
    assert_eq!(fields.party_id_types, ["", ""]);
    assert_eq!(fields.cpa_id, "CPA123");
}

#[test]
fn status_options_match_schema() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemaValidation/cpp-cpa-2_0.xsd");
    let schema = Document::parse_bytes(&fs::read(path).unwrap()).unwrap();
    let xs = Some("http://www.w3.org/2001/XMLSchema");
    let root = schema.root().unwrap();
    let status_type = schema
        .iter_named(root, xs, "simpleType")
        .find(|&node| schema.attribute(node, None, "name") == Some("statusValue.type"))
        .unwrap();
    let mut allowed: Vec<&str> = schema
        .iter_named(status_type, xs, "enumeration")
        .map(|node| schema.attribute(node, None, "value").unwrap())
        .collect();
    allowed.sort();
    let mut options = CPA_STATUS_OPTIONS.to_vec();
    options.sort();
    assert_eq!(options, allowed);
}

#[test]
fn set_general_changes_every_field() {
    let mut doc = minimal_cpa(PARTIES);
    let changes = [
        (GeneralField::CpaId, "new_id"),
        (GeneralField::Status, "signed"),
        (GeneralField::PartyName(0), "Café Ünïcode"),
        (GeneralField::PartyName(1), "B"),
        (GeneralField::PartyId(0), "1"),
        (GeneralField::PartyId(1), "2"),
        (GeneralField::PartyIdType(0), "t1"),
        (GeneralField::PartyIdType(1), "t2"),
        (GeneralField::StartDate, "2026-01-01T00:00:00Z"),
        (GeneralField::EndDate, "2027-01-01T00:00:00Z"),
    ];
    for (field, value) in changes {
        set_general(&mut doc, field, value).unwrap();
    }
    let fields = read_general(&doc);
    assert_eq!(fields.cpa_id, "new_id");
    assert_eq!(fields.status, "signed");
    assert_eq!(fields.party_names, ["Café Ünïcode", "B"]);
    assert_eq!(fields.party_ids, ["1", "2"]);
    assert_eq!(fields.party_id_types, ["t1", "t2"]);
    assert_eq!((fields.start_date.as_str(), fields.end_date.as_str()), ("2026-01-01T00:00:00Z", "2027-01-01T00:00:00Z"));
}

#[test]
fn set_general_errors_like_python() {
    // CPACreator needs two PartyInfo elements of the CPA namespace; full_cpa.xml uses the old ebXML namespace
    let mut doc = load_cpa("full_cpa.xml");
    assert_eq!(set_general(&mut doc, GeneralField::CpaId, "x").unwrap_err(), "list index out of range");

    let mut doc = Document::parse(&format!(
        "<tns:CollaborationProtocolAgreement xmlns:tns=\"{CPA_NAMESPACE}\"><tns:PartyInfo/><tns:PartyInfo/>\
         </tns:CollaborationProtocolAgreement>"
    ))
    .unwrap();
    assert_eq!(
        set_general(&mut doc, GeneralField::Status, "agreed").unwrap_err(),
        "'NoneType' object has no attribute 'attrib'"
    );
    assert_eq!(
        set_general(&mut doc, GeneralField::StartDate, "x").unwrap_err(),
        "'NoneType' object has no attribute 'text'"
    );
    assert!(set_general(&mut doc, GeneralField::PartyName(2), "x").is_err());
    assert_eq!(set_general(&mut doc, GeneralField::CpaId, "a\u{1}").unwrap_err(), XML_INCOMPATIBLE_MESSAGE);
    set_general(&mut doc, GeneralField::PartyName(1), "ok").unwrap();
}

// ---------------------------------------------------------------------------------------------------------------
// TestSetEndDateFromCertificates: the start date
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn set_start_date_to_now() {
    let mut doc = minimal_cpa(PARTIES);
    let before = Utc::now().timestamp();
    let start_date = cpa::set_start_date_to_now(&mut doc).unwrap();
    let after = Utc::now().timestamp();
    assert_eq!(element_text(&doc, "Start"), start_date);
    let parsed = NaiveDateTime::parse_from_str(&start_date, CPA_DATE_FORMAT).unwrap().and_utc().timestamp();
    assert!(before <= parsed && parsed <= after);
    assert_eq!(element_text(&doc, "End"), "2023-12-31");
}

// ---------------------------------------------------------------------------------------------------------------
// TestCommentTab (the Python methods lacked the test_ prefix and never ran)
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn loads_comments_correctly() {
    let doc = load_cpa("full_cpa_p.xml");
    let texts: Vec<String> = comments(&doc).iter().map(|&c| doc.text(c).unwrap()).collect();
    assert_eq!(texts.len(), 3);
    assert_eq!(texts[2], "Nederland");
}

#[test]
fn adds_comment_correctly() {
    let mut doc = load_cpa("full_cpa_p_without_comments.xml");
    let comment = add_comment(&mut doc, "New Comment", "nl-NL");
    assert_eq!(comments(&doc), [comment]);
    assert_eq!(doc.text(comment).unwrap(), "New Comment");
    assert_eq!(doc.attribute(comment, Some(cpa_core::xml::XML_NAMESPACE), "lang"), Some("nl-NL"));
    assert_eq!(doc.child_elements(doc.root().unwrap()).last(), Some(comment));
}

#[test]
fn added_comment_is_schema_valid() {
    for name in ["full_cpa_p_without_comments.xml", "full_cpa_p.xml", "test_cpa.xml"] {
        let mut doc = load_cpa(name);
        let before = validate_document(&doc).unwrap();
        add_comment(&mut doc, "Schema check", "en-US");
        let after = validate_document(&doc).unwrap();
        let messages = |errors: &[cpa_core::validation::ValidationError]| -> Vec<String> {
            errors.iter().map(|e| e.message.clone()).collect()
        };
        assert_eq!(messages(&after), messages(&before), "{name}: adding a comment must not add schema errors");
    }
}

#[test]
fn updates_comment_correctly() {
    let mut doc = load_cpa("full_cpa_p_without_comments.xml");
    let comment = add_comment(&mut doc, "Initial Comment", "nl-NL");
    set_comment(&mut doc, comment, "Updated Comment").unwrap();
    assert_eq!(doc.text(comments(&doc)[0]).unwrap(), "Updated Comment");
}

#[test]
fn deletes_comment_correctly() {
    let mut doc = load_cpa("full_cpa_p_without_comments.xml");
    let original = doc.to_xml();
    let comment = add_comment(&mut doc, "Comment to Delete", "nl-NL");
    delete_comment(&mut doc, comment).unwrap();
    assert!(comments(&doc).is_empty());
    assert_eq!(doc.to_xml(), original);
}

#[test]
fn delete_comment_of_other_parent_fails() {
    let mut doc = minimal_cpa(PARTIES);
    let status = doc.find(doc.root().unwrap(), Some(CPA_NAMESPACE), "Status").unwrap();
    let nested = doc.append_element(status, Some(CPA_NAMESPACE), "Comment");
    assert!(delete_comment(&mut doc, nested).is_err());
}

// ---------------------------------------------------------------------------------------------------------------
// Transport tab
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn transports_cert_ids_and_security_ids_per_party() {
    let doc = load_cpa("full_test_cpa.xml");
    let parties = cpa::party_infos(&doc);
    assert_eq!(parties.len(), 2);
    for party in parties {
        let transports = cpa::transports(&doc, party);
        assert!(!transports.is_empty());
        assert!(transports.iter().all(|&t| doc.local_name(t) == "Transport"));
        assert_eq!(cpa::cert_ids(&doc, party).len(), 2);
        assert!(!cpa::security_ids(&doc, party).is_empty());
    }
}

#[test]
fn save_changes_sets_cpa_attribute_and_text() {
    let mut doc = load_cpa("full_test_cpa.xml");
    let party = cpa::party_infos(&doc)[0];
    let transport = cpa::transports(&doc, party)[0];
    cpa::set_cpa_attribute(&mut doc, transport, "transportId", "T1").unwrap();
    assert_eq!(doc.attribute(transport, Some(CPA_NAMESPACE), "transportId"), Some("T1"));
    let protocol = doc.iter_named(transport, Some(CPA_NAMESPACE), "TransportProtocol").next().unwrap();
    cpa::set_element_text(&mut doc, protocol, "HTTPS").unwrap();
    assert_eq!(doc.text(protocol).unwrap(), "HTTPS");
}

#[test]
fn clone_element_copies_after_tail() {
    let mut doc = Document::parse("<a>\n  <b x=\"1\"><c/></b>\n  <d/>\n</a>").unwrap();
    let root = doc.root().unwrap();
    let b = doc.find(root, None, "b").unwrap();
    let copy = clone_element(&mut doc, b).unwrap();
    assert_ne!(copy, b);
    assert_eq!(
        doc.to_xml(),
        "<?xml version='1.0' encoding='UTF-8'?>\n<a>\n  <b x=\"1\"><c/></b>\n  <b x=\"1\"><c/></b>\n  <d/>\n</a>"
    );
    let d = doc.find(root, None, "d").unwrap();
    clone_element(&mut doc, d).unwrap();
    assert!(doc.to_xml().ends_with("<d/>\n<d/>\n</a>"));
    assert_eq!(clone_element(&mut doc, root), None);
}

// ---------------------------------------------------------------------------------------------------------------
// XML editor
// ---------------------------------------------------------------------------------------------------------------

fn status(doc: &Document) -> NodeId {
    doc.find(doc.root().unwrap(), Some(CPA_NAMESPACE), "Status").unwrap()
}

#[test]
fn xml_editor_changes_attributes_by_clark_name() {
    let mut doc = minimal_cpa(PARTIES);
    let root = doc.root().unwrap();
    set_attribute_by_name(&mut doc, root, &format!("{{{CPA_NAMESPACE}}}cpaid"), "edited").unwrap();
    assert_eq!(read_general(&doc).cpa_id, "edited");
    set_attribute_by_name(&mut doc, root, "plain", "v").unwrap();
    assert_eq!(doc.attribute(root, None, "plain"), Some("v"));
    delete_attribute_by_name(&mut doc, root, "plain").unwrap();
    assert_eq!(delete_attribute_by_name(&mut doc, root, "nope").unwrap_err(), "'nope'");
    assert_eq!(set_attribute_by_name(&mut doc, root, "a:b", "v").unwrap_err(), "Invalid attribute name 'a:b'");
    assert_eq!(set_attribute_by_name(&mut doc, root, "with space", "v").unwrap_err(), "Invalid attribute name 'with space'");
    assert_eq!(set_attribute_by_name(&mut doc, root, "", "v").unwrap_err(), "Empty tag name");
    assert_eq!(set_attribute_by_name(&mut doc, root, "ok", "\u{b}").unwrap_err(), XML_INCOMPATIBLE_MESSAGE);
}

#[test]
fn xml_editor_adds_elements_after_same_name() {
    let mut doc = minimal_cpa(PARTIES);
    let root = doc.root().unwrap();
    let start = add_element(&mut doc, root, "Start", "again").unwrap();
    let siblings: Vec<String> = doc.child_elements(root).map(|c| doc.local_name(c).to_string()).collect();
    assert_eq!(siblings, ["Status", "Start", "Start", "End", "PartyInfo", "PartyInfo"]);
    assert_eq!(doc.text(start).unwrap(), "again");
    let status_element = status(&doc);
    let note = add_element(&mut doc, status_element, "Note", "").unwrap();
    assert_eq!(doc.element_to_xml(note), format!("<tns:Note xmlns:tns=\"{CPA_NAMESPACE}\"></tns:Note>"));
    assert_eq!(add_element(&mut doc, root, "", "v").unwrap_err(), "Empty tag name");
    assert_eq!(add_element(&mut doc, root, "with space", "v").unwrap_err(), "Invalid tag name 'with space'");
    assert_eq!(add_element(&mut doc, root, "1a", "v").unwrap_err(), "Invalid tag name '1a'");
}

#[test]
fn xml_editor_deletes_elements_but_not_the_root() {
    let mut doc = minimal_cpa(PARTIES);
    let root = doc.root().unwrap();
    assert!(!delete_element(&mut doc, root));
    let status = status(&doc);
    assert!(delete_element(&mut doc, status));
    assert!(!doc.is_attached(status));
    assert!(doc.to_xml().contains("CPA123\">\n  <tns:Start>"));
}
