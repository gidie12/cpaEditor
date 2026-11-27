//! CPA Validation tab: validates the CPA against the schema (validator_tab.py).

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

/// The schema errors of a validation, with where they are and what to do about them.
fn problems(document: &Document, errors: &[cpa_core::validation::ValidationError]) -> Vec<Problem> {
    errors
        .iter()
        .map(|e| Problem {
            location: if e.line > 0 { Some(format!("Line {}", e.line)) } else { idref_location(document, &e.message) },
            advice: advice(&e.message),
            message: e.message.clone(),
        })
        .collect()
}

/// Shows the automatic validation that the main window runs after every change.
#[derive(Default)]
pub struct ValidationTab;

impl Tab for ValidationTab {
    fn title(&self) -> &'static str {
        "CPA Validation"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        let Some(document) = &shared.document else {
            no_cpa(ui, shared);
            return;
        };
        let Some(validation) = &shared.validation else {
            theme::empty_state(ui, "Validating…", "The CPA is checked against the ebXML CPP/CPA 2.0 schema.", |_| {});
            return;
        };

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            match &validation.outcome {
                Ok(errors) if errors.is_empty() => theme::badge(ui, "Valid", theme::ok_color(ui)),
                Ok(errors) => theme::badge(ui, &count_text(errors.len()), theme::error_color(ui)),
                Err(_) => theme::badge(ui, "Validation failed", theme::error_color(ui)),
            }
            ui.label(
                RichText::new(format!(
                    "Checked automatically after every change · last check {}",
                    validation.time.format("%H:%M:%S")
                ))
                .weak(),
            );
        });
        ui.add_space(8.0);
        match &validation.outcome {
            Ok(errors) if errors.is_empty() => {
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
            Err(message) => {
                ui.label(RichText::new(message).color(theme::error_color(ui)));
            }
            Ok(errors) => {
                let problems = problems(document, errors);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for problem in &problems {
                        error_card(ui, problem);
                        ui.add_space(6.0);
                    }
                });
            }
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
        Some("Add the element with this id, or change the reference to an existing id. For a certId: the Certificates tab can point the reference at another certId or remove it.")
    } else if message.contains("is not a valid value") || message.contains("is not an element of the set") {
        Some("Change the value to one that the schema allows.")
    } else if message.contains("Missing child element") {
        Some("Add the missing element in the XML editor.")
    } else {
        None
    }
}
