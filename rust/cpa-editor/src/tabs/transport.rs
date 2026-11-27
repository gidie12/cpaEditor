//! Transport tab (transport_tab.py): per partner the Transport elements with their child elements and attributes.
//! A value or an attribute is edited in the detail panel; a certId or securityId is chosen from the certificates
//! or security details of the same partner. Clone transport (a copy with a unique transportId) is off for now.

use std::collections::{HashMap, HashSet};

use cpa_core::cpa;
use cpa_core::xml::{Document, NodeId};
use cpa_core::CPA_NAMESPACE;
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme::{self, PARTNER_COLORS};
use crate::tree_table::{RowKey, TreeEvents, TreeRow, TreeTable};
use crate::widgets::combo;

/// Clone transport is off for now: it does not update the references to the new transportId yet (TODO.md).
const CLONE_TRANSPORT: bool = false;

/// Width of the label column of the detail form.
const LABEL_WIDTH: f32 = 110.0;
/// Attributes whose value refers to a Certificate or SecurityDetails of the same partner.
const REFERENCE_ATTRIBUTES: [&str; 2] = ["certId", "securityId"];

#[derive(Clone, Debug, PartialEq, Eq)]
enum RowKind {
    Partner(NodeId),
    Element(NodeId),
    /// An attribute of the element, by local name (the tab writes it in the CPA namespace, as Python did).
    Attribute(NodeId, String),
}

impl RowKind {
    fn element(&self) -> NodeId {
        match self {
            RowKind::Partner(node) | RowKind::Element(node) | RowKind::Attribute(node, _) => *node,
        }
    }
}

fn element_key(node: NodeId) -> RowKey {
    (node.index() as RowKey) << 12
}

fn attribute_key(node: NodeId, position: usize) -> RowKey {
    element_key(node) | (position as RowKey + 1)
}

/// The value being edited in the detail panel.
#[derive(Default)]
struct Detail {
    target: Option<RowKind>,
    name: String,
    /// The element the value belongs to.
    element_name: String,
    partner: String,
    transport_id: String,
    value: String,
    /// The value in the CPA; Save changes is only needed when the field differs.
    original: String,
    /// The certIds or securityIds of the partner for a reference attribute.
    options: Option<Vec<String>>,
    /// Per certId: the common name and expiry of its leaf certificate, shown under the drop-down list.
    option_hints: std::collections::HashMap<String, (String, bool)>,
    /// The current value of a reference attribute when it is not one of the options (Python then shows the first).
    unknown_value: Option<String>,
    error: Option<String>,
}

pub struct TransportTab {
    tree: TreeTable,
    rows: Vec<TreeRow>,
    kinds: HashMap<RowKey, RowKind>,
    built_for: Option<u64>,
    generation: u64,
    detail: Detail,
    transport_count: usize,
    focus_value: bool,
}

impl Default for TransportTab {
    fn default() -> Self {
        TransportTab {
            tree: TreeTable::new("transport", &[("Element / attribute", 380.0), ("Value", 420.0)]),
            rows: Vec::new(),
            kinds: HashMap::new(),
            built_for: None,
            generation: 0,
            detail: Detail::default(),
            transport_count: 0,
            focus_value: false,
        }
    }
}

impl TransportTab {
    fn build_rows(&mut self, document: &Document) {
        self.rows.clear();
        self.kinds.clear();
        self.transport_count = 0;
        for (index, party_info) in cpa::party_infos(document).into_iter().enumerate() {
            let name = document.attribute(party_info, Some(CPA_NAMESPACE), "partyName").unwrap_or_default();
            let transports = cpa::transports(document, party_info);
            self.transport_count += transports.len();
            let count = match transports.len() {
                0 => "no transports".to_string(),
                1 => "1 transport".to_string(),
                n => format!("{n} transports"),
            };
            let row = TreeRow::new(element_key(party_info), 0, vec![format!("Partner {name}"), String::new()])
                .header(PARTNER_COLORS[index % PARTNER_COLORS.len()])
                .secondary(count);
            self.rows.push(row);
            self.kinds.insert(element_key(party_info), RowKind::Partner(party_info));
            for transport in transports {
                self.insert_element(document, transport, 1);
            }
        }
        if self.tree.selected.is_some_and(|key| !self.kinds.contains_key(&key)) {
            self.tree.selected = None;
        }
    }

    /// An element row, then its attributes, then its child elements (as populate_tree does).
    fn insert_element(&mut self, document: &Document, element: NodeId, depth: usize) {
        let name = document.local_name(element).to_string();
        let mut row = TreeRow::new(element_key(element), depth, vec![name.clone(), document.text_trimmed(element)]);
        if name == "Transport" {
            row = row.strong();
            if let Some(id) = document.attribute(element, Some(CPA_NAMESPACE), "transportId") {
                row = row.secondary(id.to_string());
            }
        }
        self.rows.push(row);
        self.kinds.insert(element_key(element), RowKind::Element(element));
        for (position, attribute) in document.attributes(element).iter().enumerate() {
            let key = attribute_key(element, position);
            let cells = vec![format!("@{}", attribute.name.local), attribute.value.clone()];
            self.rows.push(TreeRow::new(key, depth + 1, cells));
            self.kinds.insert(key, RowKind::Attribute(element, attribute.name.local.clone()));
        }
        let children: Vec<NodeId> = document.child_elements(element).collect();
        for child in children {
            self.insert_element(document, child, depth + 1);
        }
    }

    fn selected(&self) -> Option<RowKind> {
        self.tree.selected.and_then(|key| self.kinds.get(&key).cloned())
    }

    /// The Transport that contains the selected row.
    fn selected_transport(&self, document: &Document) -> Option<NodeId> {
        match self.selected()? {
            RowKind::Partner(_) => None,
            kind => document.ancestor_named(kind.element(), Some(CPA_NAMESPACE), "Transport"),
        }
    }

    /// Fills the detail panel from the selected row (on_treeview_select). Only rows with a value can be edited.
    fn load_detail(&mut self, document: &Document) {
        self.detail = Detail::default();
        let Some(kind) = self.selected() else { return };
        let element = kind.element();
        if !document.is_attached(element) {
            return;
        }
        let (name, value) = match &kind {
            RowKind::Partner(_) => return,
            RowKind::Element(_) => {
                let text = document.text_trimmed(element);
                if text.is_empty() {
                    return;
                }
                (document.local_name(element).to_string(), text)
            }
            RowKind::Attribute(_, local) => {
                let value = document
                    .attributes(element)
                    .iter()
                    .find(|attribute| &attribute.name.local == local)
                    .map(|attribute| attribute.value.clone())
                    .unwrap_or_default();
                (local.clone(), value)
            }
        };
        let party_info = document.ancestor_named(element, Some(CPA_NAMESPACE), "PartyInfo");
        let transport = document.ancestor_named(element, Some(CPA_NAMESPACE), "Transport");
        self.detail = Detail {
            target: Some(kind.clone()),
            element_name: document.local_name(element).to_string(),
            partner: party_info
                .and_then(|p| document.attribute(p, Some(CPA_NAMESPACE), "partyName"))
                .unwrap_or_default()
                .to_string(),
            transport_id: transport
                .and_then(|t| document.attribute(t, Some(CPA_NAMESPACE), "transportId"))
                .unwrap_or_default()
                .to_string(),
            value: value.clone(),
            original: value.clone(),
            name,
            ..Default::default()
        };
        if matches!(kind, RowKind::Attribute(..)) && REFERENCE_ATTRIBUTES.contains(&self.detail.name.as_str()) {
            let options = match (party_info, self.detail.name.as_str()) {
                (Some(party), "certId") => cpa::cert_ids(document, party),
                (Some(party), _) => cpa::security_ids(document, party),
                (None, _) => {
                    self.detail.error = Some(format!("Could not determine partner name for {}", self.detail.name));
                    Vec::new()
                }
            };
            if !options.is_empty() && !options.contains(&value) {
                self.detail.unknown_value = Some(value);
                self.detail.value = options[0].clone();
            }
            if let (Some(party), "certId") = (party_info, self.detail.name.as_str()) {
                let now = chrono::Utc::now();
                for entry in cpa_core::certificates::certificate_entries(document, party).0 {
                    let leaf = cpa_core::certificates::key_info_certificates(document, entry.key_info, now).0;
                    if let Some(leaf) = leaf.first() {
                        let expired = leaf.not_after < now;
                        let state = if expired { "expired" } else { "expires" };
                        let hint = format!("{} · {state} {}", leaf.common_name, leaf.not_after.format("%Y-%m-%d"));
                        self.detail.option_hints.insert(entry.cert_id, (hint, expired));
                    }
                }
            }
            self.detail.options = Some(options);
        }
    }

    fn rebuild_and_select(&mut self, shared: &Shared, key: RowKey) {
        let Some(document) = shared.document.as_ref() else { return };
        self.build_rows(document);
        self.tree.select(Some(key), &self.rows);
        self.load_detail(document);
        self.built_for = Some(shared.revision);
    }

    /// Writes the value of the detail panel to the CPA (save_changes).
    fn save_changes(&mut self, shared: &mut Shared) {
        let (Some(target), Some(document)) = (self.detail.target.clone(), shared.document.as_mut()) else { return };
        if self.detail.value == self.detail.original && self.detail.unknown_value.is_none() {
            return;
        }
        let value = self.detail.value.clone();
        let result = match &target {
            RowKind::Attribute(element, local) => cpa::set_cpa_attribute(document, *element, local, &value),
            RowKind::Element(element) => cpa::set_element_text(document, *element, &value),
            RowKind::Partner(_) => return,
        };
        match result {
            Ok(()) => {
                shared.log.info(format!("Transport value changed: {} = {value}", self.detail.name));
                shared.changed();
                if let Some(key) = self.tree.selected {
                    self.rebuild_and_select(shared, key);
                }
            }
            Err(e) => {
                shared.log.error(format!("Error while changing element: {e}"));
                self.detail.error = Some(e);
            }
        }
    }

    /// Clones the Transport of the selection directly after it, with a transportId that is not used yet.
    fn clone_transport(&mut self, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else { return };
        let Some(transport) = self.selected_transport(document) else { return };
        let old_id = document.attribute(transport, Some(CPA_NAMESPACE), "transportId").map(str::to_string);
        let Some(copy) = cpa::clone_element(document, transport) else {
            shared.log.error("Error while cloning transport: the element has no parent");
            return;
        };
        match old_id {
            Some(old_id) => {
                let new_id = unique_id(document, &old_id);
                if let Err(e) = cpa::set_cpa_attribute(document, copy, "transportId", &new_id) {
                    shared.log.error(format!("Error while changing element: {e}"));
                }
                shared.log.info(format!(
                    "Transport '{old_id}' cloned as '{new_id}'. Delivery channels still refer to '{old_id}'."
                ));
            }
            None => shared.log.info("Transport cloned"),
        }
        shared.changed();
        self.rebuild_and_select(shared, element_key(copy));
    }

    fn toolbar(&mut self, ui: &mut Ui, document: &Document) -> bool {
        let mut clone = false;
        ui.horizontal(|ui| {
            let can_clone = self.selected_transport(document).is_some();
            clone = CLONE_TRANSPORT
                && ui
                .add_enabled(can_clone, egui::Button::new("Clone transport"))
                .on_hover_text("Copy the selected Transport, with a new transportId, directly after it")
                .on_disabled_hover_text("Select a Transport or one of its rows first")
                .clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let text = match self.transport_count {
                    1 => "1 transport".to_string(),
                    n => format!("{n} transports"),
                };
                ui.label(RichText::new(text).weak());
            });
        });
        clone
    }

    /// The field for the value: a drop-down list of the partner's ids for a reference, else a text field. Returns
    /// true when Enter was pressed in the text field.
    fn value_field(&mut self, ui: &mut Ui) -> bool {
        let mut save = false;
        match &self.detail.options {
            Some(options) if options.is_empty() => {
                ui.label(RichText::new(format!("No {}s available", self.detail.name)).weak());
            }
            Some(options) => {
                let options = options.clone();
                if combo(ui, "transport_value", &mut self.detail.value, &options, true) {
                    self.detail.error = None;
                }
                // Which certificate is behind the id, in its own grid row so "Value" stays on the line of the list
                if let Some((hint, expired)) = self.detail.option_hints.get(&self.detail.value) {
                    ui.end_row();
                    ui.label("");
                    let text = RichText::new(hint);
                    ui.label(if *expired { text.color(theme::error_color(ui)) } else { text.weak() });
                }
            }
            None => {
                let field =
                    egui::TextEdit::singleline(&mut self.detail.value).desired_width(ui.available_width());
                let response = ui.add(field);
                if std::mem::take(&mut self.focus_value) {
                    response.request_focus();
                }
                if response.changed() {
                    self.detail.error = None;
                }
                if response.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.detail.value = self.detail.original.clone();
                    }
                }
            }
        }
        save
    }

    /// The detail panel; returns true when the user asks to clone the transport.
    fn detail_panel(&mut self, ui: &mut Ui, shared: &mut Shared) -> bool {
        let mut clone = false;
        let Some(document) = shared.document.as_ref() else { return false };
        if self.detail.target.is_none() {
            match self.selected() {
                None => theme::empty_state(
                    ui,
                    "Nothing selected",
                    "Select a value or an attribute of a transport on the left to edit it.",
                    |_| {},
                ),
                Some(RowKind::Partner(party)) => {
                    let name = document.attribute(party, Some(CPA_NAMESPACE), "partyName").unwrap_or_default();
                    theme::empty_state(
                        ui,
                        &format!("Partner {name}"),
                        "Select a value or an attribute of one of its transports to edit it.",
                        |_| {},
                    );
                }
                Some(kind) => {
                    let element = kind.element();
                    let is_transport = document.is_named(element, Some(CPA_NAMESPACE), "Transport");
                    let heading = match document.attribute(element, Some(CPA_NAMESPACE), "transportId") {
                        Some(id) if is_transport => format!("Transport {id}"),
                        _ => document.local_name(element).to_string(),
                    };
                    theme::empty_state(
                        ui,
                        &heading,
                        "This element has no value of its own. Select one of its attributes or values to edit it.",
                        |ui| {
                            if is_transport && CLONE_TRANSPORT {
                                clone = ui.button("Clone transport").clicked();
                            }
                        },
                    );
                }
            }
            return clone;
        }

        let is_attribute = matches!(self.detail.target, Some(RowKind::Attribute(..)));
        ui.add_space(4.0);
        ui.label(RichText::new(&self.detail.name).heading().strong());
        let kind = if is_attribute { "Attribute of" } else { "Value of" };
        ui.label(RichText::new(format!("{kind} {}", self.detail.element_name)).weak());
        ui.add_space(10.0);

        let mut save = false;
        egui::Grid::new("transport_detail").num_columns(2).spacing([12.0, 8.0]).min_col_width(LABEL_WIDTH).show(
            ui,
            |ui| {
                ui.label("Partner");
                ui.label(&self.detail.partner);
                ui.end_row();
                ui.label("Transport");
                ui.label(&self.detail.transport_id);
                ui.end_row();
                ui.label("Element");
                ui.label(&self.detail.element_name);
                ui.end_row();
                // Not the name of the value: a long name would widen the label column and with it the panel
                ui.label("Value");
                save = self.value_field(ui);
                ui.end_row();
            },
        );

        if let Some(unknown) = &self.detail.unknown_value {
            ui.add_space(8.0);
            theme::notice(
                ui,
                &format!(
                    "'{unknown}' is not a {} of partner {}. Save changes replaces it with the value chosen above.",
                    self.detail.name, self.detail.partner
                ),
            );
        }
        if let Some(error) = &self.detail.error {
            ui.add_space(6.0);
            ui.label(RichText::new(error).color(theme::error_color(ui)));
        }
        ui.add_space(12.0);
        let changed = self.detail.value != self.detail.original || self.detail.unknown_value.is_some();
        let can_save = changed && self.detail.options.as_ref().is_none_or(|options| !options.is_empty());
        ui.horizontal(|ui| {
            save |= theme::action_button(ui, "Save changes", true, can_save).clicked();
            if ui.add_enabled(changed, egui::Button::new("Discard")).clicked() {
                if let Some(document) = shared.document.as_ref() {
                    self.load_detail(document);
                }
            }
            if !changed {
                ui.label(RichText::new("No changes").weak());
            }
        });
        if save && can_save {
            self.save_changes(shared);
        }
        clone
    }
}

/// `{id}_copy`, `{id}_copy2`, ... : the first that is not the value of any attribute in the document (an XML ID must
/// be unique in the whole document).
fn unique_id(document: &Document, id: &str) -> String {
    let used: HashSet<&str> = match document.root() {
        Some(root) => document
            .descendants(root)
            .flat_map(|node| document.attributes(node).iter().map(|attribute| attribute.value.as_str()))
            .collect(),
        None => HashSet::new(),
    };
    let mut candidate = format!("{id}_copy");
    let mut number = 2;
    while used.contains(candidate.as_str()) {
        candidate = format!("{id}_copy{number}");
        number += 1;
    }
    candidate
}

impl Tab for TransportTab {
    fn title(&self) -> &'static str {
        "Transport"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            *self = TransportTab { generation: shared.generation, ..Default::default() };
        }
        if shared.document.is_none() {
            no_cpa(ui, shared);
            return;
        }
        let Some(document) = shared.document.as_ref() else { return };
        if self.built_for != Some(shared.revision) {
            self.build_rows(document);
            self.load_detail(document);
            self.built_for = Some(shared.revision);
        }
        if self.rows.is_empty() {
            theme::empty_state(ui, "No partners", "This CPA has no PartyInfo elements with transports.", |_| {});
            return;
        }

        ui.add_space(4.0);
        let mut clone = self.toolbar(ui, document);
        ui.add_space(6.0);

        let max_width = ui.available_width() * 0.5;
        egui::Panel::right("transport_detail_panel")
            .resizable(true)
            .default_size(440.0)
            .min_size(320.0)
            .max_size(max_width)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(4.0);
                    clone |= self.detail_panel(ui, shared);
                });
            });
        let mut events = TreeEvents::default();
        self.tree.context_menu = CLONE_TRANSPORT;
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let Some(document) = shared.document.as_ref() else { return };
            let kinds = &self.kinds;
            let menu = &mut clone;
            events = self.tree.show(ui, &self.rows, |ui, row| {
                let in_transport = row.and_then(|row| kinds.get(&row)).is_some_and(|kind| match kind {
                    RowKind::Partner(_) => false,
                    kind => document.ancestor_named(kind.element(), Some(CPA_NAMESPACE), "Transport").is_some(),
                });
                if in_transport {
                    if ui.button("Clone transport").clicked() {
                        *menu = true;
                        ui.close();
                    }
                } else {
                    ui.label(RichText::new("Right-click a transport row to clone it").weak());
                }
            });
        });
        if let Some(document) = shared.document.as_ref() {
            if events.selected.is_some() {
                self.load_detail(document);
            }
        }
        if let Some((_, column)) = events.double_clicked {
            self.focus_value = column == 1 && self.detail.options.is_none() && self.detail.target.is_some();
        }
        if clone {
            self.clone_transport(shared);
        }
    }
}
