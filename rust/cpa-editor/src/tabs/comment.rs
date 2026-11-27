//! Comment tab (comment_tab.py): the Comment elements of the CPA in a list, with a panel to change the selected
//! comment or write a new one. Deleting asks for confirmation first.

use cpa_core::cpa::{add_comment, comments, delete_attribute_by_name, delete_comment, set_attribute_by_name, set_comment};
use cpa_core::xml::{Document, NodeId, XML_NAMESPACE};
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme;
use crate::tree_table::{RowKey, TreeRow, TreeTable};
use crate::widgets::{Answer, Confirm};

const LABEL_WIDTH: f32 = 80.0;
const LANGUAGES: &[&str] = &["nl-NL", "en-US"];
/// The language of a new comment, as in the Python version.
const DEFAULT_LANGUAGE: &str = "nl-NL";

fn key(node: NodeId) -> RowKey {
    node.index() as RowKey
}

/// The text of a comment on one line: line breaks and indentation become single spaces.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn language_attribute() -> String {
    format!("{{{XML_NAMESPACE}}}lang")
}

fn read(document: &Document, comment: NodeId) -> (String, String) {
    let text = document.text(comment).unwrap_or_default();
    let lang = document.attribute(comment, Some(XML_NAMESPACE), "lang").unwrap_or_default().to_string();
    (text, lang)
}

/// A text field for the language with a button that lists the usual values (Tk's editable `ttk.Combobox`).
fn language_field(ui: &mut Ui, lang: &mut String, label: egui::Id) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let button_size = ui.spacing().interact_size.y;
        let width = (ui.available_width() - button_size - 4.0).max(60.0);
        ui.add(egui::TextEdit::singleline(lang).desired_width(width).hint_text("e.g. en-US")).labelled_by(label);
        let button = ui.add(egui::Button::new("").min_size(egui::vec2(button_size, button_size)));
        button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Choose a language"));
        let color = ui.visuals().widgets.inactive.fg_stroke.color;
        theme::paint_arrow(ui.painter(), button.rect.center(), 8.0, theme::Arrow::Down, color);
        egui::Popup::menu(&button).show(|ui| {
            for option in LANGUAGES {
                if ui.selectable_label(lang == option, *option).clicked() {
                    *lang = option.to_string();
                }
            }
        });
    });
}

/// What the panel on the right shows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Nothing,
    Edit(NodeId),
    New,
}

pub struct CommentTab {
    tree: TreeTable,
    rows: Vec<TreeRow>,
    nodes: Vec<NodeId>,
    built_for: Option<u64>,
    generation: u64,
    mode: Mode,
    text: String,
    lang: String,
    /// The values of the comment as the CPA has them, to see whether the form was changed.
    original: (String, String),
    confirm: Option<Confirm<NodeId>>,
    focus_text: bool,
}

impl Default for CommentTab {
    fn default() -> Self {
        CommentTab {
            tree: TreeTable::new("comments", &[("#", 48.0), ("Language", 100.0), ("Comment", 400.0)]),
            rows: Vec::new(),
            nodes: Vec::new(),
            built_for: None,
            generation: 0,
            mode: Mode::Nothing,
            text: String::new(),
            lang: String::new(),
            original: (String::new(), String::new()),
            confirm: None,
            focus_text: false,
        }
    }
}

impl CommentTab {
    fn build_rows(&mut self, document: &Document) {
        self.nodes = comments(document);
        self.rows = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, &node)| {
                let (text, lang) = read(document, node);
                TreeRow::new(key(node), 0, vec![(index + 1).to_string(), lang, one_line(&text)])
            })
            .collect();
        match self.mode {
            Mode::Edit(node) if self.nodes.contains(&node) => self.show(document, node),
            Mode::Edit(_) => {
                self.mode = Mode::Nothing;
                self.tree.selected = None;
            }
            _ => {}
        }
    }

    /// Shows a comment in the panel.
    fn show(&mut self, document: &Document, node: NodeId) {
        let (text, lang) = read(document, node);
        self.mode = Mode::Edit(node);
        self.text = text.clone();
        self.lang = lang.clone();
        self.original = (text, lang);
    }

    fn select(&mut self, document: &Document, node: NodeId) {
        self.show(document, node);
        self.tree.select(Some(key(node)), &self.rows);
    }

    fn start_new(&mut self) {
        self.mode = Mode::New;
        self.tree.selected = None;
        self.text.clear();
        self.lang = DEFAULT_LANGUAGE.to_string();
        self.original = (String::new(), DEFAULT_LANGUAGE.to_string());
        self.focus_text = true;
    }

    fn is_changed(&self) -> bool {
        (self.text.as_str(), self.lang.as_str()) != (self.original.0.as_str(), self.original.1.as_str())
    }

    fn position(&self, node: NodeId) -> Option<usize> {
        self.nodes.iter().position(|&n| n == node)
    }

    /// Rebuilds the list after a change of this tab and selects `node`.
    fn after_change(&mut self, shared: &mut Shared, node: Option<NodeId>) {
        shared.changed();
        let Some(document) = shared.document.as_ref() else { return };
        self.mode = Mode::Nothing;
        self.tree.selected = None;
        self.build_rows(document);
        self.built_for = Some(shared.revision);
        if let Some(node) = node {
            self.select(document, node);
        }
    }

    fn add(&mut self, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else { return };
        let node = add_comment(document, &self.text, self.lang.trim());
        shared.log.info("Comment added");
        self.after_change(shared, Some(node));
    }

    fn save(&mut self, shared: &mut Shared, node: NodeId) {
        let Some(document) = shared.document.as_mut() else { return };
        if self.text != self.original.0 {
            if let Err(e) = set_comment(document, node, &self.text) {
                shared.log.error(format!("Error while changing element: {e}"));
                return;
            }
        }
        let lang = self.lang.trim();
        if lang != self.original.1 {
            let result = if lang.is_empty() {
                delete_attribute_by_name(document, node, &language_attribute())
            } else {
                set_attribute_by_name(document, node, &language_attribute(), lang)
            };
            if let Err(e) = result {
                shared.log.error(format!("Error while changing element: {e}"));
            }
        }
        if let Some(index) = self.position(node) {
            shared.log.info(format!("Comment {} changed", index + 1));
        }
        self.after_change(shared, Some(node));
    }

    fn ask_delete(&mut self, node: NodeId) {
        let Some(index) = self.position(node) else { return };
        let preview = self.rows[index].cells[2].clone();
        let preview = if preview.chars().count() > 120 {
            format!("{}…", preview.chars().take(120).collect::<String>())
        } else {
            preview
        };
        self.confirm = Some(Confirm {
            title: "Delete comment".into(),
            text: format!("Delete comment {}?\n\n\"{preview}\"", index + 1),
            with_cancel: false,
            subject: node,
        });
    }

    /// Deletes the comment and selects the one that took its place, as the Python version does.
    fn delete(&mut self, shared: &mut Shared, node: NodeId) {
        let Some(index) = self.position(node) else { return };
        let Some(document) = shared.document.as_mut() else { return };
        if let Err(e) = delete_comment(document, node) {
            shared.log.error(format!("Error while deleting element: {e}"));
            return;
        }
        shared.log.info(format!("Comment {} deleted", index + 1));
        let remaining = comments(document);
        let next = (!remaining.is_empty()).then(|| remaining[index.min(remaining.len() - 1)]);
        self.after_change(shared, next);
    }

    fn toolbar(&mut self, ui: &mut Ui) -> (bool, bool) {
        let (mut new, mut delete) = (false, false);
        ui.horizontal(|ui| {
            new = ui.button("+ New comment").clicked();
            if !self.nodes.is_empty() {
                let count = self.nodes.len();
                let text = if count == 1 { "1 comment".to_string() } else { format!("{count} comments") };
                ui.label(RichText::new(text).weak());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let can_delete = matches!(self.mode, Mode::Edit(_));
                delete = ui
                    .add(theme::danger_button_state(ui, "Delete comment…", can_delete))
                    .on_disabled_hover_text("Select a comment first")
                    .clicked();
            });
        });
        (new, delete)
    }

    fn form(&mut self, ui: &mut Ui) {
        egui::Grid::new("comment_form").num_columns(2).spacing([12.0, 8.0]).min_col_width(LABEL_WIDTH).show(ui, |ui| {
            // The label stays at the top of the multi-line field
            let label = ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add_space(4.0);
                ui.label("Text").id
            });
            let label = label.inner;
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .id_salt("comment_text")
                    .desired_rows(10)
                    .desired_width(ui.available_width())
                    .hint_text("Free text for the readers of the CPA"),
            )
            .labelled_by(label);
            if std::mem::take(&mut self.focus_text) {
                response.request_focus();
            }
            ui.end_row();
            let label = ui.label("Language").id;
            language_field(ui, &mut self.lang, label);
            ui.end_row();
        });
    }

    /// The panel on the right. Returns the action of its buttons.
    fn detail_panel(&mut self, ui: &mut Ui) -> Option<PanelAction> {
        let mut action = None;
        let empty = self.text.trim().is_empty();
        match self.mode {
            Mode::Nothing => {
                theme::empty_state(
                    ui,
                    "No comment selected",
                    "Select a comment on the left to change it, or add a new one.",
                    |_| {},
                );
            }
            Mode::New => {
                ui.label(RichText::new("New comment").heading().strong());
                ui.label(RichText::new("It is added after the other comments, at the end of the CPA.").weak());
                ui.add_space(8.0);
                self.form(ui);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let add = theme::action_button(ui, "Add comment", true, !empty);
                    if add.on_disabled_hover_text("Write the text of the comment first").clicked() {
                        action = Some(PanelAction::Add);
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(PanelAction::Cancel);
                    }
                });
            }
            Mode::Edit(node) => {
                let position = self.position(node).map_or(0, |i| i + 1);
                ui.label(RichText::new(format!("Comment {position} of {}", self.nodes.len())).heading().strong());
                ui.add_space(8.0);
                self.form(ui);
                ui.add_space(12.0);
                let changed = self.is_changed();
                ui.horizontal(|ui| {
                    let save = theme::action_button(ui, "Save changes", true, changed && !empty);
                    let reason = if changed { "A comment cannot be empty" } else { "No changes to save" };
                    if save.on_disabled_hover_text(reason).clicked() {
                        action = Some(PanelAction::Save(node));
                    }
                    if ui.add_enabled(changed, egui::Button::new("Discard")).clicked() {
                        action = Some(PanelAction::Discard(node));
                    }
                    if !changed {
                        ui.label(RichText::new("No changes").weak());
                    } else if empty {
                        ui.label(RichText::new("A comment cannot be empty").color(theme::error_color(ui)));
                    } else {
                        ui.label(RichText::new("Unsaved changes").color(theme::warning_color(ui)));
                    }
                });
            }
        }
        action
    }
}

enum PanelAction {
    Add,
    Cancel,
    Save(NodeId),
    Discard(NodeId),
}

impl Tab for CommentTab {
    fn title(&self) -> &'static str {
        "Comment"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            *self = CommentTab { generation: shared.generation, ..Default::default() };
        }
        let Some(document) = shared.document.as_ref() else {
            no_cpa(ui, shared);
            return;
        };
        if self.built_for != Some(shared.revision) {
            self.build_rows(document);
            self.built_for = Some(shared.revision);
        }

        // Without comments the empty state has the only "New comment" button
        let (mut new, mut delete) = (false, false);
        if !self.nodes.is_empty() || self.mode == Mode::New {
            ui.add_space(4.0);
            (new, delete) = self.toolbar(ui);
            ui.add_space(6.0);
        }

        let mut panel_action = None;
        let mut new_from_empty = false;
        if self.nodes.is_empty() && self.mode != Mode::New {
            // Primary unless Save CPA… is: after deleting the last comment, saving is the next step
            let primary = !shared.is_dirty();
            theme::empty_state(
                ui,
                "No comments",
                "This CPA has no comments. A comment is free text for the people who read the CPA.",
                |ui| new_from_empty = theme::action_button(ui, "+ New comment", primary, true).clicked(),
            );
        } else {
            let max_width = ui.available_width() * 0.6;
            egui::Panel::right("comment_detail")
                .resizable(true)
                .default_size(460.0)
                .min_size(340.0)
                .max_size(max_width)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.add_space(4.0);
                        panel_action = self.detail_panel(ui);
                    });
                });
            egui::CentralPanel::no_frame().show(ui, |ui| {
                if self.nodes.is_empty() {
                    theme::empty_state(ui, "No comments yet", "Write the first comment on the right.", |_| {});
                    return;
                }
                let events = self.tree.show(ui, &self.rows, |_, _| {});
                if let Some(selected) = events.selected {
                    if let Some(&node) = self.nodes.iter().find(|&&n| key(n) == selected) {
                        if let Some(document) = shared.document.as_ref() {
                            self.show(document, node);
                            shared.log.debug(format!("Comment selected: index {}", self.position(node).unwrap_or(0)));
                        }
                    }
                }
                delete |= events.delete_pressed;
            });
        }

        if new || new_from_empty {
            self.start_new();
        }
        if delete {
            if let Mode::Edit(node) = self.mode {
                self.ask_delete(node);
            }
        }
        match panel_action {
            Some(PanelAction::Add) => self.add(shared),
            Some(PanelAction::Cancel) => self.mode = Mode::Nothing,
            Some(PanelAction::Save(node)) => self.save(shared, node),
            Some(PanelAction::Discard(node)) => {
                if let Some(document) = shared.document.as_ref() {
                    self.show(document, node);
                }
            }
            None => {}
        }

        let ctx = ui.ctx().clone();
        if let Some(confirm) = &self.confirm {
            if let Some((answer, node)) = confirm.show(&ctx) {
                self.confirm = None;
                if answer == Answer::Yes {
                    self.delete(shared, node);
                }
            }
        }
    }
}
