//! Collaboration Role tab (collaboration_role.py and service_action_dialog.py): per partner the services with their
//! Send and Receive actions. Actions can be added (with the mirrored action at the other partner), changed and
//! deleted; Collaboration Roles can be added and deleted.

use std::collections::HashMap;

use cpa_core::collaboration_roles::{
    self as roles, characteristic, describe_action, describe_collaboration_role, Characteristics, Direction, Mirror,
    NewCollaborationRole, CHARACTERISTICS,
};
use cpa_core::xml::{Document, NodeId};
use cpa_core::{CPA_NAMESPACE, XLINK_NAMESPACE};
use eframe::egui::{self, Color32, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::tree_table::{RowKey, TreeEvents, TreeRow, TreeTable};
use crate::theme::{self, PARTNER_COLORS};
use crate::widgets::{combo, editable_combo, message_box, Answer, Confirm, Message};

/// Width of the label column of the forms, so the fields of all form parts line up.
const LABEL_WIDTH: f32 = 230.0;
const PROBLEM_COLOR: Color32 = Color32::from_rgb(0xdc, 0x35, 0x45);
const EXPERIMENTAL_NOTICE: &str = "Experimental: adding, changing and deleting Service actions and Collaboration \
    Roles is new. Check the result on the CPA Validation tab before you save the CPA.";
const TIME_TO_PERFORM_SUGGESTIONS: &[&str] = &["PT1H", "PT2H", "PT24H"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Partner,
    Role,
    Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuAction {
    AddAction,
    AddRole,
    Delete,
}

/// Send points right (out of the partner), Receive points left (into the partner).
fn points_right(direction: Direction) -> bool {
    direction == Direction::Send
}

fn key(node: NodeId) -> RowKey {
    node.index() as RowKey
}

/// The values of the detail panel of the selected action.
#[derive(Default)]
struct Detail {
    action: Option<NodeId>,
    title: String,
    binding_id: String,
    action_name: String,
    package_id: String,
    channel_id: String,
    characteristics: Characteristics,
    package_options: Vec<String>,
    channel_options: Vec<String>,
    problems: Vec<String>,
    /// Action, package, channel and characteristics as shown, to see whether the user changed something.
    original: (String, String, String, Characteristics),
}

impl Detail {
    fn current(&self) -> (String, String, String, Characteristics) {
        (self.action_name.clone(), self.package_id.clone(), self.channel_id.clone(), self.characteristics.clone())
    }

    fn is_changed(&self) -> bool {
        self.action.is_some() && self.current() != self.original
    }
}

#[derive(Clone)]
enum PendingDelete {
    Action(NodeId),
    Role(NodeId),
}

pub struct CollaborationRoleTab {
    tree: TreeTable,
    rows: Vec<TreeRow>,
    kinds: HashMap<RowKey, (Kind, NodeId)>,
    filter: String,
    built_for: Option<(u64, String)>,
    generation: u64,
    detail: Detail,
    confirm: Option<Confirm<PendingDelete>>,
    message: Option<Message>,
    action_dialog: Option<ServiceActionDialog>,
    role_dialog: Option<CollaborationRoleDialog>,
}

impl Default for CollaborationRoleTab {
    fn default() -> Self {
        CollaborationRoleTab {
            tree: TreeTable::new(
                "collaboration_role",
                &[
                    ("Partner / Service / Direction", 330.0),
                    ("Action", 130.0),
                    ("Channel", 170.0),
                    ("Package", 120.0),
                    ("OtherPartyActionBinding", 260.0),
                ],
            ),
            rows: Vec::new(),
            kinds: HashMap::new(),
            filter: String::new(),
            built_for: None,
            generation: 0,
            detail: Detail::default(),
            confirm: None,
            message: None,
            action_dialog: None,
            role_dialog: None,
        }
    }
}

impl CollaborationRoleTab {
    /// Rebuilds the rows of the tree from the CPA, filtered on the search text.
    fn build_rows(&mut self, document: &Document) {
        self.rows.clear();
        self.kinds.clear();
        let search = self.filter.trim().to_lowercase();
        for (index, party_info) in roles::party_infos(document).into_iter().enumerate() {
            let text = format!("Partner {}", roles::party_name(document, party_info));
            let row = TreeRow::new(key(party_info), 0, vec![text]).header(PARTNER_COLORS[index % PARTNER_COLORS.len()]);
            self.rows.push(row);
            self.kinds.insert(key(party_info), (Kind::Partner, party_info));
            let collaboration_roles: Vec<NodeId> =
                document.find_all(party_info, Some(CPA_NAMESPACE), "CollaborationRole").collect();
            for role in collaboration_roles {
                self.insert_role(document, role, &search);
            }
        }
        if self.tree.selected.is_some_and(|key| !self.kinds.contains_key(&key)) {
            self.tree.selected = None;
        }
    }

    fn insert_role(&mut self, document: &Document, role: NodeId, search: &str) {
        let d = describe_collaboration_role(document, role);
        let actions = roles::service_binding(document, role)
            .map(|binding| roles::action_elements(document, binding))
            .unwrap_or_default();
        let role_text = [&d.process_name, &d.process_version, &d.role_name, &d.service_name, &d.service_type]
            .map(|s| s.as_str())
            .join(" ")
            .to_lowercase();
        let matches = |action: NodeId| action_matches(document, action, search);
        if !search.is_empty() && !role_text.contains(search) && !actions.iter().any(|&a| matches(a)) {
            return;
        }
        let service_type = if d.service_type.is_empty() { String::new() } else { format!(" [{}]", d.service_type) };
        let text = format!("{}{}", d.service_name, service_type);
        let secondary = format!("Role {} · {} {}", d.role_name, d.process_name, d.process_version);
        self.rows.push(TreeRow::new(key(role), 1, vec![text]).strong().secondary(secondary));
        self.kinds.insert(key(role), (Kind::Role, role));
        for action in actions {
            if search.is_empty() || role_text.contains(search) || matches(action) {
                self.insert_action(document, action, 2);
            }
        }
    }

    fn insert_action(&mut self, document: &Document, element: NodeId, depth: usize) {
        let action = describe_action(document, element);
        let problems = roles::validate_action(document, element);
        let mut row = TreeRow::new(
            key(element),
            depth,
            vec![
                action.direction.label().to_string(),
                action.action.clone(),
                action.channel_ids.join(", "),
                action.package_id.clone(),
                action.other_party_binding.clone(),
            ],
        );
        let color = match action.direction {
            Direction::Send => theme::ACCENT_LIGHT,
            Direction::Receive => theme::ACCENT_RECEIVE,
        };
        row = row.icon(points_right(action.direction), color);
        // Only a problem is worth a mark: red text and the number of problems after the direction
        if !problems.is_empty() {
            row = row.text_color(PROBLEM_COLOR).secondary(problem_count(problems.len()));
        }
        self.rows.push(row);
        self.kinds.insert(key(element), (Kind::Action, element));
        // Nested actions become child rows
        for nested in roles::action_elements(document, element) {
            self.insert_action(document, nested, depth + 1);
        }
    }

    fn selected(&self) -> Option<(Kind, NodeId)> {
        self.tree.selected.and_then(|key| self.kinds.get(&key).copied())
    }

    /// The CollaborationRole of the selected row: the role itself or the role of the selected action.
    fn selected_collaboration_role(&self, document: &Document) -> Option<NodeId> {
        match self.selected()? {
            (Kind::Partner, _) => None,
            (_, element) => roles::collaboration_role_of(document, element),
        }
    }

    fn selected_party_info(&self, document: &Document) -> Option<NodeId> {
        self.selected().and_then(|(_, element)| roles::party_info_of(document, element))
    }

    /// Shows the selected action in the detail panel, or empties the panel.
    fn show_action(&mut self, document: &Document) {
        let action = match self.selected() {
            Some((Kind::Action, element)) if document.is_attached(element) => Some(element),
            _ => None,
        };
        self.detail = Detail { action, title: "Select an action to edit".into(), ..Default::default() };
        let Some(element) = action else { return };
        let description = describe_action(document, element);
        let party_info = roles::party_info_of(document, element);
        let party_name = party_info.map(|p| roles::party_name(document, p)).unwrap_or_default();
        self.detail.title = format!("{} action of partner {}", description.direction.label(), party_name);
        self.detail.binding_id = description.id;
        self.detail.action_name = description.action;
        self.detail.package_id = description.package_id;
        self.detail.channel_id = description.channel_ids.first().cloned().unwrap_or_default();
        self.detail.characteristics = description.characteristics;
        self.detail.package_options = roles::package_ids(document);
        self.detail.channel_options = party_info.map(|p| roles::channel_ids(document, p)).unwrap_or_default();
        self.detail.problems = roles::validate_action(document, element);
        self.detail.original = self.detail.current();
    }

    /// Rebuilds the tree, selects the row of the node and shows it in the detail panel.
    fn rebuild_and_select(&mut self, shared: &Shared, node: NodeId) {
        let Some(document) = shared.document.as_ref() else { return };
        self.build_rows(document);
        self.tree.select(Some(key(node)), &self.rows);
        self.show_action(document);
        self.built_for = Some((shared.revision, self.filter.clone()));
    }

    fn save_changes(&mut self, shared: &mut Shared) {
        let (Some(element), Some(document)) = (self.detail.action, shared.document.as_mut()) else { return };
        let result = roles::update_action(
            document,
            element,
            &self.detail.action_name,
            &self.detail.package_id,
            &self.detail.channel_id,
            &self.detail.characteristics,
        );
        match result {
            Ok(()) => {
                shared.log.info(format!("Action '{}' changed", self.detail.binding_id));
                shared.changed();
            }
            Err(e) => self.message = Some(("Save Changes".into(), e, true)),
        }
    }

    fn add_service_action(&mut self, document: &Document) {
        match self.selected_collaboration_role(document) {
            Some(role) => self.action_dialog = Some(ServiceActionDialog::new(document, role)),
            None => {
                let text = "Select a service or an action first.".to_string();
                self.message = Some(("Add Service action".into(), text, false));
            }
        }
    }

    fn add_collaboration_role(&mut self, document: &Document) {
        let party_info = self.selected_party_info(document).or_else(|| roles::party_infos(document).first().copied());
        if let Some(party_info) = party_info {
            self.role_dialog = Some(CollaborationRoleDialog::new(document, party_info));
        }
    }

    /// Asks for confirmation before deleting the selected action or CollaborationRole.
    fn delete_selected(&mut self, document: &Document) {
        match self.selected() {
            Some((Kind::Action, element)) => {
                let action = describe_action(document, element);
                let other_party_info =
                    roles::party_info_of(document, element).and_then(|p| roles::other_party_info(document, p));
                let counterpart = other_party_info
                    .and_then(|other| roles::find_action_by_binding_id(document, other, &action.other_party_binding));
                self.confirm = Some(match (counterpart, other_party_info) {
                    (Some(_), Some(other)) => Confirm {
                        title: "Delete action".into(),
                        text: format!(
                            "Delete action '{}'?\n\nYes: also delete the mirrored action '{}' of partner {}.\n\
                             No: delete only this action.",
                            action.id,
                            action.other_party_binding,
                            roles::party_name(document, other)
                        ),
                        with_cancel: true,
                        subject: PendingDelete::Action(element),
                    },
                    _ => Confirm {
                        title: "Delete action".into(),
                        text: format!("Delete action '{}'?", action.id),
                        with_cancel: false,
                        subject: PendingDelete::Action(element),
                    },
                });
            }
            Some((Kind::Role, role)) => {
                let description = describe_collaboration_role(document, role);
                self.confirm = Some(Confirm {
                    title: "Delete Collaboration Role".into(),
                    text: format!(
                        "Delete the Collaboration Role of service '{}' with all its actions?\n\
                         Actions of the other partner that refer to them are not deleted.",
                        description.service_name
                    ),
                    with_cancel: false,
                    subject: PendingDelete::Role(role),
                });
            }
            _ => {}
        }
    }

    fn handle_confirmation(&mut self, ctx: &egui::Context, shared: &mut Shared) {
        let Some(confirm) = &self.confirm else { return };
        let Some((answer, subject)) = confirm.show(ctx) else { return };
        let with_cancel = confirm.with_cancel;
        self.confirm = None;
        let Some(document) = shared.document.as_mut() else { return };
        match subject {
            PendingDelete::Action(element) => {
                // With a mirrored action: Yes deletes both, No only this one, Cancel nothing
                let delete_counterpart = match (answer, with_cancel) {
                    (Answer::Cancel, _) | (Answer::No, false) => return,
                    (Answer::Yes, true) => true,
                    _ => false,
                };
                let deleted = roles::delete_action(document, element, delete_counterpart);
                shared.log.info(format!("Service action deleted: {}", deleted.join(", ")));
                shared.changed();
            }
            PendingDelete::Role(role) => {
                if answer != Answer::Yes {
                    return;
                }
                let service = describe_collaboration_role(document, role).service_name;
                match roles::delete_collaboration_role(document, role) {
                    Ok(()) => {
                        shared.log.info(format!("Collaboration Role deleted: service {service}"));
                        shared.changed();
                    }
                    Err(e) => self.message = Some(("Delete Collaboration Role".into(), e, true)),
                }
            }
        }
    }

    fn toolbar(&mut self, ui: &mut Ui) -> Option<MenuAction> {
        let mut action = None;
        let selected = self.selected().map(|(kind, _)| kind);
        ui.horizontal(|ui| {
            let filter = egui::TextEdit::singleline(&mut self.filter)
                .hint_text("Filter on service, role, action or binding id")
                .desired_width(300.0);
            ui.add(filter);
            ui.add_space(8.0);
            let can_add_action = matches!(selected, Some(Kind::Role | Kind::Action));
            let add = ui
                .add_enabled(can_add_action, egui::Button::new("+ Service Action"))
                .on_disabled_hover_text("Select a service or an action first");
            if add.clicked() {
                action = Some(MenuAction::AddAction);
            }
            if ui.button("+ Collaboration Role").clicked() {
                action = Some(MenuAction::AddRole);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = match selected {
                    Some(Kind::Role) => "Delete Collaboration Role…",
                    _ => "Delete action…",
                };
                let can_delete = matches!(selected, Some(Kind::Role | Kind::Action));
                if theme::danger(ui, label, can_delete).clicked() {
                    action = Some(MenuAction::Delete);
                }
            });
        });
        action
    }

    fn detail_panel(&mut self, ui: &mut Ui) {
        // The content takes the width of the panel and never widens it (long ids, messages and channel names)
        ui.set_max_width(ui.available_width());
        if self.detail.action.is_none() {
            theme::empty_state(
                ui,
                "No action selected",
                "Select a Send or Receive row on the left to edit its binding.",
                |_| {},
            );
            return;
        }
        ui.add_space(4.0);
        ui.label(RichText::new(&self.detail.title).heading().strong());
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if self.detail.problems.is_empty() {
                theme::badge(ui, "Valid", theme::ok_color(ui));
            } else {
                theme::badge(ui, &problem_count(self.detail.problems.len()), theme::error_color(ui));
            }
            // A long binding id must not widen the panel
            let id = egui::Label::new(RichText::new(&self.detail.binding_id).monospace().weak()).truncate();
            ui.add(id).on_hover_text(&self.detail.binding_id);
        });
        for problem in &self.detail.problems {
            ui.label(RichText::new(format!("• {problem}")).color(theme::error_color(ui)));
        }
        ui.add_space(8.0);
        ui.add_space(-6.0);
        theme::section(ui, "Binding");
        form_grid("action_detail").show(ui, |ui| {
            ui.label("Action");
            let field = egui::TextEdit::singleline(&mut self.detail.action_name).desired_width(ui.available_width());
            ui.add(field);
            ui.end_row();
            ui.label("Package");
            combo(ui, "detail_package", &mut self.detail.package_id, &self.detail.package_options, true);
            ui.end_row();
            ui.label("Channel");
            combo(ui, "detail_channel", &mut self.detail.channel_id, &self.detail.channel_options, true);
            ui.end_row();
        });
        ui.add_space(8.0);
        characteristics_frame(ui, "detail", &mut self.detail.characteristics, true);
        ui.add_space(8.0);
    }

    /// The buttons of the detail form, fixed at the bottom of the panel so they never scroll out of view.
    fn detail_actions(&mut self, ui: &mut Ui, shared: &mut Shared) {
        let changed = self.detail.is_changed();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if theme::action_button(ui, "Save changes", true, changed).clicked() {
                self.save_changes(shared);
            }
            if ui.add_enabled(changed, egui::Button::new("Discard")).clicked() {
                if let Some(document) = shared.document.as_ref() {
                    self.show_action(document);
                }
            }
            if !changed {
                ui.label(RichText::new("No changes").weak());
            }
        });
        ui.add_space(6.0);
    }

    fn run(&mut self, action: MenuAction, document: &Document) {
        match action {
            MenuAction::AddAction => self.add_service_action(document),
            MenuAction::AddRole => self.add_collaboration_role(document),
            MenuAction::Delete => self.delete_selected(document),
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context, shared: &mut Shared) {
        self.handle_confirmation(ctx, shared);
        if let Some((title, text, error)) = &self.message {
            if message_box(ctx, title, text, *error) {
                self.message = None;
            }
        }
        if let (Some(dialog), Some(document)) = (&mut self.action_dialog, shared.document.as_mut()) {
            match dialog.show(ctx, document) {
                DialogResult::Open => {}
                DialogResult::Cancelled => self.action_dialog = None,
                DialogResult::Done(element, mirror) => {
                    let mut added = vec![describe_action(document, element).id];
                    if let Some(mirror) = mirror {
                        added.push(describe_action(document, mirror).id);
                    }
                    shared.log.info(format!("Service action added: {}", added.join(", ")));
                    shared.changed();
                    self.action_dialog = None;
                    self.rebuild_and_select(shared, element);
                }
            }
        }
        if let (Some(dialog), Some(document)) = (&mut self.role_dialog, shared.document.as_mut()) {
            match dialog.show(ctx, document) {
                DialogResult::Open => {}
                DialogResult::Cancelled => self.role_dialog = None,
                DialogResult::Done(role, _) => {
                    let service = describe_collaboration_role(document, role).service_name;
                    shared.log.info(format!("Collaboration Role added: service {service}"));
                    if !dialog.error.is_empty() {
                        shared.log.error(&dialog.error);
                    }
                    shared.changed();
                    self.role_dialog = None;
                    self.rebuild_and_select(shared, role);
                }
            }
        }
    }
}

/// The same words as the validation tab: "1 error", "n errors".
fn problem_count(count: usize) -> String {
    if count == 1 { "1 error".to_string() } else { format!("{count} errors") }
}

fn action_matches(document: &Document, action: NodeId, search: &str) -> bool {
    let action = describe_action(document, action);
    [&action.action, &action.id, &action.other_party_binding].iter().any(|value| value.to_lowercase().contains(search))
}

/// A two-column form grid whose label column has the same width everywhere.
fn form_grid(id: &str) -> egui::Grid {
    egui::Grid::new(id).num_columns(2).spacing([12.0, 4.0]).min_col_width(LABEL_WIDTH)
}

/// Fields for the attributes of BusinessTransactionCharacteristics. An empty value leaves the attribute out.
fn characteristics_frame(ui: &mut Ui, id: &str, characteristics: &mut Characteristics, enabled: bool) {
    theme::section(ui, "BusinessTransactionCharacteristics");
    form_grid(&format!("{id}_characteristics")).show(ui, |ui| {
        for definition in CHARACTERISTICS {
            ui.label(definition.name);
            let mut value = characteristic(characteristics, definition.name).to_string();
            let changed = match definition.allowed {
                Some(allowed) => {
                    let options: Vec<String> =
                        std::iter::once(String::new()).chain(allowed.iter().map(|v| v.to_string())).collect();
                    combo(ui, (id, definition.name), &mut value, &options, enabled)
                }
                None => editable_combo(ui, (id, definition.name), &mut value, TIME_TO_PERFORM_SUGGESTIONS, enabled),
            };
            if changed {
                roles::set_characteristic(characteristics, definition.name, &value);
            }
            ui.end_row();
        }
    });
}

impl Tab for CollaborationRoleTab {
    fn title(&self) -> &'static str {
        "Collaboration Role"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            // Another CPA: forget the selection, the open rows and the dialogs; keep the filter
            let filter = std::mem::take(&mut self.filter);
            *self = CollaborationRoleTab { generation: shared.generation, filter, ..Default::default() };
        }
        if shared.document.is_none() {
            no_cpa(ui, shared);
            return;
        }
        let Some(document) = shared.document.as_ref() else { return };
        // Channel and binding ids differ at the end: shorten them in the middle
        self.tree.middle_truncate = vec![2, 4];
        let state = (shared.revision, self.filter.clone());
        if self.built_for.as_ref() != Some(&state) {
            self.build_rows(document);
            self.show_action(document);
            self.built_for = Some(state);
        }

        theme::notice(ui, EXPERIMENTAL_NOTICE);
        ui.add_space(6.0);
        let mut action = self.toolbar(ui);
        ui.add_space(6.0);

        let max_width = ui.available_width() * 0.5;
        egui::Panel::right("collaboration_role_detail")
            .resizable(true)
            .default_size(440.0)
            .min_size(340.0)
            .max_size(max_width)
            .show(ui, |ui| {
                if self.detail.action.is_some() {
                    egui::Panel::bottom("collaboration_role_detail_actions").show(ui, |ui| {
                        self.detail_actions(ui, shared);
                    });
                }
                egui::CentralPanel::no_frame().show(ui, |ui| {
                    // A form taller than the panel shows its scroll bar, so no field looks cut off
                    let scroll_bar = egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded;
                    egui::ScrollArea::vertical().auto_shrink([false, false]).scroll_bar_visibility(scroll_bar).show(ui, |ui| {
                        ui.add_space(4.0);
                        self.detail_panel(ui);
                    });
                });
            });
        let mut events = TreeEvents::default();
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let kinds = &self.kinds;
            let menu = &mut action;
            events = self.tree.show(ui, &self.rows, |ui, row| {
                let kind = row.and_then(|row| kinds.get(&row)).map(|(kind, _)| *kind);
                let mut item = |ui: &mut Ui, text: &str, chosen: MenuAction| {
                    if ui.button(text).clicked() {
                        *menu = Some(chosen);
                        ui.close();
                    }
                };
                if matches!(kind, Some(Kind::Role | Kind::Action)) {
                    item(ui, "Add Service action", MenuAction::AddAction);
                }
                item(ui, "Add Collaboration Role", MenuAction::AddRole);
                match kind {
                    Some(Kind::Action) => item(ui, "Delete action", MenuAction::Delete),
                    Some(Kind::Role) => item(ui, "Delete Collaboration Role", MenuAction::Delete),
                    _ => {}
                }
            });
        });
        if events.delete_pressed {
            action = Some(MenuAction::Delete);
        }
        if let Some(document) = shared.document.as_ref() {
            if events.selected.is_some() {
                self.show_action(document);
            }
            if let Some(action) = action {
                self.run(action, document);
            }
        }
        let ctx = ui.ctx().clone();
        self.dialogs(&ctx, shared);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Dialogs
// ---------------------------------------------------------------------------------------------------------------

enum DialogResult {
    Open,
    Cancelled,
    Done(NodeId, Option<NodeId>),
}

/// The text of a CollaborationRole in a list: service, role and process.
fn describe_role_choice(document: &Document, role: NodeId) -> String {
    let d = describe_collaboration_role(document, role);
    format!("{} (Role {}, Process {} {})", d.service_name, d.role_name, d.process_name, d.process_version)
}

/// A field whose value follows a suggestion until the user types something else in it.
#[derive(Default)]
struct Suggested {
    value: String,
    suggestion: Option<String>,
}

impl Suggested {
    fn suggest(&mut self, value: String) {
        if self.suggestion.is_none() || self.suggestion.as_deref() == Some(self.value.as_str()) {
            self.value = value.clone();
        }
        self.suggestion = Some(value);
    }
}

/// Dialog to add a Service action (CanSend or CanReceive) to a CollaborationRole, by default with the mirrored
/// action at the other partner.
struct ServiceActionDialog {
    role: NodeId,
    party_name: String,
    service: String,
    other_party_name: String,
    other_roles: Vec<NodeId>,
    other_role_choices: Vec<String>,
    package_options: Vec<String>,
    channel_options: Vec<String>,
    mirror_channel_options: Vec<String>,
    direction: Direction,
    action_name: String,
    binding_id: Suggested,
    package_id: String,
    channel_id: String,
    characteristics: Characteristics,
    mirror_enabled: bool,
    mirror_role: String,
    mirror_binding_id: Suggested,
    mirror_channel_id: String,
    error: String,
    focus_requested: bool,
}

impl ServiceActionDialog {
    fn new(document: &Document, role: NodeId) -> Self {
        let party_info = roles::party_info_of(document, role).expect("a CollaborationRole is in a PartyInfo");
        let other_party_info = roles::other_party_info(document, party_info);
        let other_roles: Vec<NodeId> = other_party_info
            .map(|other| document.find_all(other, Some(CPA_NAMESPACE), "CollaborationRole").collect())
            .unwrap_or_default();
        let package_options = roles::package_ids(document);
        let default_package = document.attribute(party_info, Some(CPA_NAMESPACE), "defaultMshPackageId").unwrap_or("");
        let package_id = if package_options.iter().any(|p| p == default_package) {
            default_package.to_string()
        } else {
            package_options.first().cloned().unwrap_or_default()
        };
        let counterpart =
            roles::find_counterpart_collaboration_role(document, role).or_else(|| other_roles.first().copied());
        let mut dialog = ServiceActionDialog {
            role,
            party_name: roles::party_name(document, party_info),
            service: describe_role_choice(document, role),
            other_party_name: other_party_info.map(|o| roles::party_name(document, o)).unwrap_or_default(),
            other_role_choices: other_roles.iter().map(|&r| describe_role_choice(document, r)).collect(),
            mirror_enabled: !other_roles.is_empty(),
            other_roles,
            package_options,
            channel_options: roles::channel_ids(document, party_info),
            mirror_channel_options: other_party_info.map(|o| roles::channel_ids(document, o)).unwrap_or_default(),
            direction: Direction::Send,
            action_name: String::new(),
            binding_id: Suggested::default(),
            package_id,
            channel_id: String::new(),
            characteristics: roles::suggest_characteristics(document, role),
            mirror_role: counterpart.map(|c| describe_role_choice(document, c)).unwrap_or_default(),
            mirror_binding_id: Suggested::default(),
            mirror_channel_id: String::new(),
            error: String::new(),
            focus_requested: false,
        };
        dialog.update_suggestions(document, true);
        dialog
    }

    fn selected_mirror_role(&self) -> Option<NodeId> {
        self.other_role_choices.iter().position(|c| c == &self.mirror_role).map(|i| self.other_roles[i])
    }

    fn update_suggestions(&mut self, document: &Document, direction_changed: bool) {
        let action_name = match self.action_name.trim() {
            "" => "Action",
            name => name,
        }
        .to_string();
        self.binding_id.suggest(roles::suggest_binding_id(document, self.role, self.direction, &action_name, &[]));
        if direction_changed {
            self.channel_id = roles::suggest_channel_id(document, self.role, self.direction);
        }
        if let Some(mirror_role) = self.selected_mirror_role() {
            let reserved = [self.binding_id.value.clone()];
            let opposite = self.direction.opposite();
            self.mirror_binding_id.suggest(roles::suggest_binding_id(document, mirror_role, opposite, &action_name, &reserved));
            if direction_changed {
                self.mirror_channel_id = roles::suggest_channel_id(document, mirror_role, opposite);
            }
        }
    }

    /// Adds the action(s) to the CPA; on a problem the dialog stays open with the message.
    fn submit(&mut self, document: &mut Document) -> Option<(NodeId, Option<NodeId>)> {
        let mirror = if self.mirror_enabled {
            let Some(collaboration_role) = self.selected_mirror_role() else {
                self.error = "Choose the service of the mirrored action".into();
                return None;
            };
            Some(Mirror {
                collaboration_role,
                binding_id: self.mirror_binding_id.value.trim().to_string(),
                channel_id: self.mirror_channel_id.clone(),
            })
        } else {
            None
        };
        let result = roles::add_action(
            document,
            self.role,
            self.direction,
            &self.action_name,
            &self.package_id,
            &self.channel_id,
            &self.characteristics,
            self.binding_id.value.trim(),
            mirror.as_ref(),
        );
        result.map_err(|e| self.error = e).ok()
    }

    fn show(&mut self, ctx: &egui::Context, document: &mut Document) -> DialogResult {
        let mut result = DialogResult::Open;
        let mut open = true;
        let mut direction_changed = false;
        let mut name_changed = false;
        egui::Window::new("Add Service action")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(640.0)
            .max_width(800.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                form_grid("service_action_form").show(ui, |ui| {
                    ui.label("Partner");
                    ui.label(&self.party_name);
                    ui.end_row();
                    ui.label("Service");
                    ui.label(&self.service);
                    ui.end_row();
                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        for direction in [Direction::Send, Direction::Receive] {
                            let text = format!("{} ({})", direction.label(), direction.element_name());
                            direction_changed |= ui.radio_value(&mut self.direction, direction, text).changed();
                        }
                    });
                    ui.end_row();
                    ui.label("Action");
                    let field = egui::TextEdit::singleline(&mut self.action_name).desired_width(420.0);
                    let response = ui.add(field);
                    if !self.focus_requested {
                        response.request_focus();
                        self.focus_requested = true;
                    }
                    name_changed |= response.changed();
                    ui.end_row();
                    ui.label("ThisPartyActionBinding id");
                    ui.add(egui::TextEdit::singleline(&mut self.binding_id.value).desired_width(420.0));
                    ui.end_row();
                    ui.label("Package");
                    combo(ui, "dialog_package", &mut self.package_id, &self.package_options, true);
                    ui.end_row();
                    ui.label("Channel");
                    combo(ui, "dialog_channel", &mut self.channel_id, &self.channel_options, true);
                    ui.end_row();
                });
                ui.add_space(6.0);
                characteristics_frame(ui, "dialog", &mut self.characteristics, true);
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Mirrored action").strong());
                    let text = format!(
                        "Also add the opposite action at partner {}; both OtherPartyActionBindings refer to each other",
                        self.other_party_name
                    );
                    ui.add_enabled(!self.other_roles.is_empty(), egui::Checkbox::new(&mut self.mirror_enabled, text));
                    let enabled = self.mirror_enabled;
                    form_grid("mirror_form").show(ui, |ui| {
                        ui.label("Service");
                        direction_changed |=
                            combo(ui, "mirror_role", &mut self.mirror_role, &self.other_role_choices, enabled);
                        ui.end_row();
                        ui.label("ThisPartyActionBinding id");
                        let field =
                            egui::TextEdit::singleline(&mut self.mirror_binding_id.value).desired_width(420.0);
                        ui.add_enabled(enabled, field);
                        ui.end_row();
                        ui.label("Channel");
                        combo(ui, "mirror_channel", &mut self.mirror_channel_id, &self.mirror_channel_options, enabled);
                        ui.end_row();
                    });
                });
                if !self.error.is_empty() {
                    ui.label(RichText::new(&self.error).color(ui.visuals().error_fg_color));
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        if let Some((element, mirror)) = self.submit(document) {
                            result = DialogResult::Done(element, mirror);
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        result = DialogResult::Cancelled;
                    }
                });
            });
        if matches!(result, DialogResult::Open) && (direction_changed || name_changed) {
            self.update_suggestions(document, direction_changed);
        }
        if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            result = DialogResult::Cancelled;
        }
        result
    }
}

/// Dialog to add a CollaborationRole (one Service) to a partner, by default also at the other partner.
struct CollaborationRoleDialog {
    party_info: NodeId,
    other_party_info: Option<NodeId>,
    party_name: String,
    other_party_name: String,
    values: NewCollaborationRole,
    mirror_enabled: bool,
    mirror_role_name: String,
    error: String,
}

impl CollaborationRoleDialog {
    /// Process, role and service type of an existing CollaborationRole of the partner are a good start.
    fn new(document: &Document, party_info: NodeId) -> Self {
        let other_party_info = roles::other_party_info(document, party_info);
        let mut values = NewCollaborationRole::default();
        if let Some(existing) = document.find(party_info, Some(CPA_NAMESPACE), "CollaborationRole") {
            let attribute = |element: Option<NodeId>, namespace: &str, name: &str| {
                element.and_then(|e| document.attribute(e, Some(namespace), name)).unwrap_or("").to_string()
            };
            let process = document.find(existing, Some(CPA_NAMESPACE), "ProcessSpecification");
            let role = document.find(existing, Some(CPA_NAMESPACE), "Role");
            let service = document
                .find(existing, Some(CPA_NAMESPACE), "ServiceBinding")
                .and_then(|binding| document.find(binding, Some(CPA_NAMESPACE), "Service"));
            values.process_name = attribute(process, CPA_NAMESPACE, "name");
            values.process_version = attribute(process, CPA_NAMESPACE, "version");
            values.process_href = attribute(process, XLINK_NAMESPACE, "href");
            values.role_name = attribute(role, CPA_NAMESPACE, "name");
            values.role_href = attribute(role, XLINK_NAMESPACE, "href");
            values.service_type = attribute(service, CPA_NAMESPACE, "type");
        }
        let mirror_role_name = other_party_info
            .and_then(|other| document.find(other, Some(CPA_NAMESPACE), "CollaborationRole"))
            .and_then(|role| document.find(role, Some(CPA_NAMESPACE), "Role"))
            .and_then(|role| document.attribute(role, Some(CPA_NAMESPACE), "name"))
            .unwrap_or("")
            .to_string();
        CollaborationRoleDialog {
            party_info,
            other_party_info,
            party_name: roles::party_name(document, party_info),
            other_party_name: other_party_info.map(|o| roles::party_name(document, o)).unwrap_or_default(),
            values,
            mirror_enabled: other_party_info.is_some(),
            mirror_role_name,
            error: String::new(),
        }
    }

    fn submit(&mut self, document: &mut Document) -> Option<(NodeId, Option<NodeId>)> {
        if self.mirror_enabled && self.mirror_role_name.trim().is_empty() {
            self.error = "Role name of the other partner is required".into();
            return None;
        }
        let role = match roles::add_collaboration_role(document, self.party_info, &self.values) {
            Ok(role) => role,
            Err(e) => {
                self.error = e;
                return None;
            }
        };
        let mut mirror = None;
        if let (true, Some(other)) = (self.mirror_enabled, self.other_party_info) {
            let values = NewCollaborationRole { role_name: self.mirror_role_name.clone(), ..self.values.clone() };
            match roles::add_collaboration_role(document, other, &values) {
                Ok(role) => mirror = Some(role),
                Err(e) => self.error = e,
            }
        }
        Some((role, mirror))
    }

    fn show(&mut self, ctx: &egui::Context, document: &mut Document) -> DialogResult {
        let mut result = DialogResult::Open;
        let mut open = true;
        egui::Window::new("Add Collaboration Role")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(640.0)
            .max_width(800.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("collaboration_role_form").num_columns(3).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label("Partner");
                    ui.label(&self.party_name);
                    ui.end_row();
                    let v = &mut self.values;
                    let fields: [(&str, &mut String); 8] = [
                        ("Process name", &mut v.process_name),
                        ("Process version", &mut v.process_version),
                        ("Process href", &mut v.process_href),
                        ("Process uuid", &mut v.process_uuid),
                        ("Role name", &mut v.role_name),
                        ("Role href", &mut v.role_href),
                        ("Service name", &mut v.service_name),
                        ("Service type", &mut v.service_type),
                    ];
                    for (label, value) in fields {
                        ui.label(label);
                        ui.add(egui::TextEdit::singleline(value).desired_width(420.0));
                        if label == "Process uuid" && ui.button("Generate UUID").clicked() {
                            *value = format!("urn:uuid:{}", uuid::Uuid::new_v4());
                        }
                        ui.end_row();
                    }
                });
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Other partner").strong());
                    let text = format!("Also add this service at partner {}", self.other_party_name);
                    ui.add_enabled(self.other_party_info.is_some(), egui::Checkbox::new(&mut self.mirror_enabled, text));
                    ui.horizontal(|ui| {
                        ui.label("Role name");
                        let field = egui::TextEdit::singleline(&mut self.mirror_role_name).desired_width(420.0);
                        ui.add_enabled(self.mirror_enabled, field);
                    });
                });
                if !self.error.is_empty() {
                    ui.label(RichText::new(&self.error).color(ui.visuals().error_fg_color));
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        if let Some((role, mirror)) = self.submit(document) {
                            result = DialogResult::Done(role, mirror);
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        result = DialogResult::Cancelled;
                    }
                });
            });
        if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            result = DialogResult::Cancelled;
        }
        result
    }
}
