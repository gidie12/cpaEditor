//! Fixes for validation errors. A reference (IDREF) to an id that does not exist can point at an existing id of the
//! same kind, or be removed where the schema allows it.

use crate::xml::{Document, NodeId};
use crate::CPA_NAMESPACE;

/// Where an IDREF value is: an attribute (local name) or the text of the element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefPlace {
    Attribute(String),
    Text,
}

/// One reference to an id that does not exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokenRef {
    pub element: NodeId,
    pub place: RefPlace,
    pub value: String,
    /// Where it is, e.g. `ChannelId in CanSend` or `AnchorCertificateRef in SecurityDetails 'X'`.
    pub description: String,
}

/// All references with the same missing value, with the ids it could point at instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokenIdref {
    pub value: String,
    /// What kind of id it should be, e.g. `channelId of a DeliveryChannel`.
    pub kind: String,
    pub references: Vec<BrokenRef>,
    /// Existing ids of the same kind, the most similar first.
    pub candidates: Vec<String>,
    /// Every reference may be removed: the schema does not require it.
    pub removable: bool,
}

type Target = (&'static [&'static str], &'static str, &'static str);

/// The elements and ID attribute that a reference points at, and a description of that kind of id.
fn target(element: &str, place: &RefPlace) -> Option<Target> {
    let name = match place {
        RefPlace::Attribute(name) => name.as_str(),
        RefPlace::Text => element,
    };
    Some(match name {
        "channelId" | "defaultMshChannelId" | "ChannelId" => (&["DeliveryChannel"], "channelId", "channelId of a DeliveryChannel"),
        "transportId" => (&["Transport"], "transportId", "transportId of a Transport"),
        "docExchangeId" => (&["DocExchange"], "docExchangeId", "docExchangeId of a DocExchange"),
        "certId" => (&["Certificate"], "certId", "certId of a Certificate"),
        "securityId" => (&["SecurityDetails"], "securityId", "securityId of SecurityDetails"),
        "packageId" | "defaultMshPackageId" => (&["Packaging"], "id", "id of a Packaging"),
        "idref" => (&["SimplePart", "Composite", "Encapsulation"], "id", "id of a SimplePart or Composite"),
        "OtherPartyActionBinding" => (&["ThisPartyActionBinding"], "id", "id of a ThisPartyActionBinding"),
        _ => return None,
    })
}

/// Whether an attribute is the ID of its element rather than a reference.
fn is_id(element: &str, attribute: &str) -> bool {
    crate::validation::ID_ATTRIBUTES
        .iter()
        .any(|(owner, name)| *name == attribute && owner.is_none_or(|owner| owner == element))
}

/// The edit distance between two strings, to rank similar ids.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            current.push((previous[j] + cost).min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

/// Whether the schema allows a reference to be removed.
fn removable(doc: &Document, reference: &BrokenRef) -> bool {
    ["AnchorCertificateRef", "ClientCertificateRef", "ServerSecurityDetailsRef", "ClientSecurityDetailsRef"]
        .iter()
        .any(|name| doc.is_named(reference.element, Some(CPA_NAMESPACE), name))
}

/// Every reference to an id that does not exist, grouped by the missing value, in document order.
pub fn broken_idrefs(doc: &Document) -> Vec<BrokenIdref> {
    let Some(root) = doc.root() else { return Vec::new() };
    let mut ids = std::collections::HashSet::new();
    let mut references: Vec<(String, BrokenRef)> = Vec::new();
    for node in doc.descendants(root) {
        let Some(element) = doc.element(node) else { continue };
        if element.name.namespace.as_deref() != Some(CPA_NAMESPACE) {
            continue;
        }
        let local = element.name.local.clone();
        for attribute in &element.attributes {
            if attribute.name.namespace.as_deref() != Some(CPA_NAMESPACE) {
                continue;
            }
            let value = attribute.value.trim().to_string();
            if is_id(&local, &attribute.name.local) {
                ids.insert(value);
            } else if crate::validation::IDREF_ATTRIBUTES.contains(&attribute.name.local.as_str()) {
                let place = RefPlace::Attribute(attribute.name.local.clone());
                let description = crate::certificates::describe_reference(doc, node);
                references.push((local.clone(), BrokenRef { element: node, place, value, description }));
            }
        }
        if crate::validation::IDREF_ELEMENTS.contains(&local.as_str()) {
            let description = crate::certificates::describe_reference(doc, node);
            let value = doc.text_trimmed(node);
            references.push((local, BrokenRef { element: node, place: RefPlace::Text, value, description }));
        }
    }
    let mut groups: Vec<BrokenIdref> = Vec::new();
    for (element, reference) in references {
        if ids.contains(&reference.value) {
            continue;
        }
        if let Some(group) = groups.iter_mut().find(|g| g.value == reference.value) {
            group.references.push(reference);
            continue;
        }
        let (kind, mut candidates) = match target(&element, &reference.place) {
            Some((owners, attribute, kind)) => {
                let candidates: Vec<String> = doc
                    .descendants(root)
                    .filter(|&n| owners.iter().any(|owner| doc.is_named(n, Some(CPA_NAMESPACE), owner)))
                    .filter_map(|n| doc.attribute(n, Some(CPA_NAMESPACE), attribute).map(|v| v.trim().to_string()))
                    .collect();
                (kind.to_string(), candidates)
            }
            None => ("id".to_string(), Vec::new()),
        };
        // A trust anchor: the certIds its TrustAnchors already has add nothing
        if doc.is_named(reference.element, Some(CPA_NAMESPACE), "AnchorCertificateRef") {
            if let Some(anchors) = doc.parent_element(reference.element) {
                let present: Vec<String> = doc
                    .find_all(anchors, Some(CPA_NAMESPACE), "AnchorCertificateRef")
                    .filter_map(|other| doc.attribute(other, Some(CPA_NAMESPACE), "certId").map(|v| v.trim().to_string()))
                    .collect();
                candidates.retain(|candidate| !present.contains(candidate));
            }
        }
        candidates.sort_by_key(|candidate| (distance(candidate, &reference.value), candidate.clone()));
        candidates.dedup();
        let value = reference.value.clone();
        groups.push(BrokenIdref { value, kind, references: vec![reference], candidates, removable: false });
    }
    for group in &mut groups {
        group.removable = group.references.iter().all(|reference| removable(doc, reference));
    }
    groups
}

/// Points every reference of a broken IDREF at another id.
pub fn repoint(doc: &mut Document, broken: &BrokenIdref, id: &str) {
    for reference in &broken.references {
        match &reference.place {
            RefPlace::Attribute(name) => doc.set_attribute(reference.element, Some(CPA_NAMESPACE), name, id),
            RefPlace::Text => {
                // The whitespace around the value stays
                let text = doc.text(reference.element).unwrap_or_default();
                let new = if text.trim().is_empty() { id.to_string() } else { text.replacen(text.trim(), id, 1) };
                doc.set_text(reference.element, Some(&new));
            }
        }
    }
}

/// Removes every reference of a broken IDREF; only when [`BrokenIdref::removable`]. The last trust anchor takes its
/// TrustAnchors with it.
pub fn remove(doc: &mut Document, broken: &BrokenIdref) -> Result<(), String> {
    if !broken.removable {
        return Err(format!("The references to {} are required; point them at another id", broken.value));
    }
    for reference in &broken.references {
        if doc.is_named(reference.element, Some(CPA_NAMESPACE), "AnchorCertificateRef")
            || doc.is_named(reference.element, Some(CPA_NAMESPACE), "ClientCertificateRef")
        {
            crate::certificates::remove_reference(doc, reference.element)?;
        } else {
            crate::certificates::remove_element_with_indentation(doc, reference.element);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::distance;

    #[test]
    fn distance_counts_edits() {
        assert_eq!(distance("BROKER_ClientCerts", "BROKER_ClientCert"), 1);
        assert_eq!(distance("abc", "abc"), 0);
        assert_eq!(distance("", "ab"), 2);
    }
}
