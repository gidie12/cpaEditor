//! CPA Validation tab: validates the CPA against the schema (validator_tab.py).

use cpa_core::validation::validate_document;
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};

const NOT_VALIDATED: &str = "Geen validatie uitgevoerd";
const NO_ERRORS: &str = "Geen fouten gevonden";

pub struct ValidationTab {
    result: String,
    valid: Option<bool>,
    /// A result belongs to the CPA it was run on
    generation: u64,
}

impl Default for ValidationTab {
    fn default() -> Self {
        ValidationTab { result: NOT_VALIDATED.into(), valid: None, generation: 0 }
    }
}

impl Tab for ValidationTab {
    fn title(&self) -> &'static str {
        "CPA Validation"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            *self = ValidationTab { generation: shared.generation, ..Default::default() };
        }
        let Some(document) = &shared.document else {
            no_cpa(ui);
            return;
        };
        if ui.button("Validate CPA").clicked() {
            match validate_document(document) {
                Ok(errors) if errors.is_empty() => {
                    self.result = NO_ERRORS.into();
                    self.valid = Some(true);
                }
                Ok(errors) => {
                    self.result = errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
                    self.valid = Some(false);
                    shared.log.info(format!("Validation found {} problem(s)", errors.len()));
                }
                Err(e) => {
                    self.result = e.clone();
                    self.valid = Some(false);
                    shared.log.error(e);
                }
            }
        }
        ui.add_space(4.0);
        ui.label("Validation Errors:");
        let color = match self.valid {
            Some(true) => Some(egui::Color32::from_rgb(0x19, 0x87, 0x54)),
            Some(false) => Some(ui.visuals().error_fg_color),
            None => None,
        };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let mut text = RichText::new(&self.result).monospace();
            if let Some(color) = color {
                text = text.color(color);
            }
            ui.add(egui::Label::new(text).wrap().selectable(true));
        });
    }
}
