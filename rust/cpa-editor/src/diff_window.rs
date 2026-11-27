//! Show changes (diff_window.py): the changes of the CPA since it was loaded, as a window to look at, or as the
//! question before saving.

use cpa_core::diff::{changed_line_count, DiffLine, Hunk};
use eframe::egui::{self, text::LayoutJob, Color32, FontId, RichText, TextFormat, Ui};

// Light backgrounds with black text, readable in light and dark mode (the colours of the Python version)
const REMOVED: Color32 = Color32::from_rgb(0xff, 0xe3, 0xe1);
const ADDED: Color32 = Color32::from_rgb(0xdc, 0xf5, 0xdc);
const REMOVED_PART: Color32 = Color32::from_rgb(0xff, 0x9e, 0x97);
const ADDED_PART: Color32 = Color32::from_rgb(0x8f, 0xdc, 0x8f);
const CONTEXT: Color32 = Color32::from_rgb(0x88, 0x88, 0x88);

/// What the user chose in the window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffAnswer {
    /// Save the CPA with these changes (only when asked before saving); with the increased CPA ID when chosen.
    Save { new_cpa_id: Option<String> },
    /// Closed, or Cancel before saving.
    Close,
}

pub struct DiffWindow {
    hunks: Vec<Hunk>,
    changed: usize,
    /// Asked before saving: Save… and Cancel instead of a window to look at.
    confirm: bool,
    /// Before saving, when the CPA ID is still the loaded one: the current and the increased CPA ID.
    cpa_id: Option<(String, String)>,
    /// Save with the increased CPA ID.
    increase: bool,
}

impl DiffWindow {
    pub fn new(hunks: Vec<Hunk>, confirm: bool) -> Self {
        let changed = changed_line_count(&hunks);
        DiffWindow { hunks, changed, confirm, cpa_id: None, increase: true }
    }

    /// Before saving: offers to increase the CPA ID, which was not changed.
    pub fn offer_new_cpa_id(mut self, current: String, increased: String) -> Self {
        self.cpa_id = Some((current, increased));
        self
    }

    /// Asked before saving.
    pub fn is_confirm(&self) -> bool {
        self.confirm
    }

    /// Shows the window; returns the answer once it is closed.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<DiffAnswer> {
        let screen = ctx.content_rect().size();
        let size = egui::vec2((screen.x - 80.0).clamp(400.0, 1100.0), (screen.y - 80.0).clamp(300.0, 700.0));
        let mut answer = None;
        if self.confirm {
            // Before saving: a modal question, Escape is Cancel
            let modal = egui::Modal::new(egui::Id::unique("diff_confirm")).show(ctx, |ui| {
                ui.set_width(size.x);
                ui.heading("Save CPA with these changes?");
                ui.add_space(6.0);
                let extra = if self.cpa_id.is_some() { 40.0 } else { 0.0 };
                self.contents(ui, size.y - 130.0 - extra);
                ui.add_space(8.0);
                if let Some((current, increased)) = &self.cpa_id {
                    ui.checkbox(&mut self.increase, format!("Increase the CPA ID: {current} becomes {increased}"));
                    ui.label(RichText::new("The CPA ID was not changed since the CPA was loaded.").small().weak());
                    ui.add_space(6.0);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 28.0))).clicked() {
                        answer = Some(DiffAnswer::Close);
                    }
                    let save = ui.add(crate::theme::primary_button("Save…"));
                    if save.clicked() {
                        let new_cpa_id = self.cpa_id.as_ref().filter(|_| self.increase).map(|(_, increased)| increased.clone());
                        answer = Some(DiffAnswer::Save { new_cpa_id });
                    }
                });
            });
            if modal.should_close() && answer.is_none() {
                answer = Some(DiffAnswer::Close);
            }
        } else {
            let mut open = true;
            egui::Window::new("Changes since the CPA was loaded")
                .id(egui::Id::unique("diff_window"))
                .open(&mut open)
                .default_size(size)
                .collapsible(false)
                .show(ctx, |ui| self.contents(ui, f32::INFINITY));
            if !open {
                answer = Some(DiffAnswer::Close);
            }
        }
        answer
    }

    fn contents(&self, ui: &mut Ui, max_height: f32) {
        let lines = if self.changed == 1 { "1 line".to_string() } else { format!("{} lines", self.changed) };
        ui.label(format!(
            "{lines} changed since the CPA was loaded. Red: loaded CPA, green: current CPA; the changed words are \
             darker. Grey lines are unchanged."
        ));
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(max_height).auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for hunk in &self.hunks {
                ui.add_space(14.0);
                ui.label(RichText::new(format!("Line {}", hunk.line)).size(16.0).strong());
                ui.add_space(6.0);
                for line in &hunk.lines {
                    diff_line(ui, line);
                }
            }
            ui.add_space(10.0);
        });
    }
}

/// One line: a sign, the text, and the changed words on a darker background. Long lines wrap.
fn diff_line(ui: &mut Ui, line: &DiffLine) {
    let font = FontId::monospace(13.0);
    let (sign, text, parts, background, part_background) = match line {
        DiffLine::Context(text) => ("  ", text.as_str(), &[][..], None, None),
        DiffLine::Removed { text, parts } => ("- ", text.as_str(), &parts[..], Some(REMOVED), Some(REMOVED_PART)),
        DiffLine::Added { text, parts } => ("+ ", text.as_str(), &parts[..], Some(ADDED), Some(ADDED_PART)),
    };
    let color = if background.is_some() { Color32::BLACK } else { CONTEXT };
    let plain = TextFormat { font_id: font, color, ..Default::default() };
    let marked = TextFormat { background: part_background.unwrap_or_default(), ..plain.clone() };
    let mut job = LayoutJob::default();
    job.append(sign, 0.0, plain.clone());
    let mut position = 0;
    for &(start, end) in parts {
        job.append(&text[position..start], 0.0, plain.clone());
        job.append(&text[start..end], 0.0, marked.clone());
        position = end;
    }
    job.append(&text[position..], 0.0, plain);
    egui::Frame::new().fill(background.unwrap_or_default()).inner_margin(egui::Margin::symmetric(8, 2)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.add(egui::Label::new(job).wrap());
    });
}
