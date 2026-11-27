//! CPA Validation tab: validates the CPA against the schema (validator_tab.py).

use chrono::{DateTime, Local};
use cpa_core::validation::validate_document;
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme;

/// The outcome of the last validation.
enum Outcome {
    Valid,
    Errors(Vec<String>),
    /// The validation itself failed, e.g. a schema that could not be loaded.
    Failed(String),
}

#[derive(Default)]
pub struct ValidationTab {
    result: Option<(Outcome, DateTime<Local>, u64)>,
    /// A result belongs to the CPA it was run on
    generation: u64,
}

impl ValidationTab {
    fn validate(&mut self, shared: &mut Shared) {
        let Some(document) = &shared.document else { return };
        let outcome = match validate_document(document) {
            Ok(errors) if errors.is_empty() => {
                shared.log.info("Validation found no problems");
                Outcome::Valid
            }
            Ok(errors) => {
                shared.log.info(format!("Validation found {} problem(s)", errors.len()));
                Outcome::Errors(errors.iter().map(|e| e.to_string()).collect())
            }
            Err(e) => {
                shared.log.error(&e);
                Outcome::Failed(e)
            }
        };
        self.result = Some((outcome, Local::now(), shared.revision));
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
        if shared.document.is_none() {
            no_cpa(ui, shared);
            return;
        }
        let Some((outcome, time, revision)) = &self.result else {
            let mut run = false;
            theme::empty_state(
                ui,
                "Not validated yet",
                "Check the CPA against the ebXML CPP/CPA 2.0 schema before you save it.",
                |ui| run = ui.add(theme::primary_button("Validate CPA").min_size(egui::vec2(140.0, 32.0))).clicked(),
            );
            if run {
                self.validate(shared);
            }
            return;
        };

        let mut run = false;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            run = ui.add(theme::primary_button("Validate again")).clicked();
            ui.add_space(8.0);
            match outcome {
                Outcome::Valid => theme::badge(ui, "Valid", theme::ok_color(ui)),
                Outcome::Errors(errors) => {
                    let text = if errors.len() == 1 { "1 error".to_string() } else { format!("{} errors", errors.len()) };
                    theme::badge(ui, &text, theme::error_color(ui));
                }
                Outcome::Failed(_) => theme::badge(ui, "Validation failed", theme::error_color(ui)),
            }
            ui.label(RichText::new(format!("Last run {}", time.format("%H:%M:%S"))).weak());
            if *revision != shared.revision {
                ui.label(RichText::new("· the CPA has changed since").color(theme::warning_color(ui)));
            }
        });
        ui.add_space(8.0);
        match outcome {
            Outcome::Valid => {
                theme::empty_state(ui, "No errors found", "The CPA is valid against the schema.", |_| {});
            }
            Outcome::Failed(message) => {
                ui.label(RichText::new(message).color(theme::error_color(ui)));
            }
            Outcome::Errors(errors) => {
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(4)
                    .inner_margin(8)
                    .show(ui, |ui| {
                        let with_lines = errors.iter().any(|e| e.starts_with("Line "));
                        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                            for (index, error) in errors.iter().enumerate() {
                                // "Line N: message": the line number in its own column
                                let (line, message) = match error.split_once(": ") {
                                    Some((line, message)) if line.starts_with("Line ") => (line, message),
                                    _ => ("", error.as_str()),
                                };
                                ui.horizontal_top(|ui| {
                                    if with_lines {
                                        ui.add_sized(
                                            [70.0, 18.0],
                                            egui::Label::new(RichText::new(line).monospace().weak()),
                                        );
                                    }
                                    ui.add(
                                        egui::Label::new(RichText::new(message).color(theme::error_color(ui)))
                                            .wrap()
                                            .selectable(true),
                                    );
                                });
                                if index + 1 < errors.len() {
                                    ui.separator();
                                }
                            }
                        });
                    });
            }
        }
        if run {
            self.validate(shared);
        }
    }
}
