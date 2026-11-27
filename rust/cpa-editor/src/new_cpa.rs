//! New CPA from this one: the question for the new CPAId and dates, and whether to reset the status and strip the
//! certificates. The CPA itself is made by `cpa_core::cpa::new_cpa_from`.

use cpa_core::cpa::{check_new_cpa_options, GeneralField, NewCpaOptions};
use cpa_core::xml::Document;
use eframe::egui::{self, RichText, Ui};

use crate::theme;

const DATE_HINT: &str = "e.g. 2026-10-09T10:44:32Z";

pub struct NewCpaDialog {
    pub options: NewCpaOptions,
    /// The CPAId of the loaded CPA, shown as the source.
    source_id: String,
}

/// What the user chose.
pub enum NewCpaAnswer {
    Create(NewCpaOptions),
    Cancel,
}

impl NewCpaDialog {
    pub fn new(document: &Document) -> Self {
        NewCpaDialog {
            options: cpa_core::cpa::new_cpa_defaults(document),
            source_id: cpa_core::cpa::read_general(document).cpa_id,
        }
    }

    /// Shows the question; returns the answer once it is given. Escape is Cancel.
    pub fn show(&mut self, ctx: &egui::Context, document: &Document) -> Option<NewCpaAnswer> {
        let errors = check_new_cpa_options(document, &self.options);
        let error = |field: GeneralField| errors.iter().find(|(f, _)| *f == field).map(|(_, e)| e.as_str());
        let mut answer = None;
        let modal = egui::Modal::new(egui::Id::unique("new_cpa")).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.heading("New CPA from this one");
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "A copy of {} with a new CPAId and dates. The loaded file is not changed; the new CPA is \
                     written when you save it.",
                    if self.source_id.is_empty() { "the loaded CPA".to_string() } else { self.source_id.clone() }
                ))
                .weak(),
            );
            ui.add_space(10.0);
            egui::Grid::new("new_cpa_fields").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                field(ui, "CPA ID", &mut self.options.cpa_id, "", error(GeneralField::CpaId));
                field(ui, "Start date", &mut self.options.start_date, DATE_HINT, error(GeneralField::StartDate));
                field(ui, "End date", &mut self.options.end_date, DATE_HINT, error(GeneralField::EndDate));
            });
            ui.add_space(8.0);
            ui.checkbox(&mut self.options.reset_status, "Set the status to proposed");
            ui.checkbox(&mut self.options.strip_certificates, "Remove all certificates");
            if self.options.strip_certificates {
                ui.label(
                    RichText::new(
                        "Also removes transport TLS, trust anchors and message signing and encryption; https \
                         endpoints become http.",
                    )
                    .color(theme::warning_color(ui)),
                );
            }
            ui.add_space(12.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 28.0))).clicked() {
                    answer = Some(NewCpaAnswer::Cancel);
                }
                let create = ui.add_enabled(errors.is_empty(), theme::primary_button("Create CPA"));
                if create.clicked() {
                    answer = Some(NewCpaAnswer::Create(self.options.clone()));
                }
            });
        });
        if modal.should_close() && answer.is_none() {
            answer = Some(NewCpaAnswer::Cancel);
        }
        answer
    }
}

/// A labelled text field with an error border and the error below it.
fn field(ui: &mut Ui, label: &str, value: &mut String, hint: &str, error: Option<&str>) {
    let label = ui.label(label);
    ui.vertical(|ui| {
        let edit = egui::TextEdit::singleline(value).desired_width(360.0).hint_text(hint);
        theme::with_error_border(ui, error.is_some(), |ui| ui.add(edit)).labelled_by(label.id);
        if let Some(error) = error {
            ui.label(RichText::new(error).color(theme::error_color(ui)).small());
        }
    });
    ui.end_row();
}
