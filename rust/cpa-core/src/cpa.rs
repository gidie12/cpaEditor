//! The CPA model without user interface: the general fields (CPA id, status, parties, start and end date), the
//! comments, the transports, cert ids and security ids of a party, and the changes the transport tab and the XML
//! editor make. Port of `helper_classes/cpaParser.py`, `cpaCreator.py` and the XML logic of `comment_tab.py`,
//! `transport_tab.py`, `general_tab.py` and `xml_editor_tab.py`.

use crate::dates::now_cpa_date;
use crate::xml::{Document, NodeId, NodeKind, XML_NAMESPACE};
use crate::CPA_NAMESPACE;

/// The values of the status of a CPA allowed by the schema (`statusValue.type`).
pub const CPA_STATUS_OPTIONS: [&str; 3] = ["proposed", "agreed", "signed"];

/// lxml's message for a text or attribute value with characters XML cannot hold.
pub const XML_INCOMPATIBLE_MESSAGE: &str =
    "All strings must be XML compatible: Unicode or ASCII, no NULL bytes or control characters";

// ----------------------------------------------------------------------------------------------------------------
// General fields
// ----------------------------------------------------------------------------------------------------------------

/// The PartyInfo elements directly below the root, in document order (normally two: partner A and partner B).
pub fn party_infos(doc: &Document) -> Vec<NodeId> {
    match doc.root() {
        Some(root) => doc.find_all(root, Some(CPA_NAMESPACE), "PartyInfo").collect(),
        None => Vec::new(),
    }
}

/// The general fields of the CPA as the General tab shows them. A value that is missing is empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeneralFields {
    pub cpa_id: String,
    pub status: String,
    pub party_names: [String; 2],
    pub party_ids: [String; 2],
    pub party_id_types: [String; 2],
    pub start_date: String,
    pub end_date: String,
}

/// A field of the General tab. The index of the party fields is 0 for partner A and 1 for partner B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneralField {
    CpaId,
    Status,
    PartyName(usize),
    PartyId(usize),
    PartyIdType(usize),
    StartDate,
    EndDate,
}

/// Reads the general fields like `CPAParser` does.
///
/// The party values are read per PartyInfo in the order PartyId text, PartyId type, partyName; the first value
/// that is missing stops the reading of all parties. A party value is only shown when exactly two were read, so a
/// PartyInfo without PartyId also empties the fields of the other partner (as in the Python version).
pub fn read_general(doc: &Document) -> GeneralFields {
    let mut fields = GeneralFields::default();
    let Some(root) = doc.root() else { return fields };
    fields.cpa_id = doc.attribute(root, Some(CPA_NAMESPACE), "cpaid").unwrap_or_default().to_string();
    fields.status = doc
        .find(root, Some(CPA_NAMESPACE), "Status")
        .and_then(|status| doc.attribute(status, Some(CPA_NAMESPACE), "value"))
        .unwrap_or_default()
        .to_string();

    let mut ids: Vec<String> = Vec::new();
    let mut types: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for party_info in doc.find_all(root, Some(CPA_NAMESPACE), "PartyInfo") {
        let Some(party_id) = doc.find(party_info, Some(CPA_NAMESPACE), "PartyId") else { break };
        ids.push(doc.text(party_id).unwrap_or_default());
        let Some(party_id_type) = doc.attribute(party_id, Some(CPA_NAMESPACE), "type") else { break };
        types.push(party_id_type.to_string());
        let Some(name) = doc.attribute(party_info, Some(CPA_NAMESPACE), "partyName") else { break };
        names.push(name.to_string());
    }
    let pair = |values: Vec<String>| -> [String; 2] { <[String; 2]>::try_from(values).unwrap_or_default() };
    fields.party_ids = pair(ids);
    fields.party_id_types = pair(types);
    fields.party_names = pair(names);
    let first_text = |local: &str| -> String {
        doc.find(root, Some(CPA_NAMESPACE), local).and_then(|element| doc.text(element)).unwrap_or_default()
    };
    fields.start_date = first_text("Start");
    fields.end_date = first_text("End");
    fields
}

/// The PartyId elements of a partner after the first one, as (id, type). The General tab edits the first
/// PartyId; a party can be known under more ids (e.g. an OIN and a KvK number), which are listed below it.
pub fn other_party_ids(doc: &Document, party_index: usize) -> Vec<(String, String)> {
    let Some(&party_info) = party_infos(doc).get(party_index) else { return Vec::new() };
    doc.find_all(party_info, Some(CPA_NAMESPACE), "PartyId")
        .skip(1)
        .map(|party_id| {
            let id = doc.text(party_id).unwrap_or_default().trim().to_string();
            let kind = doc.attribute(party_id, Some(CPA_NAMESPACE), "type").unwrap_or_default().to_string();
            (id, kind)
        })
        .collect()
}

/// Changes a general field like `CPACreator.apply_changes` does.
///
/// Errors (the Python version printed them and left the CPA unchanged): fewer than two PartyInfo elements
/// (`list index out of range`), a missing Status, PartyId, Start or End element, a party index other than 0 or 1,
/// or a value with characters XML cannot hold.
pub fn set_general(doc: &mut Document, field: GeneralField, value: &str) -> Result<(), String> {
    let root = doc.root().ok_or_else(|| "No CPA loaded".to_string())?;
    let parties = party_infos(doc);
    if parties.len() < 2 {
        return Err("list index out of range".to_string());
    }
    check_xml_text(value)?;
    let party = |index: usize| -> Result<NodeId, String> {
        parties.get(index).copied().filter(|_| index < 2).ok_or_else(|| "list index out of range".to_string())
    };
    let missing_attrib = || "'NoneType' object has no attribute 'attrib'".to_string();
    let missing_text = || "'NoneType' object has no attribute 'text'".to_string();
    match field {
        GeneralField::CpaId => doc.set_attribute(root, Some(CPA_NAMESPACE), "cpaid", value),
        GeneralField::Status => {
            let status = doc.find(root, Some(CPA_NAMESPACE), "Status").ok_or_else(missing_attrib)?;
            doc.set_attribute(status, Some(CPA_NAMESPACE), "value", value);
        }
        GeneralField::PartyName(index) => {
            let party_info = party(index)?;
            doc.set_attribute(party_info, Some(CPA_NAMESPACE), "partyName", value);
        }
        GeneralField::PartyId(index) => {
            let party_info = party(index)?;
            let party_id = doc.find(party_info, Some(CPA_NAMESPACE), "PartyId").ok_or_else(missing_text)?;
            doc.set_text(party_id, Some(value));
        }
        GeneralField::PartyIdType(index) => {
            let party_info = party(index)?;
            let party_id = doc.find(party_info, Some(CPA_NAMESPACE), "PartyId").ok_or_else(missing_attrib)?;
            doc.set_attribute(party_id, Some(CPA_NAMESPACE), "type", value);
        }
        GeneralField::StartDate | GeneralField::EndDate => {
            let local = if field == GeneralField::StartDate { "Start" } else { "End" };
            let element = doc.find(root, Some(CPA_NAMESPACE), local).ok_or_else(missing_text)?;
            doc.set_text(element, Some(value));
        }
    }
    Ok(())
}

/// Sets the start date to the current date and time (UTC) and returns that date (the "Set to now" button).
pub fn set_start_date_to_now(doc: &mut Document) -> Result<String, String> {
    let start_date = now_cpa_date();
    set_general(doc, GeneralField::StartDate, &start_date)?;
    Ok(start_date)
}

// ----------------------------------------------------------------------------------------------------------------
// Comments
// ----------------------------------------------------------------------------------------------------------------

/// The Comment elements directly below the root.
pub fn comments(doc: &Document) -> Vec<NodeId> {
    match doc.root() {
        Some(root) => doc.find_all(root, Some(CPA_NAMESPACE), "Comment").collect(),
        None => Vec::new(),
    }
}

/// Adds a Comment at the end of the root element, where the schema allows it (`Comment*` is the last particle of
/// CollaborationProtocolAgreement). `lang` is written as `xml:lang`; an empty `lang` writes no attribute.
///
/// The Python version wrote the attribute `{xml}type="nl-NL"`, which the schema does not allow; this writes
/// `xml:lang` instead. The comment tab only adds a comment with text, the caller checks that.
pub fn add_comment(doc: &mut Document, text: &str, lang: &str) -> NodeId {
    let root = doc.root().expect("a CPA has a root element");
    let comment = doc.create_element(Some(CPA_NAMESPACE), "Comment");
    doc.set_text(comment, Some(&xml_safe(text)));
    if !lang.is_empty() {
        doc.set_attribute(comment, Some(XML_NAMESPACE), "lang", &xml_safe(lang));
    }
    doc.append_child(root, comment);
    comment
}

/// Changes the text of a comment (lxml `comment.text = text`).
pub fn set_comment(doc: &mut Document, comment: NodeId, text: &str) -> Result<(), String> {
    set_element_text(doc, comment, text)
}

/// Removes a comment from the root element; its tail text goes with it (lxml `root.remove(comment)`).
pub fn delete_comment(doc: &mut Document, comment: NodeId) -> Result<(), String> {
    let root = doc.root().ok_or_else(|| "No CPA loaded".to_string())?;
    if doc.parent(comment) != Some(root) {
        return Err("Element is not a child of this node.".to_string());
    }
    doc.remove_with_tail(comment);
    Ok(())
}

// ----------------------------------------------------------------------------------------------------------------
// Transports, certificates and security details of a party
// ----------------------------------------------------------------------------------------------------------------

/// All Transport elements below a PartyInfo, at any depth, in document order (lxml `findall('.//Transport')`).
pub fn transports(doc: &Document, party_info: NodeId) -> Vec<NodeId> {
    doc.iter_named(party_info, Some(CPA_NAMESPACE), "Transport").filter(|&node| node != party_info).collect()
}

/// The certId of every Certificate directly below a PartyInfo. A Certificate without certId is skipped (the
/// Python version stopped with a KeyError).
pub fn cert_ids(doc: &Document, party_info: NodeId) -> Vec<String> {
    doc.find_all(party_info, Some(CPA_NAMESPACE), "Certificate")
        .filter_map(|certificate| doc.attribute(certificate, Some(CPA_NAMESPACE), "certId"))
        .map(str::to_string)
        .collect()
}

/// The securityId of every SecurityDetails directly below a PartyInfo. One without securityId is skipped.
pub fn security_ids(doc: &Document, party_info: NodeId) -> Vec<String> {
    doc.find_all(party_info, Some(CPA_NAMESPACE), "SecurityDetails")
        .filter_map(|details| doc.attribute(details, Some(CPA_NAMESPACE), "securityId"))
        .map(str::to_string)
        .collect()
}

/// The transport tab's "Save Changes" for an attribute row: sets the attribute `tns:<local>` (in the CPA
/// namespace, whatever the namespace of the shown attribute was, as the Python version did).
pub fn set_cpa_attribute(doc: &mut Document, element: NodeId, local: &str, value: &str) -> Result<(), String> {
    check_xml_text(value)?;
    doc.set_attribute(element, Some(CPA_NAMESPACE), local, value);
    Ok(())
}

/// Replaces the text of an element before its first child (lxml `element.text = value`); used by the transport
/// tab, the XML editor and the comment tab.
pub fn set_element_text(doc: &mut Document, element: NodeId, value: &str) -> Result<(), String> {
    if !doc.is_element(element) {
        return Err("Not an element".to_string());
    }
    check_xml_text(value)?;
    doc.set_text(element, Some(value));
    Ok(())
}

/// Clones an element: a deep copy, with a copy of its tail, directly after the element and its tail (lxml
/// `parent.insert(parent.index(element) + 1, copy.deepcopy(element))`). Returns the copy; None for the root.
///
/// The Python transport tab meant to do this, but inserted the element itself again at its own position, which
/// changed nothing.
pub fn clone_element(doc: &mut Document, element: NodeId) -> Option<NodeId> {
    let parent = doc.parent_element(element)?;
    let tail = doc.tail(element);
    let copy = doc.deep_copy(element);
    insert_after_tail(doc, parent, element, copy);
    if let Some(tail) = tail {
        doc.set_tail(copy, Some(&tail));
    }
    strip_redundant_namespaces(doc, copy);
    Some(copy)
}

// ----------------------------------------------------------------------------------------------------------------
// XML editor
// ----------------------------------------------------------------------------------------------------------------

/// The XML editor's change or addition of an attribute. `name` is in Clark notation as the editor shows it:
/// `{namespace}local`, or `local` without namespace.
pub fn set_attribute_by_name(doc: &mut Document, element: NodeId, name: &str, value: &str) -> Result<(), String> {
    let (namespace, local) = parse_attribute_name(name)?;
    check_xml_text(value)?;
    doc.set_attribute(element, namespace.as_deref(), &local, value);
    Ok(())
}

/// The XML editor's deletion of an attribute (`name` in Clark notation). Error (Python KeyError) when the
/// element has no such attribute.
pub fn delete_attribute_by_name(doc: &mut Document, element: NodeId, name: &str) -> Result<(), String> {
    let (namespace, local) = parse_attribute_name(name)?;
    if doc.remove_attribute(element, namespace.as_deref(), &local) {
        Ok(())
    } else {
        Err(python_repr(name))
    }
}

/// The XML editor's deletion of an element: removed from its parent with its tail (lxml `parent.remove`).
/// Returns false for the root element, which the editor cannot delete.
pub fn delete_element(doc: &mut Document, element: NodeId) -> bool {
    if doc.parent_element(element).is_none() {
        return false;
    }
    doc.remove_with_tail(element);
    true
}

/// The XML editor's addition of an element `name` with text `value` to `parent`. The element gets the namespace of
/// the root element. It is placed directly after the last child with the same name, otherwise at the end.
///
/// Errors with lxml's messages for a name that is not an NCName: `Empty tag name`, `Invalid tag name '...'`.
///
/// The Python version created the element with `xml.etree.ElementTree` and failed with a TypeError when it
/// inserted it into the lxml tree; this does what that code intended.
pub fn add_element(doc: &mut Document, parent: NodeId, name: &str, value: &str) -> Result<NodeId, String> {
    let root = doc.root().ok_or_else(|| "No CPA loaded".to_string())?;
    let namespace = doc.name(root).and_then(|name| name.namespace.clone());
    if name.is_empty() {
        return Err("Empty tag name".to_string());
    }
    if !is_ncname(name) {
        return Err(format!("Invalid tag name {}", python_repr(name)));
    }
    check_xml_text(value)?;
    let element = doc.create_element(namespace.as_deref(), name);
    doc.set_text(element, Some(value));
    let last_same = doc.find_all(parent, namespace.as_deref(), name).last();
    match last_same {
        Some(last) => insert_after_tail(doc, parent, last, element),
        None => doc.append_child(parent, element),
    }
    Ok(element)
}

/// Splits a name in Clark notation into namespace and local name, with lxml's checks for attribute names.
pub fn parse_attribute_name(name: &str) -> Result<(Option<String>, String), String> {
    let (namespace, local) = match name.strip_prefix('{') {
        Some(rest) => match rest.find('}') {
            Some(end) => {
                let namespace = &rest[..end];
                (if namespace.is_empty() { None } else { Some(namespace.to_string()) }, &rest[end + 1..])
            }
            None => return Err(format!("Invalid tag name {}", python_repr(name))),
        },
        None => (None, name),
    };
    if local.is_empty() {
        return Err("Empty tag name".to_string());
    }
    if !is_ncname(local) {
        return Err(format!("Invalid attribute name {}", python_repr(local)));
    }
    Ok((namespace, local.to_string()))
}

// ----------------------------------------------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------------------------------------------

/// Errors with lxml's message when a value has characters XML 1.0 cannot hold (control characters other than tab,
/// line feed and carriage return, U+FFFE and U+FFFF).
pub fn check_xml_text(value: &str) -> Result<(), String> {
    if value.chars().all(is_xml_char) {
        Ok(())
    } else {
        Err(XML_INCOMPATIBLE_MESSAGE.to_string())
    }
}

fn is_xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{fffe}' && c != '\u{ffff}')
}

/// The value without characters XML cannot hold (for functions without error result).
fn xml_safe(value: &str) -> String {
    value.chars().filter(|&c| is_xml_char(c)).collect()
}

/// Inserts `node` into `parent` directly after `reference` and its tail text, like lxml
/// `parent.insert(parent.index(reference) + 1, node)`: before the next sibling that is not text, or at the end.
pub fn insert_after_tail(doc: &mut Document, parent: NodeId, reference: NodeId, node: NodeId) {
    let siblings = doc.children(parent);
    let next = siblings
        .iter()
        .position(|&sibling| sibling == reference)
        .and_then(|position| {
            siblings[position + 1..].iter().copied().find(|&s| !matches!(doc.kind(s), NodeKind::Text(_)))
        });
    match next {
        Some(next) => doc.insert_before(next, node),
        None => doc.append_child(parent, node),
    }
}

/// Drops the namespace declarations of a node that was just moved into the tree when the parent already has a
/// declaration for the same namespace URI in scope, like lxml does when it moves an element into a document
/// (`_stripRedundantNamespaceDeclarations`). The element and its descendants then use the prefix in scope.
pub fn strip_redundant_namespaces(doc: &mut Document, node: NodeId) {
    let Some(parent) = doc.parent_element(node) else { return };
    let declarations = match doc.element(node) {
        Some(element) => element.namespaces.clone(),
        None => return,
    };
    let redundant: Vec<Option<String>> = declarations
        .iter()
        .filter(|declaration| uri_in_scope(doc, parent, &declaration.uri))
        .map(|declaration| declaration.prefix.clone())
        .collect();
    if redundant.is_empty() {
        return;
    }
    if let Some(element) = doc.element_mut(node) {
        element.namespaces.retain(|declaration| !redundant.contains(&declaration.prefix));
        for name in std::iter::once(&mut element.name).chain(element.attributes.iter_mut().map(|a| &mut a.name)) {
            if redundant.contains(&name.prefix) && name.namespace.is_some() {
                name.prefix = None;
            }
        }
    }
}

/// Whether a declaration of the URI (with a prefix that is not shadowed) is in scope at the element (libxml2
/// `xmlSearchNsByHref`).
fn uri_in_scope(doc: &Document, element: NodeId, uri: &str) -> bool {
    if uri == XML_NAMESPACE {
        return true;
    }
    let mut seen: Vec<Option<String>> = Vec::new();
    for ancestor in doc.ancestors(element) {
        let Some(element) = doc.element(ancestor) else { continue };
        for declaration in &element.namespaces {
            if seen.contains(&declaration.prefix) {
                continue;
            }
            seen.push(declaration.prefix.clone());
            if declaration.uri == uri {
                return true;
            }
        }
    }
    false
}

/// Whether a name is an XML NCName (a name without colon), as libxml2's `xmlValidateNCName` checks it. Outside
/// ASCII the Unicode letter and digit classes are used.
pub fn is_ncname(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    if !(first == '_' || first.is_alphabetic()) {
        return false;
    }
    chars.all(|c| {
        c == '_'
            || c == '-'
            || c == '.'
            || c.is_alphanumeric()
            || c == '\u{b7}'
            || ('\u{300}'..='\u{36f}').contains(&c)
            || ('\u{203f}'..='\u{2040}').contains(&c)
    })
}

/// Python's `repr()` of a string, as lxml puts it in its messages.
pub fn python_repr(value: &str) -> String {
    let quote = if value.contains('\'') && !value.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

// ----------------------------------------------------------------------------------------------------------------
// New CPA from the loaded one
// ----------------------------------------------------------------------------------------------------------------

/// What changes when a new CPA is made from the loaded one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewCpaOptions {
    pub cpa_id: String,
    pub start_date: String,
    pub end_date: String,
    /// Status `proposed`: a new CPA still has to be agreed.
    pub reset_status: bool,
    pub strip_certificates: bool,
}

/// The proposed CPAId of a new CPA: a version number at the end goes up by one (`..._2` becomes `..._3`, the
/// Digikoppeling convention ends with the version), otherwise `_2` is added.
pub fn next_cpa_id(cpa_id: &str) -> String {
    let digits = cpa_id.len() - cpa_id.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    if digits > 0 && digits < cpa_id.len() {
        let (stem, number) = cpa_id.split_at(cpa_id.len() - digits);
        if let Ok(value) = number.parse::<u64>() {
            // Keep leading zeros: 009 becomes 010
            return format!("{stem}{:0width$}", value + 1, width = digits);
        }
    }
    if cpa_id.is_empty() { String::new() } else { format!("{cpa_id}_2") }
}

/// The proposed options: the next CPAId, starting now, valid as long as the loaded CPA was (one year when its
/// dates cannot be read), status proposed, certificates kept.
pub fn new_cpa_defaults(doc: &Document) -> NewCpaOptions {
    let fields = read_general(doc);
    let now = chrono::Utc::now();
    let length = match (crate::dates::cpa_moment(&fields.start_date), crate::dates::cpa_moment(&fields.end_date)) {
        (Some(start), Some(end)) if end > start => end - start,
        _ => chrono::Duration::days(365),
    };
    NewCpaOptions {
        cpa_id: next_cpa_id(fields.cpa_id.trim()),
        start_date: crate::dates::format_cpa_date(now),
        end_date: crate::dates::format_cpa_date(now + length),
        reset_status: true,
        strip_certificates: false,
    }
}

/// What is wrong with the options, per field; empty when a new CPA can be made.
pub fn check_new_cpa_options(doc: &Document, options: &NewCpaOptions) -> Vec<(GeneralField, String)> {
    let mut errors = Vec::new();
    let cpa_id = options.cpa_id.trim();
    if cpa_id.is_empty() {
        errors.push((GeneralField::CpaId, "Enter a CPAId".to_string()));
    } else if cpa_id == read_general(doc).cpa_id.trim() {
        errors.push((GeneralField::CpaId, "A new CPA needs another CPAId than the loaded CPA".to_string()));
    } else if let Err(e) = check_xml_text(cpa_id) {
        errors.push((GeneralField::CpaId, e));
    }
    let start = crate::dates::cpa_moment(&options.start_date);
    let end = crate::dates::cpa_moment(&options.end_date);
    if start.is_none() {
        errors.push((GeneralField::StartDate, crate::dates::INVALID_DATE_MESSAGE.to_string()));
    }
    if end.is_none() {
        errors.push((GeneralField::EndDate, crate::dates::INVALID_DATE_MESSAGE.to_string()));
    }
    if let (Some(start), Some(end)) = (start, end) {
        if end <= start {
            errors.push((GeneralField::EndDate, "The end date must be after the start date".to_string()));
        }
    }
    errors
}

/// Makes a new CPA from the loaded one: a copy with the new CPAId and dates, optionally status `proposed` and
/// without certificates. The loaded CPA is not changed.
pub fn new_cpa_from(
    doc: &Document,
    options: &NewCpaOptions,
) -> Result<(Document, Option<crate::certificates::StripReport>), String> {
    if let Some((_, error)) = check_new_cpa_options(doc, options).into_iter().next() {
        return Err(error);
    }
    let mut new = doc.clone();
    set_general(&mut new, GeneralField::CpaId, options.cpa_id.trim())?;
    set_general(&mut new, GeneralField::StartDate, options.start_date.trim())?;
    set_general(&mut new, GeneralField::EndDate, options.end_date.trim())?;
    if options.reset_status {
        set_general(&mut new, GeneralField::Status, "proposed")?;
    }
    let report = options.strip_certificates.then(|| crate::certificates::strip_certificates(&mut new));
    Ok((new, report))
}

/// The file name for a CPAId: characters a file name cannot hold become an underscore.
pub fn cpa_file_name(cpa_id: &str) -> String {
    let name: String =
        cpa_id.trim().chars().map(|c| if r#"\/:*?"<>|"#.contains(c) || c.is_control() { '_' } else { c }).collect();
    format!("{name}.xml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_repr_quotes_like_python() {
        assert_eq!(python_repr("with space"), "'with space'");
        assert_eq!(python_repr("it's"), "\"it's\"");
        assert_eq!(python_repr("a\tb"), "'a\\tb'");
    }

    #[test]
    fn ncname_rules() {
        assert!(is_ncname("Transport_1.a-b"));
        assert!(!is_ncname("1a"));
        assert!(!is_ncname("a:b"));
        assert!(!is_ncname("with space"));
        assert!(!is_ncname(""));
    }
}
