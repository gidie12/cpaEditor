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
    party_names: Vec<String>,
    /// The code of each partner in the ids of the loaded CPA (`BROKER` in `BROKER_T_HTTPS`), when they have one.
    codes: Vec<Option<String>>,
    /// The CPA ID was typed: a changed PartyId no longer changes it.
    cpa_id_edited: bool,
}

/// What the user chose.
pub enum NewCpaAnswer {
    Create(NewCpaOptions),
    Cancel,
}

impl NewCpaDialog {
    pub fn new(document: &Document) -> Self {
        let parties = cpa_core::cpa::party_infos(document);
        NewCpaDialog {
            options: cpa_core::cpa::new_cpa_defaults(document),
            source_id: cpa_core::cpa::read_general(document).cpa_id,
            party_names: parties.iter().map(|&p| cpa_core::collaboration_roles::party_name(document, p)).collect(),
            codes: (0..parties.len()).map(|index| cpa_core::cpa::id_code(document, index)).collect(),
            cpa_id_edited: false,
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
                    "A copy of {} with a new CPA ID, dates and PartyIds. The loaded file is not changed; the new CPA is \
                     written when you save it.",
                    if self.source_id.is_empty() { "the loaded CPA".to_string() } else { self.source_id.clone() }
                ))
                .weak(),
            );
            ui.add_space(10.0);
            let mut party_id_changed = false;
            egui::Grid::new("new_cpa_fields").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                if field(ui, "CPA ID", &mut self.options.cpa_id, "", error(GeneralField::CpaId)) {
                    self.cpa_id_edited = true;
                }
                field(ui, "Start date", &mut self.options.start_date, DATE_HINT, error(GeneralField::StartDate));
                field(ui, "End date", &mut self.options.end_date, DATE_HINT, error(GeneralField::EndDate));
                for index in 0..self.options.party_ids.len() {
                    let label = format!("PartyId {}", self.party_names.get(index).map(String::as_str).unwrap_or(""));
                    let field_error = error(GeneralField::PartyId(index));
                    party_id_changed |= field(ui, &label, &mut self.options.party_ids[index], "", field_error);
                }
            });
            if party_id_changed && !self.cpa_id_edited {
                let renames = cpa_core::cpa::code_renames(document, &self.options.party_ids);
                self.options.cpa_id =
                    cpa_core::cpa::next_cpa_id(&cpa_core::cpa::replace_codes(self.source_id.trim(), &renames));
            }
            ui.add_space(8.0);
            ui.checkbox(&mut self.options.rename_ids, "Rename the ids with the PartyIds");
            if self.options.rename_ids {
                let mut lines = Vec::new();
                for (index, party_id) in self.options.party_ids.iter().enumerate() {
                    let name = self.party_names.get(index).map(String::as_str).unwrap_or("");
                    lines.push(match self.codes.get(index).cloned().flatten() {
                        Some(code) if code == party_id.trim() => format!("{name}: the ids already use {code}"),
                        Some(code) => format!("{name}: {code}_… becomes {}_…", party_id.trim()),
                        None => format!("{name}: its ids have no common code and keep their names"),
                    });
                }
                if let Ok(renames) = cpa_core::cpa::id_renames(document, &self.options.party_ids) {
                    lines.push(match renames.len() {
                        0 => "No id changes".to_string(),
                        1 => "1 id or reference is renamed".to_string(),
                        n => format!("{n} ids and references are renamed"),
                    });
                }
                for line in lines {
                    ui.label(RichText::new(format!("     {line}")).small().weak());
                }
            }
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

/// A labelled text field with an error border and the error below it. Returns true when the value was changed.
fn field(ui: &mut Ui, label: &str, value: &mut String, hint: &str, error: Option<&str>) -> bool {
    let label = ui.label(label);
    let changed = ui
        .vertical(|ui| {
            let edit = egui::TextEdit::singleline(value).desired_width(360.0).hint_text(hint);
            let response = theme::with_error_border(ui, error.is_some(), |ui| ui.add(edit)).labelled_by(label.id);
            if let Some(error) = error {
                ui.add(egui::Label::new(RichText::new(error).color(theme::error_color(ui)).small()).wrap());
            }
            response.changed()
        })
        .inner;
    ui.end_row();
    changed
}
