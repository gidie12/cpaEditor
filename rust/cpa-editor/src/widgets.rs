//! Small widgets shared by the tabs: drop-down lists, message boxes and the confirmation dialog.

use eframe::egui::{self, RichText, Ui};

/// A read-only drop-down list (Tk `ttk.Combobox` with state "readonly"). Returns true when the value changed.
pub fn combo(ui: &mut Ui, id: impl egui::AsIdSalt, value: &mut String, options: &[String], enabled: bool) -> bool {
    let mut changed = false;
    ui.add_enabled_ui(enabled, |ui| {
        egui::ComboBox::from_id_salt(id).width(ui.available_width()).truncate().selected_text(value.as_str()).show_ui(ui, |ui| {
            for option in options {
                if ui.selectable_label(value == option, option).clicked() && value != option {
                    *value = option.clone();
                    changed = true;
                }
            }
        });
    });
    changed
}

/// A text field with a list of suggestions next to it (Tk `ttk.Combobox` with state "normal").
pub fn editable_combo(ui: &mut Ui, id: impl egui::AsIdSalt, value: &mut String, options: &[&str], enabled: bool) -> bool {
    let _ = id;
    let mut changed = false;
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let button_size = ui.spacing().interact_size.y;
            let width = (ui.available_width() - button_size - 4.0).max(60.0);
            changed |= ui.add(egui::TextEdit::singleline(value).desired_width(width)).changed();
            let button = ui.add(egui::Button::new("").min_size(egui::vec2(button_size, button_size)));
            let color = ui.visuals().widgets.inactive.fg_stroke.color;
            crate::theme::paint_arrow(ui.painter(), button.rect.center(), 8.0, crate::theme::Arrow::Down, color);
            egui::Popup::menu(&button).show(|ui| {
                for option in options {
                    if ui.selectable_label(value == option, *option).clicked() {
                        *value = option.to_string();
                        changed = true;
                    }
                }
            });
        });
    });
    changed
}

/// The answer of a [`Confirm`] dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
    Cancel,
}

/// A modal question with Yes/No (and optionally Cancel), like Tk's messagebox.askyesno(cancel).
pub struct Confirm<T> {
    pub title: String,
    pub text: String,
    pub with_cancel: bool,
    /// What the question is about, handed back with the answer.
    pub subject: T,
}

impl<T: Clone> Confirm<T> {
    /// Shows the dialog; returns the answer once a button is pressed (Escape is Cancel, or No without Cancel).
    pub fn show(&self, ctx: &egui::Context) -> Option<(Answer, T)> {
        let mut answer = None;
        let modal = egui::Modal::new(egui::Id::unique(("confirm", &self.title))).show(ctx, |ui| {
            ui.set_max_width(480.0);
            ui.set_min_width(360.0);
            ui.heading(&self.title);
            ui.add_space(6.0);
            ui.label(&self.text);
            ui.add_space(10.0);
            // A delete or replace is confirmed with a red button named after the action, next to Cancel
            let destructive = !self.with_cancel
                && ["Delete", "Remove", "Replace"].iter().any(|verb| self.title.starts_with(verb));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if destructive {
                    let label = self.title.trim_end_matches('?').to_string();
                    if ui.add(crate::theme::danger_button(ui, &label).min_size(egui::vec2(96.0, 28.0))).clicked() {
                        answer = Some(Answer::Yes);
                    }
                    let cancel = ui.add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 28.0)));
                    if cancel.clicked() {
                        answer = Some(Answer::No);
                    }
                    if !ui.memory(|m| m.focused().is_some()) {
                        cancel.request_focus();
                    }
                    return;
                }
                // Right to left: the last button is drawn first
                if self.with_cancel && ui.button("Cancel").clicked() {
                    answer = Some(Answer::Cancel);
                }
                if ui.button("No").clicked() {
                    answer = Some(Answer::No);
                }
                if ui.button("Yes").clicked() {
                    answer = Some(Answer::Yes);
                }
            });
        });
        if modal.should_close() && answer.is_none() {
            answer = Some(if self.with_cancel { Answer::Cancel } else { Answer::No });
        }
        answer.map(|a| (a, self.subject.clone()))
    }
}

/// A modal message with an OK button (Tk's messagebox.showerror / showinfo). Returns true when it is closed.
pub fn message_box(ctx: &egui::Context, title: &str, text: &str, error: bool) -> bool {
    let mut close = false;
    let modal = egui::Modal::new(egui::Id::unique(("message", title))).show(ctx, |ui| {
        ui.set_max_width(480.0);
        ui.heading(title);
        ui.add_space(6.0);
        let text = RichText::new(text);
        ui.label(if error { text.color(ui.visuals().error_fg_color) } else { text });
        ui.add_space(10.0);
        if ui.button("OK").clicked() {
            close = true;
        }
    });
    close || modal.should_close()
}

/// A message that stays until it is closed: (title, text, is error).
pub type Message = (String, String, bool);

pub use crate::theme::notice;
