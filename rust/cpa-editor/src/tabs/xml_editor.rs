//! XML editor tab (xml_editor_tab.py): the whole CPA as a tree of elements and attributes. The value of an element
//! or attribute is edited in the detail panel; elements and attributes can be added to the selected element and
//! deleted. "Copy value" copies the name and the value to the clipboard and into the Add form.

use std::collections::HashMap;

use cpa_core::cpa;
use cpa_core::xml::{Document, NodeId};
use cpa_core::CPA_NAMESPACE;
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme::{self, PARTNER_COLORS};
use crate::tree_table::{RowKey, TreeEvents, TreeRow, TreeTable};
use crate::widgets::{combo, Answer, Confirm};

/// Width of the label column of the detail forms.
const LABEL_WIDTH: f32 = 90.0;
const ADD_TYPES: [&str; 2] = ["Element", "Attribute"];

#[derive(Clone, Debug, PartialEq, Eq)]
enum RowKind {
    Element(NodeId),
    /// An attribute: its element, its name in Clark notation (for the core functions) and the shown name.
    Attribute(NodeId, String, String),
}

impl RowKind {
    fn element(&self) -> NodeId {
        match self {
            RowKind::Element(node) | RowKind::Attribute(node, _, _) => *node,
        }
    }
}

fn element_key(node: NodeId) -> RowKey {
    (node.index() as RowKey) << 12
}

fn attribute_key(node: NodeId, position: usize) -> RowKey {
    element_key(node) | (position as RowKey + 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuAction {
    CopyValue,
    Delete,
}

#[derive(Default)]
struct Detail {
    target: Option<RowKind>,
    /// The shown name: the local name of an element, `prefix:local` of an attribute.
    name: String,
    /// The element of an attribute, the parent of an element.
    owner: String,
    path: String,
    namespace: String,
    /// False for an element with child elements: its text is not edited here.
    editable: bool,
    child_count: usize,
    value: String,
    original: String,
    error: Option<String>,
}

impl Detail {
    fn is_changed(&self) -> bool {
        self.target.is_some() && self.editable && self.value != self.original
    }
}

pub struct XmlEditorTab {
    tree: TreeTable,
    /// All rows; `rows` is this list filtered on the search text.
    all_rows: Vec<TreeRow>,
    rows: Vec<TreeRow>,
    kinds: HashMap<RowKey, RowKind>,
    filter: String,
    built_for: Option<(u64, String)>,
    generation: u64,
    detail: Detail,
    add_type: String,
    add_name: String,
    add_value: String,
    add_error: Option<String>,
    confirm: Option<Confirm<RowKind>>,
    focus_value: bool,
    closed_initially: bool,
}

impl Default for XmlEditorTab {
    fn default() -> Self {
        XmlEditorTab {
            tree: TreeTable::new("xml_editor", &[("Element", 420.0), ("Value", 400.0)]),
            all_rows: Vec::new(),
            rows: Vec::new(),
            kinds: HashMap::new(),
            filter: String::new(),
            built_for: None,
            generation: 0,
            detail: Detail::default(),
            add_type: ADD_TYPES[0].to_string(),
            add_name: String::new(),
            add_value: String::new(),
            add_error: None,
            confirm: None,
            focus_value: false,
            closed_initially: false,
        }
    }
}

/// The prefix declared for a namespace URI on the element or one of its ancestors.
fn prefix_in_scope(document: &Document, element: NodeId, uri: &str) -> Option<String> {
    document.ancestors(element).filter_map(|node| document.element(node)).find_map(|element| {
        element.namespaces.iter().find(|declaration| declaration.uri == uri).and_then(|d| d.prefix.clone())
    })
}

/// `prefix:local` for an attribute in a namespace, `local` without.
fn attribute_display_name(document: &Document, element: NodeId, position: usize) -> String {
    let attribute = &document.attributes(element)[position];
    let prefix = match (&attribute.name.prefix, &attribute.name.namespace) {
        (Some(prefix), _) => Some(prefix.clone()),
        (None, Some(namespace)) => prefix_in_scope(document, element, namespace),
        (None, None) => None,
    };
    match prefix {
        Some(prefix) => format!("{prefix}:{}", attribute.name.local),
        None => attribute.name.local.clone(),
    }
}

/// The local names from the root down to the element.
fn element_path(document: &Document, element: NodeId) -> String {
    let mut names: Vec<&str> =
        document.ancestors(element).filter(|&node| document.is_element(node)).map(|n| document.local_name(n)).collect();
    names.reverse();
    names.join(" / ")
}

impl XmlEditorTab {
    fn build_rows(&mut self, document: &Document) {
        self.all_rows.clear();
        self.kinds.clear();
        let mut partner = 0;
        if let Some(root) = document.root() {
            self.insert_element(document, root, 0, &mut partner);
        }
        if !self.closed_initially {
            // The certificate data is long and rarely edited: KeyInfo starts closed
            for row in &self.all_rows {
                if let Some(RowKind::Element(node)) = self.kinds.get(&row.key) {
                    if document.local_name(*node) == "KeyInfo" {
                        self.tree.closed.insert(row.key);
                    }
                }
            }
            self.closed_initially = true;
        }
        self.apply_filter();
    }

    /// An element row, then its attributes, then its child elements (populate_tree).
    fn insert_element(&mut self, document: &Document, element: NodeId, depth: usize, partner: &mut usize) {
        let name = document.local_name(element).to_string();
        let text = document.text_trimmed(element);
        let mut row = TreeRow::new(element_key(element), depth, vec![name, text.clone()]);
        if document.is_named(element, Some(CPA_NAMESPACE), "PartyInfo") {
            row = row.header(PARTNER_COLORS[*partner % PARTNER_COLORS.len()]);
            *partner += 1;
            if let Some(party) = document.attribute(element, Some(CPA_NAMESPACE), "partyName") {
                row = row.secondary(party.to_string());
            }
        } else if depth == 0 {
            row = row.strong();
        } else if text.is_empty() {
            // The id of the element, so it can be recognised when its attribute rows are closed
            let id = document.attributes(element).iter().find(|a| a.name.local.ends_with("Id") || a.name.local == "id");
            if let Some(id) = id {
                row = row.secondary(id.value.clone());
            }
        }
        self.all_rows.push(row);
        self.kinds.insert(element_key(element), RowKind::Element(element));
        for (position, attribute) in document.attributes(element).iter().enumerate() {
            let key = attribute_key(element, position);
            let display = attribute_display_name(document, element, position);
            let row = TreeRow::new(key, depth + 1, vec![format!("@{display}"), attribute.value.clone()]);
            self.all_rows.push(row);
            self.kinds.insert(key, RowKind::Attribute(element, attribute.name.clark(), display));
        }
        let children: Vec<NodeId> = document.child_elements(element).collect();
        for child in children {
            self.insert_element(document, child, depth + 1, partner);
        }
    }

    /// Keeps the rows whose name or value contains the search text, with their ancestors.
    fn apply_filter(&mut self) {
        let search = self.filter.trim().to_lowercase();
        if search.is_empty() {
            self.rows = self.all_rows.clone();
        } else {
            let mut keep = vec![false; self.all_rows.len()];
            let mut stack: Vec<usize> = Vec::new();
            for (index, row) in self.all_rows.iter().enumerate() {
                while stack.last().is_some_and(|&parent| self.all_rows[parent].depth >= row.depth) {
                    stack.pop();
                }
                let matches = row.cells.iter().chain(row.secondary.iter()).any(|c| c.to_lowercase().contains(&search));
                if matches {
                    keep[index] = true;
                    for &ancestor in &stack {
                        keep[ancestor] = true;
                    }
                }
                stack.push(index);
            }
            self.rows = self.all_rows.iter().zip(keep).filter(|(_, keep)| *keep).map(|(row, _)| row.clone()).collect();
        }
        if self.tree.selected.is_some_and(|key| !self.rows.iter().any(|row| row.key == key)) {
            self.tree.selected = None;
        }
    }

    fn selected(&self) -> Option<RowKind> {
        self.tree.selected.and_then(|key| self.kinds.get(&key).cloned())
    }

    fn load_detail(&mut self, document: &Document) {
        self.detail = Detail::default();
        let Some(kind) = self.selected() else { return };
        let element = kind.element();
        if !document.is_attached(element) {
            return;
        }
        self.detail = match &kind {
            RowKind::Element(_) => {
                let child_count = document.child_elements(element).count();
                let text = document.text_trimmed(element);
                Detail {
                    name: document.local_name(element).to_string(),
                    owner: document.parent_element(element).map(|p| document.local_name(p).to_string()).unwrap_or_default(),
                    path: element_path(document, element),
                    namespace: document.name(element).and_then(|n| n.namespace.clone()).unwrap_or_default(),
                    editable: child_count == 0,
                    child_count,
                    value: text.clone(),
                    original: text,
                    ..Default::default()
                }
            }
            RowKind::Attribute(_, clark, display) => {
                let attribute = document.attributes(element).iter().find(|a| &a.name.clark() == clark);
                let value = attribute.map(|a| a.value.clone()).unwrap_or_default();
                Detail {
                    name: display.clone(),
                    owner: document.local_name(element).to_string(),
                    path: element_path(document, element),
                    namespace: attribute.and_then(|a| a.name.namespace.clone()).unwrap_or_default(),
                    editable: true,
                    value: value.clone(),
                    original: value,
                    ..Default::default()
                }
            }
        };
        self.detail.target = Some(kind);
    }

    fn rebuild_and_select(&mut self, shared: &Shared, key: Option<RowKey>) {
        let Some(document) = shared.document.as_ref() else { return };
        self.build_rows(document);
        self.tree.select(key, &self.rows);
        self.load_detail(document);
        self.built_for = Some((shared.revision, self.filter.clone()));
    }

    /// Writes the edited value (edit_cell / change_xml_object). An empty value is ignored, as in Python.
    fn save_changes(&mut self, shared: &mut Shared) {
        let (Some(target), Some(document)) = (self.detail.target.clone(), shared.document.as_mut()) else { return };
        if !self.detail.is_changed() || self.detail.value.is_empty() {
            return;
        }
        let value = self.detail.value.clone();
        let result = match &target {
            RowKind::Element(element) => cpa::set_element_text(document, *element, &value),
            RowKind::Attribute(element, clark, _) => cpa::set_attribute_by_name(document, *element, clark, &value),
        };
        match result {
            Ok(()) => {
                let kind = if matches!(target, RowKind::Element(_)) { "Element" } else { "Attribute" };
                shared.log.info(format!("{kind} '{}' changed", self.detail.name));
                shared.changed();
                self.rebuild_and_select(shared, self.tree.selected);
            }
            Err(e) => {
                shared.log.error(format!("Error while changing element: {e}"));
                self.detail.error = Some(e);
            }
        }
    }

    /// Adds an element or attribute to the selected element, or to the element of the selected attribute (add_item).
    fn add_item(&mut self, shared: &mut Shared) {
        let (Some(target), Some(document)) = (self.selected(), shared.document.as_mut()) else { return };
        let element = target.element();
        let name = self.add_name.trim().to_string();
        let value = self.add_value.clone();
        let owner = document.local_name(element).to_string();
        let result = if self.add_type == "Attribute" {
            cpa::set_attribute_by_name(document, element, &name, &value).map(|()| {
                let position = cpa::parse_attribute_name(&name).ok().and_then(|(namespace, local)| {
                    document.attributes(element).iter().position(|a| a.name.is(namespace.as_deref(), &local))
                });
                position.map(|p| attribute_key(element, p))
            })
        } else {
            cpa::add_element(document, element, &name, &value).map(|new| Some(element_key(new)))
        };
        match result {
            Ok(key) => {
                shared.log.info(format!("{} '{name}' added to '{owner}'", self.add_type));
                shared.changed();
                self.add_name.clear();
                self.add_value.clear();
                self.add_error = None;
                // A new row must be visible, whatever the filter
                self.filter.clear();
                self.rebuild_and_select(shared, key.or(Some(element_key(element))));
            }
            Err(e) => {
                shared.log.error(format!("Error while adding element: {e}"));
                self.add_error = Some(e);
            }
        }
    }

    fn ask_delete(&mut self, document: &Document) {
        let Some(kind) = self.selected() else { return };
        let text = match &kind {
            RowKind::Element(element) => {
                if document.parent_element(*element).is_none() {
                    return;
                }
                format!("Delete element '{}' with all its content?", document.local_name(*element))
            }
            RowKind::Attribute(element, _, display) => {
                format!("Delete attribute '{display}' of element '{}'?", document.local_name(*element))
            }
        };
        let title = if matches!(kind, RowKind::Element(_)) { "Delete element" } else { "Delete attribute" };
        self.confirm = Some(Confirm { title: title.into(), text, with_cancel: false, subject: kind });
    }

    fn delete(&mut self, shared: &mut Shared, kind: RowKind) {
        let Some(document) = shared.document.as_mut() else { return };
        let (result, select) = match &kind {
            RowKind::Element(element) => {
                let name = document.local_name(*element).to_string();
                let parent = document.parent_element(*element);
                if cpa::delete_element(document, *element) {
                    (Ok(format!("Element '{name}' deleted")), parent)
                } else {
                    (Err("The root element cannot be deleted".to_string()), None)
                }
            }
            RowKind::Attribute(element, clark, display) => (
                cpa::delete_attribute_by_name(document, *element, clark).map(|()| format!("Attribute '{display}' deleted")),
                Some(*element),
            ),
        };
        match result {
            Ok(message) => {
                shared.log.info(message);
                shared.changed();
                self.rebuild_and_select(shared, select.map(element_key));
            }
            Err(e) => shared.log.error(format!("Error while deleting element: {e}")),
        }
    }

    /// copy_value: "name value" to the clipboard, and name and value into the Add form.
    fn copy_value(&mut self, ctx: &egui::Context) {
        let Some(key) = self.tree.selected else { return };
        let Some(row) = self.rows.iter().find(|row| row.key == key) else { return };
        let name = row.cells[0].trim_start_matches('@').to_string();
        let value = row.cells.get(1).cloned().unwrap_or_default();
        ctx.copy_text(format!("{name} {value}"));
        self.add_name = name;
        self.add_value = value;
    }

    fn toolbar(&mut self, ui: &mut Ui, document: &Document) -> Option<MenuAction> {
        let mut action = None;
        ui.horizontal(|ui| {
            let filter = egui::TextEdit::singleline(&mut self.filter)
                .hint_text("Filter on element, attribute or value")
                .desired_width(320.0);
            ui.add(filter);
            if !self.filter.is_empty() {
                let matches = self.rows.len();
                ui.label(RichText::new(format!("{matches} rows shown")).weak());
                if ui.button("Clear").clicked() {
                    self.filter.clear();
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let selected = self.selected();
                let (label, can_delete, why) = match &selected {
                    Some(RowKind::Attribute(..)) => ("Delete attribute…", true, ""),
                    Some(RowKind::Element(element)) if document.parent_element(*element).is_none() => {
                        ("Delete element…", false, "The root element cannot be deleted")
                    }
                    Some(RowKind::Element(_)) => ("Delete element…", true, ""),
                    None => ("Delete…", false, "Select an element or an attribute first"),
                };
                let response = ui.add_enabled(can_delete, theme::danger_button(ui, label));
                if response.on_disabled_hover_text(why).clicked() {
                    action = Some(MenuAction::Delete);
                }
            });
        });
        action
    }

    fn detail_panel(&mut self, ui: &mut Ui, shared: &mut Shared) {
        let Some(target) = self.detail.target.clone() else {
            theme::empty_state(
                ui,
                "Nothing selected",
                "Select an element or an attribute in the tree to edit its value or add to it.",
                |_| {},
            );
            return;
        };
        let is_attribute = matches!(target, RowKind::Attribute(..));
        ui.add_space(4.0);
        let heading = if is_attribute { format!("@{}", self.detail.name) } else { self.detail.name.clone() };
        ui.label(RichText::new(heading).heading().strong());
        let kind = if is_attribute {
            format!("Attribute of {}", self.detail.owner)
        } else if self.detail.owner.is_empty() {
            "Root element".to_string()
        } else {
            format!("Element in {}", self.detail.owner)
        };
        ui.label(RichText::new(kind).weak());

        let mut save = false;
        ui.add_space(10.0);
        egui::Grid::new("xml_detail").num_columns(2).spacing([12.0, 8.0]).min_col_width(LABEL_WIDTH).show(ui, |ui| {
            ui.label("Path");
            ui.add(egui::Label::new(RichText::new(&self.detail.path).monospace().weak()).wrap());
            ui.end_row();
            ui.label("Namespace");
            let namespace = if self.detail.namespace.is_empty() { "(none)" } else { self.detail.namespace.as_str() };
            ui.add(egui::Label::new(RichText::new(namespace).weak()).wrap());
            ui.end_row();
            ui.label(if self.detail.editable { "Value" } else { "Content" });
            if self.detail.editable {
                let multiline = self.detail.original.contains('\n') || self.detail.original.len() > 120;
                let width = ui.available_width();
                let field = if multiline {
                    egui::TextEdit::multiline(&mut self.detail.value).desired_rows(6).code_editor()
                } else {
                    egui::TextEdit::singleline(&mut self.detail.value)
                };
                let response = ui.add(field.desired_width(width));
                if std::mem::take(&mut self.focus_value) {
                    response.request_focus();
                }
                if response.changed() {
                    self.detail.error = None;
                }
                if response.lost_focus() && !multiline {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.detail.value = self.detail.original.clone();
                    }
                }
            } else {
                let count = self.detail.child_count;
                let text = if count == 1 { "1 child element".to_string() } else { format!("{count} child elements") };
                ui.label(RichText::new(text).weak());
            }
            ui.end_row();
        });
        if let Some(error) = &self.detail.error {
            ui.label(RichText::new(error).color(theme::error_color(ui)));
        }
        if self.detail.editable {
            ui.add_space(8.0);
            let changed = self.detail.is_changed();
            let empty = self.detail.value.is_empty();
            ui.horizontal(|ui| {
                save |= theme::action_button(ui, "Save changes", true, changed && !empty).clicked();
                if ui.add_enabled(changed, egui::Button::new("Discard")).clicked() {
                    if let Some(document) = shared.document.as_ref() {
                        self.load_detail(document);
                    }
                }
                if !changed {
                    ui.label(RichText::new("No changes").weak());
                } else if empty {
                    ui.label(RichText::new("A value cannot be empty").color(theme::warning_color(ui)));
                }
            });
        }
        if save {
            self.save_changes(shared);
        }

        // Add an element or an attribute to the element (of the attribute)
        let owner = if is_attribute { self.detail.owner.clone() } else { self.detail.name.clone() };
        theme::section(ui, &format!("Add to {owner}"));
        let mut add = false;
        egui::Grid::new("xml_add").num_columns(2).spacing([12.0, 8.0]).min_col_width(LABEL_WIDTH).show(ui, |ui| {
            ui.label("Type");
            let types: Vec<String> = ADD_TYPES.iter().map(|t| t.to_string()).collect();
            combo(ui, "xml_add_type", &mut self.add_type, &types, true);
            ui.end_row();
            ui.label("Name");
            let hint = if self.add_type == "Attribute" { "e.g. version" } else { "e.g. Comment" };
            let width = ui.available_width();
            if ui.add(egui::TextEdit::singleline(&mut self.add_name).hint_text(hint).desired_width(width)).changed() {
                self.add_error = None;
            }
            ui.end_row();
            ui.label("Value");
            let width = ui.available_width();
            ui.add(egui::TextEdit::singleline(&mut self.add_value).desired_width(width));
            ui.end_row();
        });
        if let Some(error) = &self.add_error {
            ui.label(RichText::new(error).color(theme::error_color(ui)));
        }
        ui.add_space(8.0);
        let can_add = !self.add_name.trim().is_empty();
        let label = if self.add_type == "Attribute" { "Add attribute" } else { "Add element" };
        // Primary only when nothing else is waiting to be saved
        let primary = can_add && !self.detail.is_changed();
        add |= theme::action_button(ui, label, primary, can_add).on_disabled_hover_text("Enter a name first").clicked();
        if add {
            self.add_item(shared);
        }
    }

    fn handle_confirmation(&mut self, ctx: &egui::Context, shared: &mut Shared) {
        let Some(confirm) = &self.confirm else { return };
        let Some((answer, subject)) = confirm.show(ctx) else { return };
        self.confirm = None;
        if answer == Answer::Yes {
            self.delete(shared, subject);
        }
    }
}

impl Tab for XmlEditorTab {
    fn title(&self) -> &'static str {
        "XML editor"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            let filter = std::mem::take(&mut self.filter);
            *self = XmlEditorTab { generation: shared.generation, filter, ..Default::default() };
        }
        if shared.document.is_none() {
            no_cpa(ui, shared);
            return;
        }
        let Some(document) = shared.document.as_ref() else { return };
        let state = (shared.revision, self.filter.clone());
        if self.built_for.as_ref() != Some(&state) {
            let filter_changed = self.built_for.as_ref().is_some_and(|(revision, _)| *revision == shared.revision);
            if filter_changed {
                // Only the search text changed: open everything so all matches are visible
                if !self.filter.trim().is_empty() {
                    self.tree.closed.clear();
                }
                self.apply_filter();
            } else {
                self.build_rows(document);
            }
            self.load_detail(document);
            self.built_for = Some(state);
        }

        ui.add_space(4.0);
        let mut action = self.toolbar(ui, document);
        ui.add_space(6.0);

        let max_width = ui.available_width() * 0.5;
        egui::Panel::right("xml_editor_detail_panel")
            .resizable(true)
            .default_size(440.0)
            .min_size(320.0)
            .max_size(max_width)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(4.0);
                    self.detail_panel(ui, shared);
                });
            });
        let mut events = TreeEvents::default();
        egui::CentralPanel::no_frame().show(ui, |ui| {
            if self.rows.is_empty() {
                theme::empty_state(ui, "No matches", "No element, attribute or value contains the filter text.", |_| {});
                return;
            }
            let menu = &mut action;
            let kinds = &self.kinds;
            let root_key = shared.document.as_ref().and_then(|d| d.root()).map(element_key);
            events = self.tree.show(ui, &self.rows, |ui, row| {
                let Some(row) = row.filter(|row| kinds.contains_key(row)) else {
                    ui.label(RichText::new("Right-click a row for its actions").weak());
                    return;
                };
                if ui.button("Copy value").clicked() {
                    *menu = Some(MenuAction::CopyValue);
                    ui.close();
                }
                if Some(row) != root_key && ui.button("Delete…").clicked() {
                    *menu = Some(MenuAction::Delete);
                    ui.close();
                }
            });
        });
        if events.delete_pressed {
            action = Some(MenuAction::Delete);
        }
        if let Some(document) = shared.document.as_ref() {
            if events.selected.is_some() {
                self.load_detail(document);
            }
        }
        if let Some((_, column)) = events.double_clicked {
            self.focus_value = column == 1 && self.detail.editable;
        }
        let ctx = ui.ctx().clone();
        match action {
            Some(MenuAction::CopyValue) => self.copy_value(&ctx),
            Some(MenuAction::Delete) => {
                if let Some(document) = shared.document.as_ref() {
                    self.ask_delete(document);
                }
            }
            None => {}
        }
        self.handle_confirmation(&ctx, shared);
    }
}
