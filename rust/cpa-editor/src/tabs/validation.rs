//! CPA Validation tab: validates the CPA against the schema (validator_tab.py).

use chrono::{DateTime, Local};
use cpa_core::validation::validate_document;
use cpa_core::xml::Document;
use eframe::egui::{self, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme;

/// One schema error: the message and where it is (a line number or an element path), when known.
struct Problem {
    message: String,
    location: Option<String>,
    /// What to do about it, when the kind of error is known.
    advice: Option<&'static str>,
}

/// The outcome of the last validation.
enum Outcome {
    Valid,
    Errors(Vec<Problem>),
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
                shared.log.info("Validation found no errors");
                Outcome::Valid
            }
            Ok(errors) => {
                shared.log.warning(format!("Validation found {}", count_text(errors.len())));
                let problems = errors
                    .iter()
                    .map(|e| Problem {
                        location: if e.line > 0 {
                            Some(format!("Line {}", e.line))
                        } else {
                            idref_location(document, &e.message)
                        },
                        advice: advice(&e.message),
                        message: e.message.clone(),
                    })
                    .collect();
                Outcome::Errors(problems)
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
            run = theme::action_button(ui, "Validate again", false, true).clicked();
            ui.add_space(8.0);
            match outcome {
                Outcome::Valid => theme::badge(ui, "Valid", theme::ok_color(ui)),
                Outcome::Errors(errors) => theme::badge(ui, &count_text(errors.len()), theme::error_color(ui)),
                Outcome::Failed(_) => theme::badge(ui, "Validation failed", theme::error_color(ui)),
            }
            ui.label(RichText::new(format!("Last run {}", time.format("%H:%M:%S"))).weak());
            if *revision != shared.revision {
                ui.label(RichText::new("· Changed since the last run, validate again").color(theme::warning_color(ui)));
            }
        });
        ui.add_space(8.0);
        match outcome {
            Outcome::Valid => {
                let dirty = shared.is_dirty();
                let text = if dirty {
                    "The CPA is valid against the schema. Save it to keep your changes."
                } else {
                    "The CPA is valid against the schema."
                };
                let mut save = false;
                theme::empty_state(ui, "No errors found", text, |ui| {
                    // Only a next step when there is something to save; the bottom bar has Save CPA… as well
                    if dirty {
                        save = ui.add(theme::primary_button("Save CPA…")).clicked();
                    }
                });
                if save {
                    shared.request_save = true;
                }
            }
            Outcome::Failed(message) => {
                ui.label(RichText::new(message).color(theme::error_color(ui)));
            }
            Outcome::Errors(errors) => {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for problem in errors {
                        error_card(ui, problem);
                        ui.add_space(6.0);
                    }
                });
            }
        }
        if run {
            self.validate(shared);
        }
    }
}

fn count_text(count: usize) -> String {
    if count == 1 {
        "1 error".to_string()
    } else {
        format!("{count} errors")
    }
}

/// One error as a card: a red bar, the message in normal text and the location in weak monospace.
fn error_card(ui: &mut Ui, problem: &Problem) {
    let error = theme::error_color(ui);
    let response = egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(4)
        .inner_margin(egui::Margin { left: 14, right: 10, top: 8, bottom: 8 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let text = RichText::new(&problem.message).color(ui.visuals().strong_text_color());
            ui.add(egui::Label::new(text).wrap().selectable(true));
            if let Some(location) = &problem.location {
                ui.add(egui::Label::new(RichText::new(location).monospace().weak()).wrap().selectable(true));
            }
            if let Some(advice) = problem.advice {
                ui.add_space(2.0);
                ui.label(RichText::new(advice));
            }
        })
        .response;
    let rect = response.rect;
    let bar = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height()));
    ui.painter().rect_filled(bar, egui::CornerRadius { nw: 4, sw: 4, ne: 0, se: 0 }, error);
}

/// Where a dangling IDREF is used: the path of the element whose attribute holds the missing id.
fn idref_location(document: &Document, message: &str) -> Option<String> {
    let id = message.strip_prefix("IDREF '")?.split('\'').next()?;
    let root = document.root()?;
    let (element, attribute) = document.descendants(root).filter(|&n| document.is_element(n)).find_map(|n| {
        document.attributes(n).iter().find(|a| a.value == id).map(|a| (n, a.name.local.clone()))
    })?;
    // A PartyInfo is named after its partner: a CPA has two of them
    let mut names: Vec<String> = document
        .ancestors(element)
        .filter(|&n| document.is_element(n))
        .map(|n| match document.local_name(n) {
            "PartyInfo" => match document.attributes(n).iter().find(|a| a.name.local == "partyName") {
                Some(name) => format!("PartyInfo ({})", name.value),
                None => "PartyInfo".to_string(),
            },
            name => name.to_string(),
        })
        .collect();
    names.reverse();
    Some(format!("Used in {} @{attribute}", names.join(" / ")))
}

/// Advice for the errors whose cause is clear from the message.
fn advice(message: &str) -> Option<&'static str> {
    if message.starts_with("IDREF '") {
        Some("Add the element with this id (for a certificate: upload it on the Certificates tab), or change the reference to an existing id.")
    } else if message.contains("is not a valid value") || message.contains("is not an element of the set") {
        Some("Change the value to one that the schema allows.")
    } else if message.contains("Missing child element") {
        Some("Add the missing element in the XML editor.")
    } else {
        None
    }
}
