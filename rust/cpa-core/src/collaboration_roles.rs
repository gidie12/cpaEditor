//! Reading and editing the CollaborationRoles of a CPA and the Service actions (CanSend and CanReceive) in their
//! ServiceBinding. Port of `application/helper_classes/collaborationRoles.py`.
//!
//! Edits keep the indentation of the CPA, so a CPA saved after an edit is byte for byte what the Python version
//! writes (see `tests/collaboration_roles_parity.rs`).
//!
//! Positions among the children of an element follow lxml: they count elements, comments and processing
//! instructions, not text.

use std::collections::HashSet;

use crate::xml::{Document, NodeId, NodeKind};
use crate::{CPA_NAMESPACE, XLINK_NAMESPACE};

const CPA: Option<&str> = Some(CPA_NAMESPACE);
const XLINK: Option<&str> = Some(XLINK_NAMESPACE);

/// The direction of a Service action: the element CanSend or CanReceive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Send,
    Receive,
}

impl Direction {
    /// The local name of the element: `CanSend` or `CanReceive`.
    pub fn element_name(self) -> &'static str {
        match self {
            Direction::Send => "CanSend",
            Direction::Receive => "CanReceive",
        }
    }

    /// The label shown to the user: `Send` or `Receive`.
    pub fn label(self) -> &'static str {
        match self {
            Direction::Send => "Send",
            Direction::Receive => "Receive",
        }
    }

    /// Returns Receive for Send and Send for Receive.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Send => Direction::Receive,
            Direction::Receive => Direction::Send,
        }
    }

    /// The direction of a CPA element CanSend or CanReceive; None for other nodes.
    pub fn of_element(document: &Document, element: NodeId) -> Option<Direction> {
        if document.is_named(element, CPA, "CanSend") {
            Some(Direction::Send)
        } else if document.is_named(element, CPA, "CanReceive") {
            Some(Direction::Receive)
        } else {
            None
        }
    }
}

/// Returns Receive for Send and Send for Receive.
pub fn opposite_direction(direction: Direction) -> Direction {
    direction.opposite()
}

pub const PERSISTENCE_LEVELS: &[&str] = &["none", "transient", "persistent", "transient-and-persistent"];
pub const BOOLEAN_VALUES: &[&str] = &["true", "false"];

/// An attribute of BusinessTransactionCharacteristics with its allowed values (None: free text) and the default
/// value of a new action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Characteristic {
    pub name: &'static str,
    pub allowed: Option<&'static [&'static str]>,
    pub default: &'static str,
}

/// The attributes of BusinessTransactionCharacteristics. The order is the order in which the attributes are shown
/// and written.
pub const CHARACTERISTICS: &[Characteristic] = &[
    Characteristic { name: "isAuthenticated", allowed: Some(PERSISTENCE_LEVELS), default: "transient" },
    Characteristic { name: "isAuthorizationRequired", allowed: Some(BOOLEAN_VALUES), default: "true" },
    Characteristic { name: "isIntelligibleCheckRequired", allowed: Some(BOOLEAN_VALUES), default: "false" },
    Characteristic { name: "isNonRepudiationReceiptRequired", allowed: Some(BOOLEAN_VALUES), default: "false" },
    Characteristic { name: "isNonRepudiationRequired", allowed: Some(BOOLEAN_VALUES), default: "false" },
    Characteristic { name: "isConfidential", allowed: Some(PERSISTENCE_LEVELS), default: "transient" },
    Characteristic { name: "isTamperProof", allowed: Some(PERSISTENCE_LEVELS), default: "transient" },
    Characteristic { name: "timeToPerform", allowed: None, default: "PT1H" },
];

/// Attribute name (without namespace) -> value of BusinessTransactionCharacteristics, in the order of the
/// attributes. Like the Python dict, a name occurs at most once when it is filled by the functions of this module.
pub type Characteristics = Vec<(String, String)>;

/// The value of a characteristic; empty when it is missing.
pub fn characteristic<'a>(characteristics: &'a Characteristics, name: &str) -> &'a str {
    characteristics.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str()).unwrap_or("")
}

/// Sets a characteristic like a Python dict: an existing name keeps its position, a new one is added at the end.
pub fn set_characteristic(characteristics: &mut Characteristics, name: &str, value: &str) {
    match characteristics.iter_mut().find(|(n, _)| n == name) {
        Some(entry) => entry.1 = value.to_string(),
        None => characteristics.push((name.to_string(), value.to_string())),
    }
}

/// The default characteristics of a new action.
pub fn default_characteristics() -> Characteristics {
    CHARACTERISTICS.iter().map(|c| (c.name.to_string(), c.default.to_string())).collect()
}

/// The values that identify a CollaborationRole. A missing value is an empty string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoleDescription {
    pub process_name: String,
    pub process_version: String,
    pub role_name: String,
    pub service_name: String,
    pub service_type: String,
}

/// The values of a CanSend or CanReceive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionDescription {
    pub element: NodeId,
    pub direction: Direction,
    pub id: String,
    pub action: String,
    pub package_id: String,
    pub channel_ids: Vec<String>,
    /// Empty when missing.
    pub other_party_binding: String,
    pub characteristics: Characteristics,
}

/// The mirrored action of [`add_action`]: the CollaborationRole of the other partner that gets it, its id and its
/// channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mirror {
    pub collaboration_role: NodeId,
    pub binding_id: String,
    pub channel_id: String,
}

/// The values of a new CollaborationRole for [`build_collaboration_role`] and [`add_collaboration_role`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewCollaborationRole {
    pub process_name: String,
    pub process_version: String,
    pub process_href: String,
    pub process_uuid: String,
    pub role_name: String,
    pub role_href: String,
    pub service_name: String,
    pub service_type: String,
}

// --------------------------------------------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------------------------------------------

/// Python's `str.isspace` for one character (it also counts the separators U+001C to U+001F).
fn is_python_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's `str.strip()`.
fn py_strip(value: &str) -> &str {
    value.trim_matches(is_python_space)
}

/// The text of an element without surrounding whitespace, or an empty string.
fn text_of(document: &Document, element: Option<NodeId>) -> String {
    element.and_then(|e| document.text(e)).map(|t| py_strip(&t).to_string()).unwrap_or_default()
}

/// A CPA attribute of an element.
fn cpa_attribute<'a>(document: &'a Document, element: NodeId, name: &str) -> Option<&'a str> {
    document.attribute(element, CPA, name)
}

/// The children as lxml counts them (`len(parent)`, `parent[i]`): elements, comments and processing
/// instructions, without text.
fn lxml_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document.children(parent).iter().copied().filter(|&c| !matches!(document.kind(c), NodeKind::Text(_))).collect()
}

/// lxml `parent.insert(index, element)`: directly before the child at the lxml index (after the tail of the
/// previous child), or at the end when there is no such child.
fn lxml_insert(document: &mut Document, parent: NodeId, index: usize, element: NodeId) {
    match lxml_children(document, parent).get(index) {
        Some(&reference) => document.insert_before(reference, element),
        None => document.append_child(parent, element),
    }
}

/// The text after the last line end (Python `value.rsplit('\n', 1)[-1]`).
fn last_line(value: &str) -> &str {
    value.rsplit('\n').next().unwrap_or(value)
}

// --------------------------------------------------------------------------------------------------------------
// Reading
// --------------------------------------------------------------------------------------------------------------

/// Returns the PartyInfo elements of the CPA.
pub fn party_infos(document: &Document) -> Vec<NodeId> {
    match document.root() {
        Some(root) => document.find_all(root, CPA, "PartyInfo").collect(),
        None => Vec::new(),
    }
}

/// Returns the partyName of a PartyInfo element.
pub fn party_name(document: &Document, party_info: NodeId) -> String {
    cpa_attribute(document, party_info, "partyName").unwrap_or("").to_string()
}

/// Returns the PartyInfo of the other partner, or None when the CPA has no other partner.
pub fn other_party_info(document: &Document, party_info: NodeId) -> Option<NodeId> {
    party_infos(document).into_iter().find(|&other| other != party_info)
}

/// Returns the channelIds of the DeliveryChannels of a partner, in the order of the CPA.
pub fn channel_ids(document: &Document, party_info: NodeId) -> Vec<String> {
    document
        .find_all(party_info, CPA, "DeliveryChannel")
        .filter_map(|channel| cpa_attribute(document, channel, "channelId"))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

/// Returns the ids of the Packaging elements of the CPA, in the order of the CPA.
pub fn package_ids(document: &Document) -> Vec<String> {
    let Some(root) = document.root() else { return Vec::new() };
    document
        .find_all(root, CPA, "Packaging")
        .filter_map(|package| cpa_attribute(document, package, "id"))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

/// Returns all values of id attributes in the CPA: a new id must not be one of them (type ID).
pub fn used_ids(document: &Document) -> HashSet<String> {
    let Some(root) = document.root() else { return HashSet::new() };
    document
        .descendants(root)
        .filter_map(|node| cpa_attribute(document, node, "id"))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

/// Returns the PartyInfo element that contains the element (or is the element), or None.
pub fn party_info_of(document: &Document, element: NodeId) -> Option<NodeId> {
    document.ancestor_named(element, CPA, "PartyInfo")
}

/// Returns the CollaborationRole element that contains the element (or is the element), or None.
pub fn collaboration_role_of(document: &Document, element: NodeId) -> Option<NodeId> {
    document.ancestor_named(element, CPA, "CollaborationRole")
}

/// Collects the values of a CollaborationRole that identify it: process name and version, role name, service
/// name and type. A missing value is an empty string.
pub fn describe_collaboration_role(document: &Document, collaboration_role: NodeId) -> RoleDescription {
    let process = document.find(collaboration_role, CPA, "ProcessSpecification");
    let role = document.find(collaboration_role, CPA, "Role");
    // lxml find('ServiceBinding/Service'): the first Service of any ServiceBinding
    let service = document
        .find_all(collaboration_role, CPA, "ServiceBinding")
        .find_map(|binding| document.find(binding, CPA, "Service"));
    let get = |element: Option<NodeId>, name: &str| {
        element.and_then(|e| cpa_attribute(document, e, name)).unwrap_or("").to_string()
    };
    RoleDescription {
        process_name: get(process, "name"),
        process_version: get(process, "version"),
        role_name: get(role, "name"),
        service_name: text_of(document, service),
        service_type: get(service, "type"),
    }
}

/// Collects the values of a CanSend or CanReceive element: direction, id, action, packageId, the ChannelIds, the
/// OtherPartyActionBinding (empty when missing) and the attributes of BusinessTransactionCharacteristics (without
/// namespace).
///
/// The direction is Receive for a CanReceive and Send for any other element.
pub fn describe_action(document: &Document, action_element: NodeId) -> ActionDescription {
    let binding = document.find(action_element, CPA, "ThisPartyActionBinding");
    let get = |name: &str| binding.and_then(|b| cpa_attribute(document, b, name)).unwrap_or("").to_string();
    let mut characteristics = Characteristics::new();
    if let Some(element) = binding.and_then(|b| document.find(b, CPA, "BusinessTransactionCharacteristics")) {
        for attribute in document.attributes(element) {
            set_characteristic(&mut characteristics, &attribute.name.local, &attribute.value);
        }
    }
    ActionDescription {
        element: action_element,
        direction: Direction::of_element(document, action_element).unwrap_or(Direction::Send),
        id: get("id"),
        action: get("action"),
        package_id: get("packageId"),
        channel_ids: match binding {
            Some(b) => document.find_all(b, CPA, "ChannelId").map(|c| text_of(document, Some(c))).collect(),
            None => Vec::new(),
        },
        other_party_binding: text_of(document, document.find(action_element, CPA, "OtherPartyActionBinding")),
        characteristics,
    }
}

/// Returns the CanSend and CanReceive children of a ServiceBinding (or of a CanSend/CanReceive).
pub fn action_elements(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document.child_elements(parent).filter(|&child| Direction::of_element(document, child).is_some()).collect()
}

/// Returns the ServiceBinding of a CollaborationRole, or None.
pub fn service_binding(document: &Document, collaboration_role: NodeId) -> Option<NodeId> {
    document.find(collaboration_role, CPA, "ServiceBinding")
}

/// Finds the CanSend or CanReceive whose ThisPartyActionBinding has the id, searching the container (for
/// example a PartyInfo or the root) and its descendants. None for an empty id.
pub fn find_action_by_binding_id(document: &Document, container: NodeId, binding_id: &str) -> Option<NodeId> {
    if binding_id.is_empty() {
        return None;
    }
    let binding = document
        .iter_named(container, CPA, "ThisPartyActionBinding")
        .find(|&binding| cpa_attribute(document, binding, "id") == Some(binding_id))?;
    document.parent_element(binding)
}

/// Checks the references of a CanSend or CanReceive.
///
/// The ChannelId must be a DeliveryChannel of the same partner, the packageId a Packaging of the CPA and the
/// OtherPartyActionBinding the opposite action of the other partner, which refers back.
///
/// Returns descriptions of the problems; empty when the action is correct.
pub fn validate_action(document: &Document, action_element: NodeId) -> Vec<String> {
    let action = describe_action(document, action_element);
    let party_info = party_info_of(document, action_element);
    let mut problems = Vec::new();

    let own_channels = party_info.map(|p| channel_ids(document, p)).unwrap_or_default();
    for channel_id in &action.channel_ids {
        if !own_channels.contains(channel_id) {
            problems.push(format!("ChannelId '{channel_id}' is not a DeliveryChannel of this partner"));
        }
    }
    if !package_ids(document).contains(&action.package_id) {
        problems.push(format!("packageId '{}' is not a Packaging of the CPA", action.package_id));
    }

    let other_binding = &action.other_party_binding;
    if other_binding.is_empty() {
        problems.push("No OtherPartyActionBinding".to_string());
        return problems;
    }
    let counterpart = party_info
        .and_then(|p| other_party_info(document, p))
        .and_then(|other| find_action_by_binding_id(document, other, other_binding));
    let Some(counterpart) = counterpart else {
        problems.push(format!("OtherPartyActionBinding '{other_binding}' is not an action of the other partner"));
        return problems;
    };
    let counterpart_action = describe_action(document, counterpart);
    if Direction::of_element(document, counterpart) != Some(action.direction.opposite()) {
        problems
            .push(format!("OtherPartyActionBinding '{other_binding}' is also a {} action", action.direction.label()));
    }
    if counterpart_action.other_party_binding != action.id {
        problems.push(format!("OtherPartyActionBinding '{other_binding}' does not refer back to '{}'", action.id));
    }
    problems
}

/// Finds the CollaborationRole of the other partner that belongs to a CollaborationRole.
///
/// First the OtherPartyActionBindings of its actions are followed. Without links the CollaborationRole of the
/// other partner with the same Service (name and type) is used. None when there is none.
pub fn find_counterpart_collaboration_role(document: &Document, collaboration_role: NodeId) -> Option<NodeId> {
    let other = other_party_info(document, party_info_of(document, collaboration_role)?)?;
    for binding in document.iter_named(collaboration_role, CPA, "ThisPartyActionBinding") {
        let other_party_binding = match document.parent_element(binding) {
            Some(action) => describe_action(document, action).other_party_binding,
            None => continue,
        };
        if let Some(counterpart) = find_action_by_binding_id(document, other, &other_party_binding) {
            return collaboration_role_of(document, counterpart);
        }
    }
    let service = describe_collaboration_role(document, collaboration_role);
    document.find_all(other, CPA, "CollaborationRole").find(|&other_role| {
        let other_service = describe_collaboration_role(document, other_role);
        other_service.service_name == service.service_name && other_service.service_type == service.service_type
    })
}

// --------------------------------------------------------------------------------------------------------------
// Suggestions for a new action
// --------------------------------------------------------------------------------------------------------------

/// Turns a text into a valid XML id (NCName): other characters become '_' and a leading digit, '.' or '-' gets a
/// '_' in front.
pub fn make_valid_id(value: &str) -> String {
    let valid: String = py_strip(value)
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') { c } else { '_' })
        .collect();
    match valid.chars().next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => valid,
        _ => format!("_{valid}"),
    }
}

/// Adds _2, _3, ... to the id until no element of the CPA and no reserved id uses it.
pub fn make_unique_id(document: &Document, candidate: &str, reserved: &[String]) -> String {
    let mut used = used_ids(document);
    used.extend(reserved.iter().cloned());
    let mut unique = candidate.to_string();
    let mut number = 2;
    while used.contains(&unique) {
        unique = format!("{candidate}_{number}");
        number += 1;
    }
    unique
}

/// Suggests the id of the ThisPartyActionBinding of a new action.
///
/// An action with the same direction in the same CollaborationRole whose id contains its action name is used as
/// example: the action name in that id is replaced, so the naming convention of the CPA is kept. Without such an
/// action the id is `<partyName>_<service>_<S or R>_<action>`.
///
/// `reserved` are ids that are not in the CPA yet but are taken, for example the mirrored action. Returns a valid
/// id that is not used in the CPA.
pub fn suggest_binding_id(
    document: &Document,
    collaboration_role: NodeId,
    direction: Direction,
    action_name: &str,
    reserved: &[String],
) -> String {
    let mut candidate = None;
    if let Some(binding) = service_binding(document, collaboration_role) {
        for element in action_elements(document, binding) {
            let example = describe_action(document, element);
            if example.direction == direction && !example.action.is_empty() && example.id.contains(&example.action) {
                // Replace the last occurrence: the action name is near the end in the usual convention
                let start = example.id.rfind(&example.action).expect("contained");
                candidate = Some(format!(
                    "{}{}{}",
                    &example.id[..start],
                    action_name,
                    &example.id[start + example.action.len()..]
                ));
                break;
            }
        }
    }
    let candidate = candidate.unwrap_or_else(|| {
        let party = party_info_of(document, collaboration_role).map(|p| party_name(document, p)).unwrap_or_default();
        let service = describe_collaboration_role(document, collaboration_role).service_name;
        let letter = if direction == Direction::Send { "S" } else { "R" };
        [party.as_str(), service.as_str(), letter, action_name]
            .iter()
            .filter(|part| !part.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("_")
    });
    make_unique_id(document, &make_valid_id(&candidate), reserved)
}

/// Suggests the ChannelId of a new action: the channel of another action with the same direction in the
/// CollaborationRole, else the defaultMshChannelId of the partner, else its first DeliveryChannel.
pub fn suggest_channel_id(document: &Document, collaboration_role: NodeId, direction: Direction) -> String {
    if let Some(binding) = service_binding(document, collaboration_role) {
        for element in action_elements(document, binding) {
            let example = describe_action(document, element);
            if example.direction == direction && !example.channel_ids.is_empty() {
                return example.channel_ids[0].clone();
            }
        }
    }
    let Some(party_info) = party_info_of(document, collaboration_role) else { return String::new() };
    let channels = channel_ids(document, party_info);
    if let Some(default_channel) = cpa_attribute(document, party_info, "defaultMshChannelId") {
        if channels.iter().any(|c| c == default_channel) {
            return default_channel.to_string();
        }
    }
    channels.into_iter().next().unwrap_or_default()
}

/// Returns the BusinessTransactionCharacteristics for a new action: those of the first action in the
/// CollaborationRole, completed with the defaults.
pub fn suggest_characteristics(document: &Document, collaboration_role: NodeId) -> Characteristics {
    let mut characteristics = default_characteristics();
    let first = service_binding(document, collaboration_role)
        .and_then(|binding| action_elements(document, binding).into_iter().next());
    if let Some(first) = first {
        for (name, value) in describe_action(document, first).characteristics {
            set_characteristic(&mut characteristics, &name, &value);
        }
    }
    characteristics
}

// --------------------------------------------------------------------------------------------------------------
// Editing
// --------------------------------------------------------------------------------------------------------------

/// Inserts the element at the index (counted like lxml: elements, comments and processing instructions) and
/// indents it like its siblings, so the saved CPA stays readable. An index past the last child appends.
pub fn insert_indented(document: &mut Document, parent: NodeId, index: usize, element: NodeId) {
    let children = lxml_children(document, parent);
    let index = index.min(children.len());
    // The whitespace before the first child (for example "\n            ") and before the closing tag of the
    // parent (for example "\n        "): the difference is one level of indentation
    let mut sibling_whitespace = if children.is_empty() {
        None
    } else {
        document.text(parent).filter(|text| !text.is_empty() && py_strip(text).is_empty())
    };
    let closing_whitespace = children.last().and_then(|&last| document.tail(last));
    match (&sibling_whitespace, &closing_whitespace) {
        (Some(sibling), Some(closing)) if py_strip(closing).is_empty() => {
            let child_indent = last_line(sibling);
            let parent_indent = last_line(closing);
            let child_length = child_indent.chars().count();
            let space: String = child_indent.chars().skip(parent_indent.chars().count()).collect();
            let space_length = space.chars().count();
            if space_length > 0 && child_indent.starts_with(parent_indent) && child_length % space_length == 0 {
                document.indent(element, &space, child_length / space_length);
            }
        }
        _ => sibling_whitespace = None,
    }
    lxml_insert(document, parent, index, element);
    let Some(sibling_whitespace) = sibling_whitespace else { return };
    if index > 0 {
        let previous = lxml_children(document, parent)[index - 1];
        let previous_tail = document.tail(previous);
        document.set_tail(element, previous_tail.as_deref());
        document.set_tail(previous, Some(&sibling_whitespace));
    } else {
        document.set_tail(element, Some(&sibling_whitespace));
    }
}

/// Builds a CanSend or CanReceive element, not yet in the CPA.
///
/// Empty characteristics are left out; an empty `other_party_binding` leaves out the OtherPartyActionBinding.
#[allow(clippy::too_many_arguments)]
pub fn build_action(
    document: &mut Document,
    direction: Direction,
    binding_id: &str,
    action_name: &str,
    package_id: &str,
    channel_id: &str,
    characteristics: &Characteristics,
    other_party_binding: &str,
) -> NodeId {
    let action_element = document.create_element(CPA, direction.element_name());
    let binding = document.append_element(action_element, CPA, "ThisPartyActionBinding");
    document.set_attribute(binding, CPA, "id", binding_id);
    document.set_attribute(binding, CPA, "action", action_name);
    document.set_attribute(binding, CPA, "packageId", package_id);
    let characteristics_element = document.append_element(binding, CPA, "BusinessTransactionCharacteristics");
    set_characteristics(document, characteristics_element, characteristics);
    let channel = document.append_element(binding, CPA, "ChannelId");
    document.set_text(channel, Some(channel_id));
    if !other_party_binding.is_empty() {
        let other = document.append_element(action_element, CPA, "OtherPartyActionBinding");
        document.set_text(other, Some(other_party_binding));
    }
    action_element
}

/// Writes the attributes of BusinessTransactionCharacteristics; an empty value removes the attribute. The known
/// characteristics come first in the order of [`CHARACTERISTICS`], then the others in their own order.
pub fn set_characteristics(
    document: &mut Document,
    characteristics_element: NodeId,
    characteristics: &Characteristics,
) {
    let mut names: Vec<&str> = CHARACTERISTICS.iter().map(|c| c.name).collect();
    for (name, _) in characteristics {
        if !names.contains(&name.as_str()) {
            names.push(name);
        }
    }
    for name in names {
        let Some((_, value)) = characteristics.iter().find(|(n, _)| n == name) else { continue };
        let value = py_strip(value);
        if !value.is_empty() {
            document.set_attribute(characteristics_element, CPA, name, value);
        } else {
            document.remove_attribute(characteristics_element, CPA, name);
        }
    }
}

/// Adds a CanSend or CanReceive to the ServiceBinding of a CollaborationRole in the order of the schema: Service,
/// then all CanSend, then all CanReceive.
pub fn insert_action(
    document: &mut Document,
    collaboration_role: NodeId,
    action_element: NodeId,
) -> Result<(), String> {
    let binding = service_binding(document, collaboration_role)
        .ok_or_else(|| "The CollaborationRole has no ServiceBinding".to_string())?;
    let is_receive = document.is_named(action_element, CPA, "CanReceive");
    let mut index = 0;
    for (position, child) in lxml_children(document, binding).into_iter().enumerate() {
        if !document.is_element(child) {
            continue;
        }
        if document.is_named(child, CPA, "Service")
            || document.is_named(child, CPA, "CanSend")
            || (document.is_named(child, CPA, "CanReceive") && is_receive)
        {
            index = position + 1;
        }
    }
    insert_indented(document, binding, index, action_element);
    Ok(())
}

/// Adds a Service action to a CollaborationRole and optionally the mirrored action at the other partner.
///
/// The mirrored action has the opposite direction (a CanSend gets a CanReceive) and both OtherPartyActionBindings
/// refer to each other. The package and characteristics are used for both actions.
///
/// Returns the new element and the new mirrored element (None without mirror). An Err when a required value is
/// missing or an id is already used; the CPA is then unchanged. (When the CollaborationRole of the mirror has no
/// ServiceBinding the first action has already been added, as in the Python version.)
#[allow(clippy::too_many_arguments)]
pub fn add_action(
    document: &mut Document,
    collaboration_role: NodeId,
    direction: Direction,
    action_name: &str,
    package_id: &str,
    channel_id: &str,
    characteristics: &Characteristics,
    binding_id: &str,
    mirror: Option<&Mirror>,
) -> Result<(NodeId, Option<NodeId>), String> {
    let action_name = py_strip(action_name);
    if action_name.is_empty() {
        return Err("Action is required".into());
    }
    if package_id.is_empty() {
        return Err("Package is required".into());
    }
    if channel_id.is_empty() {
        return Err("Channel is required".into());
    }
    let used = used_ids(document);
    let mut ids = vec![binding_id];
    if let Some(mirror) = mirror {
        ids.push(&mirror.binding_id);
    }
    for new_id in ids {
        if new_id.is_empty() || make_valid_id(new_id) != new_id {
            return Err(format!("'{new_id}' is not a valid id"));
        }
        if used.contains(new_id) {
            return Err(format!("The id '{new_id}' is already used in the CPA"));
        }
    }
    if let Some(mirror) = mirror {
        if mirror.binding_id == binding_id {
            return Err("The action and the mirrored action need different ids".into());
        }
        if mirror.channel_id.is_empty() {
            return Err("Channel of the mirrored action is required".into());
        }
    }

    let other_party_binding = mirror.map(|m| m.binding_id.as_str()).unwrap_or("");
    let action_element = build_action(
        document,
        direction,
        binding_id,
        action_name,
        package_id,
        channel_id,
        characteristics,
        other_party_binding,
    );
    insert_action(document, collaboration_role, action_element)?;
    let mut mirror_element = None;
    if let Some(mirror) = mirror {
        let element = build_action(
            document,
            direction.opposite(),
            &mirror.binding_id,
            action_name,
            package_id,
            &mirror.channel_id,
            characteristics,
            binding_id,
        );
        insert_action(document, mirror.collaboration_role, element)?;
        mirror_element = Some(element);
    }
    Ok((action_element, mirror_element))
}

/// Changes the action, packageId, first ChannelId and BusinessTransactionCharacteristics of a CanSend or
/// CanReceive. The id stays the same, so the OtherPartyActionBinding of the other partner keeps working.
///
/// An Err when action, package or channel is empty, or when the element has no ThisPartyActionBinding.
pub fn update_action(
    document: &mut Document,
    action_element: NodeId,
    action_name: &str,
    package_id: &str,
    channel_id: &str,
    characteristics: &Characteristics,
) -> Result<(), String> {
    let action_name = py_strip(action_name);
    if action_name.is_empty() {
        return Err("Action is required".into());
    }
    if package_id.is_empty() {
        return Err("Package is required".into());
    }
    if channel_id.is_empty() {
        return Err("Channel is required".into());
    }
    let binding = document
        .find(action_element, CPA, "ThisPartyActionBinding")
        .ok_or_else(|| "The action has no ThisPartyActionBinding".to_string())?;
    document.set_attribute(binding, CPA, "action", action_name);
    document.set_attribute(binding, CPA, "packageId", package_id);
    let channel = match document.find(binding, CPA, "ChannelId") {
        Some(channel) => channel,
        None => document.append_element(binding, CPA, "ChannelId"),
    };
    document.set_text(channel, Some(channel_id));
    let characteristics_element = match document.find(binding, CPA, "BusinessTransactionCharacteristics") {
        Some(element) => element,
        None => {
            let element = document.create_element(CPA, "BusinessTransactionCharacteristics");
            lxml_insert(document, binding, 0, element);
            element
        }
    };
    set_characteristics(document, characteristics_element, characteristics);
    Ok(())
}

/// Removes an element and keeps the whitespace of its siblings correct.
pub fn remove_element(document: &mut Document, element: NodeId) {
    let Some(parent) = document.parent_element(element) else { return };
    let siblings = lxml_children(document, parent);
    let position = siblings.iter().position(|&s| s == element).expect("a child of its parent");
    let previous = position.checked_sub(1).map(|p| siblings[p]);
    let is_last = position + 1 == siblings.len();
    if let Some(previous) = previous {
        let tail = if is_last { document.tail(element) } else { document.tail(previous) };
        document.set_tail(previous, tail.as_deref());
    } else if is_last {
        // Last child: the closing tag of the parent moves to the indentation of the parent
        let tail = document.tail(element);
        document.set_text(parent, tail.as_deref());
    }
    document.remove_with_tail(element);
}

/// Deletes a CanSend or CanReceive and, with `delete_counterpart`, the action of the other partner in its
/// OtherPartyActionBinding.
///
/// Returns the ids of the deleted ThisPartyActionBindings.
pub fn delete_action(document: &mut Document, action_element: NodeId, delete_counterpart: bool) -> Vec<String> {
    let action = describe_action(document, action_element);
    let mut deleted = vec![action.id.clone()];
    let mut counterpart = None;
    if delete_counterpart {
        if let Some(other) = party_info_of(document, action_element).and_then(|p| other_party_info(document, p)) {
            counterpart = find_action_by_binding_id(document, other, &action.other_party_binding);
        }
    }
    remove_element(document, action_element);
    if let Some(counterpart) = counterpart {
        deleted.push(describe_action(document, counterpart).id);
        remove_element(document, counterpart);
    }
    deleted
}

/// Builds a CollaborationRole with a ProcessSpecification, Role and ServiceBinding without actions, not yet in the
/// CPA. The values are used as given; an empty uuid or service type is left out.
pub fn build_collaboration_role(document: &mut Document, values: &NewCollaborationRole) -> NodeId {
    let collaboration_role = document.create_element(CPA, "CollaborationRole");
    let process = document.append_element(collaboration_role, CPA, "ProcessSpecification");
    document.set_attribute(process, CPA, "name", &values.process_name);
    document.set_attribute(process, CPA, "version", &values.process_version);
    document.set_attribute(process, XLINK, "type", "simple");
    document.set_attribute(process, XLINK, "href", &values.process_href);
    if !values.process_uuid.is_empty() {
        document.set_attribute(process, CPA, "uuid", &values.process_uuid);
    }
    let role = document.append_element(collaboration_role, CPA, "Role");
    document.set_attribute(role, CPA, "name", &values.role_name);
    document.set_attribute(role, XLINK, "type", "simple");
    document.set_attribute(role, XLINK, "href", &values.role_href);
    let binding = document.append_element(collaboration_role, CPA, "ServiceBinding");
    let service = document.append_element(binding, CPA, "Service");
    if !values.service_type.is_empty() {
        document.set_attribute(service, CPA, "type", &values.service_type);
    }
    document.set_text(service, Some(&values.service_name));
    collaboration_role
}

/// Adds a CollaborationRole to a partner, after its last CollaborationRole (order of the schema). The values are
/// stripped of surrounding whitespace.
///
/// Returns the new CollaborationRole; an Err when the process name, process version, role name or service name is
/// empty.
pub fn add_collaboration_role(
    document: &mut Document,
    party_info: NodeId,
    values: &NewCollaborationRole,
) -> Result<NodeId, String> {
    for (label, value) in [
        ("Process name", &values.process_name),
        ("Process version", &values.process_version),
        ("Role name", &values.role_name),
        ("Service name", &values.service_name),
    ] {
        if py_strip(value).is_empty() {
            return Err(format!("{label} is required"));
        }
    }
    let strip = |value: &String| py_strip(value).to_string();
    let stripped = NewCollaborationRole {
        process_name: strip(&values.process_name),
        process_version: strip(&values.process_version),
        process_href: strip(&values.process_href),
        process_uuid: strip(&values.process_uuid),
        role_name: strip(&values.role_name),
        role_href: strip(&values.role_href),
        service_name: strip(&values.service_name),
        service_type: strip(&values.service_type),
    };
    let collaboration_role = build_collaboration_role(document, &stripped);
    // Schema order of PartyInfo: PartyId, PartyRef, CollaborationRole, Certificate, ...
    let mut index = 0;
    for (position, child) in lxml_children(document, party_info).into_iter().enumerate() {
        if ["PartyId", "PartyRef", "CollaborationRole"].iter().any(|name| document.is_named(child, CPA, name)) {
            index = position + 1;
        }
    }
    insert_indented(document, party_info, index, collaboration_role);
    Ok(collaboration_role)
}

/// Deletes a CollaborationRole. A partner must keep at least one CollaborationRole (schema): an Err when it is the
/// last CollaborationRole of the partner.
pub fn delete_collaboration_role(document: &mut Document, collaboration_role: NodeId) -> Result<(), String> {
    let count = document
        .parent_element(collaboration_role)
        .map(|party_info| document.find_all(party_info, CPA, "CollaborationRole").count())
        .unwrap_or(0);
    if count <= 1 {
        return Err("A partner needs at least one CollaborationRole".into());
    }
    remove_element(document, collaboration_role);
    Ok(())
}
