//! Port of application/tests/TestCollaborationRoles.py.

use std::path::Path;

use cpa_core::collaboration_roles::*;
use cpa_core::validation::validate_document;
use cpa_core::xml::{Document, NodeId, NodeKind};
use cpa_core::CPA_NAMESPACE;

const NS: Option<&str> = Some(CPA_NAMESPACE);

const CHARACTERISTICS_ATTRIBUTES: &str = "tns:isAuthenticated=\"persistent\" tns:isAuthorizationRequired=\"true\" \
                                          tns:isConfidential=\"transient\" tns:timeToPerform=\"PT2H\"";

fn cpa_text() -> String {
    format!(
        r#"<tns:CollaborationProtocolAgreement xmlns:tns="{CPA_NAMESPACE}" xmlns:xlink="http://www.w3.org/1999/xlink">
    <tns:PartyInfo tns:partyName="A" tns:defaultMshChannelId="A_default">
        <tns:PartyId>A_ID</tns:PartyId>
        <tns:PartyRef xlink:href="https://a.example.com"/>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Sender" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Order</tns:Service>
                <tns:CanSend>
                    <tns:ThisPartyActionBinding tns:id="A_S1_S_Place_PV_1" tns:action="Place" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics {c}/>
                        <tns:ChannelId>A_reliable</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>
                        B_S1_R_Place_PV_1
                    </tns:OtherPartyActionBinding>
                </tns:CanSend>
                <tns:CanReceive>
                    <tns:ThisPartyActionBinding tns:id="A_S1_R_Confirm_PV_1" tns:action="Confirm" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics {c}/>
                        <tns:ChannelId>A_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>B_S1_S_Confirm_PV_1</tns:OtherPartyActionBinding>
                </tns:CanReceive>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Sender" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Invoice</tns:Service>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:DeliveryChannel tns:channelId="A_default"/>
        <tns:DeliveryChannel tns:channelId="A_reliable"/>
    </tns:PartyInfo>
    <tns:PartyInfo tns:partyName="B">
        <tns:PartyId>B_ID</tns:PartyId>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Receiver" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">OrderService</tns:Service>
                <tns:CanSend>
                    <tns:ThisPartyActionBinding tns:id="B_S1_S_Confirm_PV_1" tns:action="Confirm" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics/>
                        <tns:ChannelId>B_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>A_S1_R_Confirm_PV_1</tns:OtherPartyActionBinding>
                </tns:CanSend>
                <tns:CanReceive>
                    <tns:ThisPartyActionBinding tns:id="B_S1_R_Place_PV_1" tns:action="Place" tns:packageId="pkg">
                        <tns:BusinessTransactionCharacteristics/>
                        <tns:ChannelId>B_default</tns:ChannelId>
                    </tns:ThisPartyActionBinding>
                    <tns:OtherPartyActionBinding>A_S1_S_Place_PV_1</tns:OtherPartyActionBinding>
                </tns:CanReceive>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:CollaborationRole>
            <tns:ProcessSpecification tns:name="Process" tns:version="1.0" xlink:href="spec"/>
            <tns:Role tns:name="Receiver" xlink:href="spec"/>
            <tns:ServiceBinding>
                <tns:Service tns:type="urn:test">Invoice</tns:Service>
            </tns:ServiceBinding>
        </tns:CollaborationRole>
        <tns:DeliveryChannel tns:channelId="B_default"/>
    </tns:PartyInfo>
    <tns:Packaging tns:id="pkg"/>
    <tns:Packaging tns:id="other_pkg"/>
</tns:CollaborationProtocolAgreement>"#,
        c = CHARACTERISTICS_ATTRIBUTES
    )
}

struct Fixture {
    doc: Document,
    party_a: NodeId,
    party_b: NodeId,
    order_a: NodeId,
    invoice_a: NodeId,
    order_b: NodeId,
    invoice_b: NodeId,
}

impl Fixture {
    fn new() -> Fixture {
        let doc = Document::parse(&cpa_text()).unwrap();
        let parties = party_infos(&doc);
        let (party_a, party_b) = (parties[0], parties[1]);
        let roles_a: Vec<NodeId> = doc.find_all(party_a, NS, "CollaborationRole").collect();
        let roles_b: Vec<NodeId> = doc.find_all(party_b, NS, "CollaborationRole").collect();
        Fixture {
            party_a,
            party_b,
            order_a: roles_a[0],
            invoice_a: roles_a[1],
            order_b: roles_b[0],
            invoice_b: roles_b[1],
            doc,
        }
    }

    fn action(&self, binding_id: &str) -> NodeId {
        find_action_by_binding_id(&self.doc, self.doc.root().unwrap(), binding_id).unwrap()
    }

    fn binding_ids(&self, element: NodeId) -> Vec<String> {
        binding_ids(&self.doc, element)
    }
}

fn binding_ids(doc: &Document, element: NodeId) -> Vec<String> {
    doc.iter_named(element, NS, "ThisPartyActionBinding")
        .map(|b| doc.attribute(b, NS, "id").unwrap_or("").to_string())
        .collect()
}

fn chars(pairs: &[(&str, &str)]) -> Characteristics {
    pairs.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

fn ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}

fn mirror(role: NodeId, binding_id: &str, channel_id: &str) -> Mirror {
    Mirror { collaboration_role: role, binding_id: binding_id.into(), channel_id: channel_id.into() }
}

/// The child nodes as lxml counts them (no text).
fn lxml_len(doc: &Document, parent: NodeId) -> usize {
    doc.children(parent).iter().filter(|&&c| !matches!(doc.kind(c), NodeKind::Text(_))).count()
}

fn local_names(doc: &Document, parent: NodeId) -> Vec<String> {
    doc.child_elements(parent).map(|c| doc.local_name(c).to_string()).collect()
}

#[allow(clippy::too_many_arguments)]
fn new_role(
    process_name: &str,
    process_version: &str,
    process_href: &str,
    process_uuid: &str,
    role_name: &str,
    role_href: &str,
    service_name: &str,
    service_type: &str,
) -> NewCollaborationRole {
    NewCollaborationRole {
        process_name: process_name.into(),
        process_version: process_version.into(),
        process_href: process_href.into(),
        process_uuid: process_uuid.into(),
        role_name: role_name.into(),
        role_href: role_href.into(),
        service_name: service_name.into(),
        service_type: service_type.into(),
    }
}

fn service_of(doc: &Document, role: NodeId) -> NodeId {
    let binding = doc.find(role, NS, "ServiceBinding").unwrap();
    doc.find(binding, NS, "Service").unwrap()
}

// ------------------------------------------------------------------------------------------------------------------
// TestDescribe
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_describe_collaboration_role() {
    let f = Fixture::new();
    assert_eq!(
        describe_collaboration_role(&f.doc, f.order_a),
        RoleDescription {
            process_name: "Process".into(),
            process_version: "1.0".into(),
            role_name: "Sender".into(),
            service_name: "Order".into(),
            service_type: "urn:test".into(),
        }
    );
}

#[test]
fn test_describe_action() {
    let f = Fixture::new();
    let action = describe_action(&f.doc, f.action("A_S1_S_Place_PV_1"));
    assert_eq!(action.direction, Direction::Send);
    assert_eq!(action.action, "Place");
    assert_eq!(action.package_id, "pkg");
    assert_eq!(action.channel_ids, ids(&["A_reliable"]));
    assert_eq!(action.other_party_binding, "B_S1_R_Place_PV_1");
    assert_eq!(
        action.characteristics,
        chars(&[
            ("isAuthenticated", "persistent"),
            ("isAuthorizationRequired", "true"),
            ("isConfidential", "transient"),
            ("timeToPerform", "PT2H"),
        ])
    );
}

#[test]
fn test_find_action_by_binding_id() {
    let f = Fixture::new();
    assert!(f.doc.is_named(f.action("B_S1_S_Confirm_PV_1"), NS, "CanSend"));
    assert_eq!(find_action_by_binding_id(&f.doc, f.party_a, "B_S1_S_Confirm_PV_1"), None);
    assert_eq!(find_action_by_binding_id(&f.doc, f.doc.root().unwrap(), ""), None);
}

// ------------------------------------------------------------------------------------------------------------------
// TestValidate
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_validate_action_without_problems() {
    let f = Fixture::new();
    assert!(validate_action(&f.doc, f.action("A_S1_S_Place_PV_1")).is_empty());
    assert!(validate_action(&f.doc, f.action("B_S1_R_Place_PV_1")).is_empty());
}

#[test]
fn test_validate_action_unknown_channel_and_package() {
    let mut f = Fixture::new();
    let action = f.action("A_S1_S_Place_PV_1");
    let binding = f.doc.find(action, NS, "ThisPartyActionBinding").unwrap();
    let channel = f.doc.find(binding, NS, "ChannelId").unwrap();
    f.doc.set_text(channel, Some("B_default"));
    f.doc.set_attribute(binding, NS, "packageId", "missing");
    assert_eq!(
        validate_action(&f.doc, action),
        ids(&[
            "ChannelId 'B_default' is not a DeliveryChannel of this partner",
            "packageId 'missing' is not a Packaging of the CPA",
        ])
    );
}

#[test]
fn test_validate_action_missing_other_party_binding() {
    let mut f = Fixture::new();
    let action = f.action("A_S1_S_Place_PV_1");
    let other = f.doc.find(action, NS, "OtherPartyActionBinding").unwrap();
    f.doc.remove_with_tail(other);
    assert_eq!(validate_action(&f.doc, action), ids(&["No OtherPartyActionBinding"]));
}

#[test]
fn test_validate_action_other_party_binding_not_found() {
    let mut f = Fixture::new();
    let action = f.action("A_S1_S_Place_PV_1");
    let other = f.doc.find(action, NS, "OtherPartyActionBinding").unwrap();
    f.doc.set_text(other, Some("A_S1_R_Confirm_PV_1"));
    assert_eq!(
        validate_action(&f.doc, action),
        ids(&["OtherPartyActionBinding 'A_S1_R_Confirm_PV_1' is not an action of the other partner"])
    );
}

#[test]
fn test_validate_action_other_party_binding_same_direction() {
    let mut f = Fixture::new();
    let action = f.action("A_S1_S_Place_PV_1");
    let other = f.doc.find(action, NS, "OtherPartyActionBinding").unwrap();
    f.doc.set_text(other, Some("B_S1_S_Confirm_PV_1"));
    assert_eq!(
        validate_action(&f.doc, action),
        ids(&[
            "OtherPartyActionBinding 'B_S1_S_Confirm_PV_1' is also a Send action",
            "OtherPartyActionBinding 'B_S1_S_Confirm_PV_1' does not refer back to 'A_S1_S_Place_PV_1'",
        ])
    );
}

#[test]
fn test_validate_action_other_party_binding_does_not_refer_back() {
    let mut f = Fixture::new();
    let other = f.doc.find(f.action("B_S1_R_Place_PV_1"), NS, "OtherPartyActionBinding").unwrap();
    f.doc.set_text(other, Some("something_else"));
    assert_eq!(
        validate_action(&f.doc, f.action("A_S1_S_Place_PV_1")),
        ids(&["OtherPartyActionBinding 'B_S1_R_Place_PV_1' does not refer back to 'A_S1_S_Place_PV_1'"])
    );
}

// ------------------------------------------------------------------------------------------------------------------
// TestCounterpart
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_find_counterpart_collaboration_role_by_links() {
    let f = Fixture::new();
    assert_eq!(find_counterpart_collaboration_role(&f.doc, f.order_a), Some(f.order_b));
    assert_eq!(find_counterpart_collaboration_role(&f.doc, f.order_b), Some(f.order_a));
}

#[test]
fn test_find_counterpart_collaboration_role_by_service() {
    let f = Fixture::new();
    assert_eq!(find_counterpart_collaboration_role(&f.doc, f.invoice_a), Some(f.invoice_b));
}

#[test]
fn test_find_counterpart_collaboration_role_not_found() {
    let mut f = Fixture::new();
    let service = service_of(&f.doc, f.invoice_b);
    f.doc.set_text(service, Some("Other"));
    assert_eq!(find_counterpart_collaboration_role(&f.doc, f.invoice_a), None);
}

// ------------------------------------------------------------------------------------------------------------------
// TestSuggestions
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_make_valid_id() {
    assert_eq!(make_valid_id("Order Service/v1"), "Order_Service_v1");
    assert_eq!(make_valid_id("1abc"), "_1abc");
    assert_eq!(make_valid_id(""), "_");
    assert_eq!(make_valid_id("A_b-1.0"), "A_b-1.0");
}

#[test]
fn test_make_unique_id() {
    let f = Fixture::new();
    assert_eq!(make_unique_id(&f.doc, "new", &[]), "new");
    assert_eq!(make_unique_id(&f.doc, "pkg", &[]), "pkg_2");
    assert_eq!(make_unique_id(&f.doc, "new", &ids(&["new", "new_2"])), "new_3");
}

#[test]
fn test_suggest_binding_id_follows_example() {
    let f = Fixture::new();
    assert_eq!(suggest_binding_id(&f.doc, f.order_a, Direction::Send, "Cancel", &[]), "A_S1_S_Cancel_PV_1");
    assert_eq!(suggest_binding_id(&f.doc, f.order_b, Direction::Receive, "Cancel", &[]), "B_S1_R_Cancel_PV_1");
}

#[test]
fn test_suggest_binding_id_without_example() {
    let f = Fixture::new();
    assert_eq!(
        suggest_binding_id(&f.doc, f.invoice_a, Direction::Send, "Send Invoice", &[]),
        "A_Invoice_S_Send_Invoice"
    );
}

#[test]
fn test_suggest_binding_id_is_unique() {
    let f = Fixture::new();
    assert_eq!(suggest_binding_id(&f.doc, f.order_a, Direction::Send, "Place", &[]), "A_S1_S_Place_PV_1_2");
    assert_eq!(
        suggest_binding_id(&f.doc, f.order_a, Direction::Send, "Cancel", &ids(&["A_S1_S_Cancel_PV_1"])),
        "A_S1_S_Cancel_PV_1_2"
    );
}

#[test]
fn test_suggest_channel_id() {
    let f = Fixture::new();
    assert_eq!(suggest_channel_id(&f.doc, f.order_a, Direction::Send), "A_reliable");
    assert_eq!(suggest_channel_id(&f.doc, f.invoice_a, Direction::Send), "A_default");
    assert_eq!(suggest_channel_id(&f.doc, f.invoice_b, Direction::Send), "B_default");
}

#[test]
fn test_suggest_characteristics() {
    let f = Fixture::new();
    assert_eq!(suggest_characteristics(&f.doc, f.invoice_a), default_characteristics());
    let characteristics = suggest_characteristics(&f.doc, f.order_a);
    assert_eq!(characteristic(&characteristics, "isAuthenticated"), "persistent");
    assert_eq!(characteristic(&characteristics, "timeToPerform"), "PT2H");
    assert_eq!(characteristic(&characteristics, "isTamperProof"), "transient");
}

// ------------------------------------------------------------------------------------------------------------------
// TestBuildAndInsert
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_build_action() {
    let mut f = Fixture::new();
    let element = build_action(
        &mut f.doc,
        Direction::Receive,
        "id1",
        "Do",
        "pkg",
        "A_default",
        &chars(&[("isAuthenticated", "none"), ("timeToPerform", "")]),
        "other1",
    );
    let action = describe_action(&f.doc, element);
    assert_eq!(
        (action.direction, action.id.as_str(), action.action.as_str(), action.package_id.as_str()),
        (Direction::Receive, "id1", "Do", "pkg")
    );
    assert_eq!(action.channel_ids, ids(&["A_default"]));
    assert_eq!(action.other_party_binding, "other1");
    assert_eq!(action.characteristics, chars(&[("isAuthenticated", "none")]));
    let without = build_action(&mut f.doc, Direction::Send, "id2", "Do", "pkg", "c", &Vec::new(), "");
    assert_eq!(f.doc.find(without, NS, "OtherPartyActionBinding"), None);
}

#[test]
fn test_insert_action_keeps_schema_order() {
    let mut f = Fixture::new();
    let send = build_action(&mut f.doc, Direction::Send, "new_send", "X", "pkg", "A_default", &Vec::new(), "");
    insert_action(&mut f.doc, f.order_a, send).unwrap();
    let receive = build_action(&mut f.doc, Direction::Receive, "new_receive", "Y", "pkg", "A_default", &Vec::new(), "");
    insert_action(&mut f.doc, f.order_a, receive).unwrap();
    let binding = f.doc.find(f.order_a, NS, "ServiceBinding").unwrap();
    assert_eq!(local_names(&f.doc, binding), ["Service", "CanSend", "CanSend", "CanReceive", "CanReceive"]);
    assert_eq!(f.binding_ids(f.order_a), ids(&["A_S1_S_Place_PV_1", "new_send", "A_S1_R_Confirm_PV_1", "new_receive"]));
}

#[test]
fn test_insert_indented_matches_siblings() {
    let mut f = Fixture::new();
    let send = build_action(&mut f.doc, Direction::Send, "new_send", "X", "pkg", "A_default", &Vec::new(), "");
    insert_action(&mut f.doc, f.invoice_a, send).unwrap();
    let text = f.doc.element_to_xml(f.invoice_a);
    assert!(text.contains("\n                <tns:Service"), "{text}");
    assert!(
        text.contains("\n                <tns:CanSend>\n                    <tns:ThisPartyActionBinding"),
        "{text}"
    );
    assert!(text.contains("\n                        <tns:ChannelId>A_default</tns:ChannelId>"), "{text}");
    assert!(text.contains("</tns:CanSend>\n            </tns:ServiceBinding>"), "{text}");
}

#[test]
fn test_insert_indented_without_whitespace() {
    let mut doc = Document::parse("<a><b/></a>").unwrap();
    let root = doc.root().unwrap();
    let c = doc.create_element(None, "c");
    doc.append_element(c, None, "d");
    insert_indented(&mut doc, root, 1, c);
    assert_eq!(doc.element_to_xml(root), "<a><b/><c><d/></c></a>");
}

// ------------------------------------------------------------------------------------------------------------------
// TestAddAction
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_add_action_without_mirror() {
    let mut f = Fixture::new();
    let (element, mirror_element) = add_action(
        &mut f.doc,
        f.order_a,
        Direction::Send,
        " Cancel ",
        "pkg",
        "A_reliable",
        &default_characteristics(),
        "A_cancel",
        None,
    )
    .unwrap();
    assert_eq!(mirror_element, None);
    let action = describe_action(&f.doc, element);
    assert_eq!((action.action.as_str(), action.other_party_binding.as_str()), ("Cancel", ""));
    assert!(f.binding_ids(f.order_a).contains(&"A_cancel".to_string()));
    let original = Document::parse(&cpa_text()).unwrap();
    let original_b = party_infos(&original)[1];
    assert_eq!(f.binding_ids(f.party_b), binding_ids(&original, original_b));
}

#[test]
fn test_add_action_with_mirror() {
    let mut f = Fixture::new();
    let (element, mirror_element) = add_action(
        &mut f.doc,
        f.order_a,
        Direction::Send,
        "Cancel",
        "other_pkg",
        "A_reliable",
        &chars(&[("isAuthenticated", "none")]),
        "A_cancel",
        Some(&mirror(f.order_b, "B_cancel", "B_default")),
    )
    .unwrap();
    let mirror_element = mirror_element.unwrap();
    let mirrored = describe_action(&f.doc, mirror_element);
    assert_eq!(mirrored.direction, Direction::Receive);
    assert_eq!(
        (mirrored.id.as_str(), mirrored.action.as_str(), mirrored.package_id.as_str()),
        ("B_cancel", "Cancel", "other_pkg")
    );
    assert_eq!(mirrored.channel_ids, ids(&["B_default"]));
    assert_eq!(mirrored.characteristics, chars(&[("isAuthenticated", "none")]));
    assert_eq!(mirrored.other_party_binding, "A_cancel");
    assert_eq!(describe_action(&f.doc, element).other_party_binding, "B_cancel");
    assert!(validate_action(&f.doc, element).is_empty());
    assert!(validate_action(&f.doc, mirror_element).is_empty());
}

fn assert_error<T: std::fmt::Debug>(result: Result<T, String>, expected: &str) {
    let error = result.unwrap_err();
    assert!(error.contains(expected), "{error:?} does not contain {expected:?}");
}

#[test]
fn test_add_action_rejects_used_id() {
    let mut f = Fixture::new();
    let (doc, order_a, order_b) = (&mut f.doc, f.order_a, f.order_b);
    let empty = Vec::new();
    assert_error(
        add_action(doc, order_a, Direction::Send, "X", "pkg", "A_default", &empty, "A_S1_R_Confirm_PV_1", None),
        "already used",
    );
    assert_error(
        add_action(
            doc,
            order_a,
            Direction::Send,
            "X",
            "pkg",
            "A_default",
            &empty,
            "new",
            Some(&mirror(order_b, "pkg", "B_default")),
        ),
        "already used",
    );
    assert_error(
        add_action(
            doc,
            order_a,
            Direction::Send,
            "X",
            "pkg",
            "A_default",
            &empty,
            "new",
            Some(&mirror(order_b, "new", "B_default")),
        ),
        "different ids",
    );
    assert_error(
        add_action(doc, order_a, Direction::Send, "X", "pkg", "A_default", &empty, "with space", None),
        "not a valid id",
    );
    let binding = doc.find(order_a, NS, "ServiceBinding").unwrap();
    assert_eq!(lxml_len(doc, binding), 3);
}

#[test]
fn test_add_action_rejects_missing_values() {
    let mut f = Fixture::new();
    let empty = Vec::new();
    for (action_name, package_id, channel_id, message) in
        [("", "pkg", "c", "Action"), ("X", "", "c", "Package"), ("X", "pkg", "", "Channel")]
    {
        assert_error(
            add_action(
                &mut f.doc,
                f.order_a,
                Direction::Send,
                action_name,
                package_id,
                channel_id,
                &empty,
                "new",
                None,
            ),
            message,
        );
    }
    assert_error(
        add_action(
            &mut f.doc,
            f.order_a,
            Direction::Send,
            "X",
            "pkg",
            "A_default",
            &empty,
            "new",
            Some(&mirror(f.order_b, "new_b", "")),
        ),
        "mirrored",
    );
}

#[test]
fn test_add_action_result_is_valid_against_schema() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas/full_cpa_p.xml");
    let mut doc = Document::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let baseline: Vec<String> = validate_document(&doc).unwrap().iter().map(|e| e.to_string()).collect();
    assert_eq!(baseline, Vec::<String>::new(), "the unchanged CPA must be valid");

    let parties = party_infos(&doc);
    let party_a = parties[0];
    let role_a = doc.find(party_a, NS, "CollaborationRole").unwrap();
    let role_b = find_counterpart_collaboration_role(&doc, role_a).unwrap();
    let root = doc.root().unwrap();
    let package_id = doc.attribute(doc.find(root, NS, "Packaging").unwrap(), NS, "id").unwrap().to_string();
    for direction in [Direction::Send, Direction::Receive] {
        let action_name = format!("New{}", direction.element_name());
        let binding_id = suggest_binding_id(&doc, role_a, direction, &action_name, &[]);
        let mirror_id =
            suggest_binding_id(&doc, role_b, direction.opposite(), &action_name, std::slice::from_ref(&binding_id));
        let channel_id = suggest_channel_id(&doc, role_a, direction);
        let characteristics = suggest_characteristics(&doc, role_a);
        let mirror_channel = suggest_channel_id(&doc, role_b, direction.opposite());
        add_action(
            &mut doc,
            role_a,
            direction,
            &action_name,
            &package_id,
            &channel_id,
            &characteristics,
            &binding_id,
            Some(&mirror(role_b, &mirror_id, &mirror_channel)),
        )
        .unwrap();
    }
    add_collaboration_role(&mut doc, party_a, &new_role("P", "1", "href", "", "R", "href", "S", "")).unwrap();
    let errors: Vec<String> = validate_document(&doc).unwrap().iter().map(|e| e.to_string()).collect();
    assert_eq!(errors, Vec::<String>::new());
}

// ------------------------------------------------------------------------------------------------------------------
// TestUpdateAndDelete
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_update_action() {
    let mut f = Fixture::new();
    let element = f.action("A_S1_S_Place_PV_1");
    update_action(
        &mut f.doc,
        element,
        "PlaceNew",
        "other_pkg",
        "A_default",
        &chars(&[("isAuthenticated", "none"), ("timeToPerform", ""), ("isTamperProof", "persistent")]),
    )
    .unwrap();
    let action = describe_action(&f.doc, element);
    assert_eq!(
        (action.id.as_str(), action.action.as_str(), action.package_id.as_str()),
        ("A_S1_S_Place_PV_1", "PlaceNew", "other_pkg")
    );
    assert_eq!(action.channel_ids, ids(&["A_default"]));
    assert_eq!(
        action.characteristics,
        chars(&[
            ("isAuthenticated", "none"),
            ("isAuthorizationRequired", "true"),
            ("isConfidential", "transient"),
            ("isTamperProof", "persistent"),
        ])
    );
}

#[test]
fn test_update_action_rejects_empty_action() {
    let mut f = Fixture::new();
    let element = f.action("A_S1_S_Place_PV_1");
    assert_error(update_action(&mut f.doc, element, " ", "pkg", "A_default", &Vec::new()), "Action");
    assert_eq!(describe_action(&f.doc, f.action("A_S1_S_Place_PV_1")).action, "Place");
}

#[test]
fn test_delete_action_only() {
    let mut f = Fixture::new();
    let element = f.action("A_S1_S_Place_PV_1");
    assert_eq!(delete_action(&mut f.doc, element, false), ids(&["A_S1_S_Place_PV_1"]));
    assert_eq!(f.binding_ids(f.order_a), ids(&["A_S1_R_Confirm_PV_1"]));
    assert!(find_action_by_binding_id(&f.doc, f.doc.root().unwrap(), "B_S1_R_Place_PV_1").is_some());
}

#[test]
fn test_delete_action_with_counterpart() {
    let mut f = Fixture::new();
    let element = f.action("A_S1_R_Confirm_PV_1");
    assert_eq!(delete_action(&mut f.doc, element, true), ids(&["A_S1_R_Confirm_PV_1", "B_S1_S_Confirm_PV_1"]));
    assert_eq!(f.binding_ids(f.order_a), ids(&["A_S1_S_Place_PV_1"]));
    assert_eq!(f.binding_ids(f.order_b), ids(&["B_S1_R_Place_PV_1"]));
    assert!(f.doc.element_to_xml(f.order_a).contains("</tns:CanSend>\n            </tns:ServiceBinding>"));
}

// ------------------------------------------------------------------------------------------------------------------
// TestCollaborationRoleElements
// ------------------------------------------------------------------------------------------------------------------

#[test]
fn test_build_collaboration_role() {
    let mut doc = Document::parse("<a/>").unwrap();
    let role =
        build_collaboration_role(&mut doc, &new_role("P", "2.0", "phref", "urn:uuid", "R", "rhref", "S", "urn:type"));
    assert_eq!(
        describe_collaboration_role(&doc, role),
        RoleDescription {
            process_name: "P".into(),
            process_version: "2.0".into(),
            role_name: "R".into(),
            service_name: "S".into(),
            service_type: "urn:type".into(),
        }
    );
    let process = doc.find(role, NS, "ProcessSpecification").unwrap();
    assert_eq!(doc.attribute(process, NS, "uuid"), Some("urn:uuid"));
    assert_eq!(doc.attribute(process, Some("http://www.w3.org/1999/xlink"), "href"), Some("phref"));
    let without_type = build_collaboration_role(&mut doc, &new_role("P", "1", "", "", "R", "", "S", ""));
    assert_eq!(doc.attribute(service_of(&doc, without_type), NS, "type"), None);
}

#[test]
fn test_add_collaboration_role() {
    let mut f = Fixture::new();
    let role =
        add_collaboration_role(&mut f.doc, f.party_a, &new_role(" P ", "1", "", "", "R", "", "New", "")).unwrap();
    assert_eq!(
        local_names(&f.doc, f.party_a),
        [
            "PartyId",
            "PartyRef",
            "CollaborationRole",
            "CollaborationRole",
            "CollaborationRole",
            "DeliveryChannel",
            "DeliveryChannel"
        ]
    );
    assert_eq!(f.doc.child_elements(f.party_a).nth(4), Some(role));
    assert_eq!(describe_collaboration_role(&f.doc, role).process_name, "P");
}

#[test]
fn test_add_collaboration_role_rejects_missing_values() {
    let mut f = Fixture::new();
    assert_error(
        add_collaboration_role(&mut f.doc, f.party_a, &new_role("P", "1", "", "", "R", "", " ", "")),
        "Service name",
    );
    assert_eq!(f.doc.find_all(f.party_a, NS, "CollaborationRole").count(), 2);
}

#[test]
fn test_delete_collaboration_role() {
    let mut f = Fixture::new();
    delete_collaboration_role(&mut f.doc, f.invoice_a).unwrap();
    assert_eq!(f.doc.find_all(f.party_a, NS, "CollaborationRole").collect::<Vec<_>>(), [f.order_a]);
}

#[test]
fn test_delete_last_collaboration_role_is_refused() {
    let mut f = Fixture::new();
    delete_collaboration_role(&mut f.doc, f.invoice_a).unwrap();
    assert_error(delete_collaboration_role(&mut f.doc, f.order_a), "at least one");
    assert_eq!(f.doc.find_all(f.party_a, NS, "CollaborationRole").collect::<Vec<_>>(), [f.order_a]);
}
