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

/// A partner with more PartyIds: the first is edited on the General tab, the others are listed.
#[test]
fn other_party_ids_lists_the_party_ids_after_the_first() {
    let doc = minimal_cpa(
        "<tns:PartyInfo tns:partyName=\"PartnerA\">\
           <tns:PartyId tns:type=\"urn:osb:oin\">00000001234567890000</tns:PartyId>\
           <tns:PartyId tns:type=\"urn:example:kvk\"> 12345678 </tns:PartyId>\
           <tns:PartyId>NO_TYPE</tns:PartyId>\
         </tns:PartyInfo>\n  \
         <tns:PartyInfo tns:partyName=\"PartnerB\"><tns:PartyId tns:type=\"TypeB\">PartnerB_ID</tns:PartyId></tns:PartyInfo>",
    );
    assert_eq!(read_general(&doc).party_ids[0], "00000001234567890000");
    assert_eq!(
        cpa::other_party_ids(&doc, 0),
        vec![("12345678".to_string(), "urn:example:kvk".to_string()), ("NO_TYPE".to_string(), String::new())]
    );
    assert!(cpa::other_party_ids(&doc, 1).is_empty());
    assert!(cpa::other_party_ids(&doc, 2).is_empty());
}

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

#[test]
fn next_cpa_id_raises_the_version() {
    use cpa_core::cpa::next_cpa_id;
    assert_eq!(next_cpa_id("osb_ebms_123_456_1"), "osb_ebms_123_456_2");
    assert_eq!(next_cpa_id("CPA009"), "CPA010");
    assert_eq!(next_cpa_id("cpa"), "cpa_2");
    assert_eq!(next_cpa_id("42"), "42_2");
    assert_eq!(next_cpa_id(""), "");
}

#[test]
fn new_cpa_from_copies_with_new_id_dates_and_status() {
    use cpa_core::cpa::{check_new_cpa_options, new_cpa_defaults, new_cpa_from, read_general, GeneralField};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas/full_test_cpa.xml");
    let doc = cpa_core::xml::Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let before = doc.to_xml();
    let mut options = new_cpa_defaults(&doc);
    assert_ne!(options.cpa_id, read_general(&doc).cpa_id);
    assert!(check_new_cpa_options(&doc, &options).is_empty());

    options.strip_certificates = true;
    let (new, report) = new_cpa_from(&doc, &options).unwrap();
    assert_eq!(doc.to_xml(), before, "the loaded CPA stays as it was");
    let fields = read_general(&new);
    assert_eq!(fields.cpa_id, options.cpa_id);
    assert_eq!(fields.start_date, options.start_date);
    assert_eq!(fields.end_date, options.end_date);
    assert_eq!(fields.status, "proposed");
    assert_eq!(report.unwrap().cert_ids.len(), 4);
    assert!(cpa_core::validation::validate_document(&new).unwrap().is_empty());

    // The same CPAId and an end before the start are refused
    options.cpa_id = read_general(&doc).cpa_id;
    options.end_date = "2000-01-01T00:00:00Z".into();
    let fields: Vec<GeneralField> = check_new_cpa_options(&doc, &options).into_iter().map(|(f, _)| f).collect();
    assert_eq!(fields, vec![GeneralField::CpaId, GeneralField::EndDate]);
    assert!(new_cpa_from(&doc, &options).is_err());
}

#[test]
fn cpa_file_name_replaces_characters_a_file_name_cannot_hold() {
    assert_eq!(cpa_core::cpa::cpa_file_name("a/b:c*d"), "a_b_c_d.xml");
}

fn test_cpa(name: &str) -> cpa_core::xml::Document {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name);
    cpa_core::xml::Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn id_code_is_the_party_id_or_the_common_start_of_the_ids() {
    use cpa_core::cpa::id_code;
    let doc = test_cpa("full_test_cpa.xml");
    assert_eq!(id_code(&doc, 0).as_deref(), Some("BROKER"));
    assert_eq!(id_code(&doc, 1).as_deref(), Some("Ketenpartner"));
    let doc = test_cpa("example.xml");
    assert_eq!(id_code(&doc, 0).as_deref(), Some("DUMMYBROKER_O"));
    assert_eq!(id_code(&doc, 1).as_deref(), Some("DUMMYPARTNER_O"));
    let doc = test_cpa("full_cpa_p.xml");
    assert_eq!(id_code(&doc, 0), None);
}

#[test]
fn replace_codes_replaces_whole_words_in_one_pass() {
    use cpa_core::cpa::replace_codes;
    let swap = vec![("A".to_string(), "B".to_string()), ("B".to_string(), "A".to_string())];
    assert_eq!(replace_codes("A_S1_B_X_AB", &swap), "B_S1_A_X_AB");
    let renames = vec![("BROKER".to_string(), "OIN1".to_string())];
    assert_eq!(replace_codes("BROKER_S1_BrokerX_BROKERS", &renames), "OIN1_S1_BrokerX_BROKERS");
}

#[test]
fn new_cpa_renames_the_ids_with_the_party_ids() {
    use cpa_core::cpa::{check_new_cpa_options, id_renames, new_cpa_defaults, new_cpa_from, read_general};
    let doc = test_cpa("full_test_cpa.xml");
    let mut options = new_cpa_defaults(&doc);
    assert_eq!(options.party_ids, vec!["PARTNER_A".to_string(), "KETENPARTNER_1".to_string()]);
    options.party_ids = vec!["OIN_A".into(), "OIN_B".into()];
    let renames = id_renames(&doc, &options.party_ids).unwrap();
    assert!(renames.contains(&("BROKER_T_HTTPS".into(), "OIN_A_T_HTTPS".into())));
    assert!(renames.contains(&(
        "BROKER_S1_BrokerX_Ketenpartner_S_ProcessAntwoord_PV_0.1".into(),
        "OIN_A_S1_BrokerX_OIN_B_S_ProcessAntwoord_PV_0.1".into()
    )));
    let errors_before = cpa_core::validation::validate_document(&doc).unwrap().len();
    let (new, _) = new_cpa_from(&doc, &options).unwrap();
    let xml = new.to_xml();
    let left: Vec<&str> = xml.lines().filter(|l| l.contains("BROKER_") || l.contains("Ketenpartner_")).collect();
    assert!(left.is_empty(), "every code is renamed: {left:#?}");
    assert!(xml.contains("tns:transportId=\"OIN_A_T_HTTPS\""));
    assert!(xml.contains("tns:defaultMshChannelId=\"OIN_B_DC_T_Besteffort_S_Transport_R_\""));
    let fields = read_general(&new);
    assert_eq!(fields.party_ids, ["OIN_A".to_string(), "OIN_B".to_string()]);
    // The same (dangling) reference as before, under its new name: references and ids stay consistent
    assert_eq!(cpa_core::validation::validate_document(&new).unwrap().len(), errors_before);

    // An id cannot start with a digit (an OIN as PartyId)
    options.party_ids = vec!["00000001234567890000".into(), "OIN_B".into()];
    let errors = check_new_cpa_options(&doc, &options);
    assert!(errors.iter().any(|(_, e)| e.contains("not a valid id")), "{errors:?}");
    options.rename_ids = false;
    assert!(check_new_cpa_options(&doc, &options).is_empty());
}

#[test]
fn new_cpa_id_follows_the_party_ids() {
    let doc = test_cpa("example.xml");
    let options = cpa_core::cpa::new_cpa_defaults(&doc);
    // The code DUMMYBROKER_O in the ids becomes the PartyId DUMMYBRONT1, also in the proposed CPA ID
    assert_eq!(options.cpa_id, "DUMMYBRONT1-DUMMYPARTNER_O_V002");
}
