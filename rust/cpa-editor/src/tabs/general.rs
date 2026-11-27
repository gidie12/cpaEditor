//! General tab (general_tab.py): the CPA id and status, the name, id and id type of both partners, and the start
//! and end date with their Zulu and local time. Every edit is written to the CPA right away, so nothing is lost when
//! the user saves or switches tabs while a field still has the focus.

use chrono::{DateTime, TimeZone, Utc};
use cpa_core::certificates::{certificate_expiries, end_date_from_certificates, end_date_message};
use cpa_core::cpa::{read_general, set_general, set_start_date_to_now, GeneralField, CPA_STATUS_OPTIONS};
use cpa_core::dates::{describe_cpa_date, parse_iso_datetime, INVALID_DATE_MESSAGE, NO_TIMEZONE_MESSAGE};
use cpa_core::xml::Document;
use eframe::egui::{self, Color32, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme::{self, PARTNER_COLORS};

/// Width of the label column, the same in every section so all fields line up.
const LABEL_WIDTH: f32 = 130.0;
/// The form does not grow wider than this; long lines of empty field are hard to read.
const FORM_WIDTH: f32 = 720.0;
/// Two columns of at least this width side by side, otherwise one column.
const COLUMN_MIN_WIDTH: f32 = 520.0;
const COLUMN_GAP: f32 = 40.0;
/// Both date buttons have this width, so the date fields end at the same edge.
const DATE_BUTTON_WIDTH: f32 = 168.0;
/// An end date closer than this is shown as a warning.
const EXPIRY_WARNING_DAYS: i64 = 30;
const DATE_HINT: &str = "e.g. 2026-10-09T10:44:32Z";

/// A text field of the form with its edit buffer.
struct Field {
    field: GeneralField,
    /// The name the Python version uses in its log messages, e.g. `partyNamePartnerA`.
    log_name: &'static str,
    label: &'static str,
    value: String,
    /// Why the last change could not be written to the CPA.
    error: Option<String>,
    /// Changed since it got the focus; logged once when the focus leaves.
    edited: bool,
}

impl Field {
    fn new(field: GeneralField, log_name: &'static str, label: &'static str) -> Self {
        Field { field, log_name, label, value: String::new(), error: None, edited: false }
    }
}

const CPA_ID: usize = 0;
const START_DATE: usize = 7;
const END_DATE: usize = 8;

fn fields() -> Vec<Field> {
    vec![
        Field::new(GeneralField::CpaId, "CPAId", "CPA ID"),
        Field::new(GeneralField::PartyName(0), "partyNamePartnerA", "Party name"),
        Field::new(GeneralField::PartyId(0), "partyIdPartnerA", "Party ID"),
        Field::new(GeneralField::PartyIdType(0), "partyIdTypePartnerA", "Party ID type"),
        Field::new(GeneralField::PartyName(1), "partyNamePartnerB", "Party name"),
        Field::new(GeneralField::PartyId(1), "partyIdPartnerB", "Party ID"),
        Field::new(GeneralField::PartyIdType(1), "partyIdTypePartnerB", "Party ID type"),
        Field::new(GeneralField::StartDate, "cpaStartDate", "Start date"),
        Field::new(GeneralField::EndDate, "cpaEndDate", "End date"),
    ]
}

pub struct GeneralTab {
    fields: Vec<Field>,
    status: String,
    status_error: Option<String>,
    built_for: Option<u64>,
    generation: u64,
    /// Per partner the PartyIds after the first one (id, type); listed, not edited here.
    other_party_ids: [Vec<(String, String)>; 2],
}

impl Default for GeneralTab {
    fn default() -> Self {
        GeneralTab {
            fields: fields(),
            status: String::new(),
            status_error: None,
            built_for: None,
            generation: 0,
            other_party_ids: Default::default(),
        }
    }
}

/// Which date button was pressed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DateAction {
    StartNow,
    EndFromCertificates,
}

impl GeneralTab {
    /// Fills the fields from the CPA.
    fn load(&mut self, document: &Document) {
        let general = read_general(document);
        self.other_party_ids = [0, 1].map(|partner| cpa_core::cpa::other_party_ids(document, partner));
        for field in &mut self.fields {
            field.value = match field.field {
                GeneralField::CpaId => general.cpa_id.clone(),
                GeneralField::Status => general.status.clone(),
                GeneralField::PartyName(i) => general.party_names[i].clone(),
                GeneralField::PartyId(i) => general.party_ids[i].clone(),
                GeneralField::PartyIdType(i) => general.party_id_types[i].clone(),
                GeneralField::StartDate => general.start_date.clone(),
                GeneralField::EndDate => general.end_date.clone(),
            };
            field.error = None;
            field.edited = false;
        }
        self.status = general.status;
        self.status_error = None;
    }

    /// Writes one field to the CPA. The fields are not reloaded afterwards, so a value the CPA did not accept stays
    /// in the field next to its error.
    fn write(&mut self, index: usize, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else { return };
        let field = &mut self.fields[index];
        match set_general(document, field.field, &field.value) {
            Ok(()) => {
                field.error = None;
                shared.changed();
                self.built_for = Some(shared.revision);
            }
            Err(e) => {
                // Logged once per message, not on every key
                if field.error.as_deref() != Some(e.as_str()) {
                    shared.log.error(format!("Error while changing element: {e}"));
                }
                field.error = Some(e);
            }
        }
    }

    fn write_status(&mut self, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else { return };
        match set_general(document, GeneralField::Status, &self.status) {
            Ok(()) => {
                self.status_error = None;
                shared.changed();
                self.built_for = Some(shared.revision);
                shared.log.debug(format!("Dropdown changed cpaStatus New selected value: {}", self.status));
            }
            Err(e) => {
                shared.log.error(format!("Error while changing element: {e}"));
                self.status_error = Some(e);
            }
        }
    }

    /// "Set to now" and "Set from certificates", with the log messages of the Python version.
    fn date_action(&mut self, action: DateAction, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else {
            shared.log.error("No CPA loaded");
            return;
        };
        match action {
            DateAction::StartNow => match set_start_date_to_now(document) {
                Ok(start_date) => {
                    shared.log.info(format!("Start date set to {start_date} (current date and time, UTC)"));
                }
                Err(e) => {
                    shared.log.error(format!("Error while changing element: {e}"));
                    return;
                }
            },
            DateAction::EndFromCertificates => {
                for error in certificate_expiries(document).1 {
                    shared.log.error(error);
                }
                match end_date_from_certificates(document) {
                    Ok((end_date, expiry, debug_lines)) => {
                        for line in debug_lines {
                            shared.log.debug(line);
                        }
                        shared.log.info(end_date_message(&end_date, &expiry));
                        if expiry.not_after < Utc::now() {
                            shared.log.warning("This certificate has already expired, the end date is in the past");
                        }
                    }
                    Err(e) => {
                        shared.log.error(e);
                        return;
                    }
                }
            }
        }
        shared.changed();
        // The date field shows the new value even while it has the focus
        let document = shared.document.as_ref().expect("checked above");
        let index = if action == DateAction::StartNow { START_DATE } else { END_DATE };
        let value = if action == DateAction::StartNow {
            read_general(document).start_date
        } else {
            read_general(document).end_date
        };
        self.fields[index].value = value;
        self.fields[index].error = None;
        self.built_for = Some(shared.revision);
    }

    /// A text field in the second column of a form grid, named by its label for screen readers.
    fn text_field(&mut self, ui: &mut Ui, index: usize, label: egui::Id, width: f32, shared: &mut Shared) {
        let field = &mut self.fields[index];
        let invalid = matches!(field.field, GeneralField::StartDate | GeneralField::EndDate)
            && !field.value.trim().is_empty()
            && moment(&field.value).is_none();
        let mut edit = egui::TextEdit::singleline(&mut field.value).id_salt(field.log_name).desired_width(width);
        if matches!(field.field, GeneralField::StartDate | GeneralField::EndDate) {
            edit = edit.hint_text(DATE_HINT);
        }
        let response = theme::with_error_border(ui, invalid, |ui| ui.add(edit)).labelled_by(label);
        if response.changed() {
            field.edited = true;
            self.write(index, shared);
        }
        let field = &mut self.fields[index];
        if response.lost_focus() && field.edited {
            field.edited = false;
            if field.error.is_none() {
                shared.log.debug(format!("Field '{}' changed: New value: {}", field.log_name, field.value));
            }
        }
    }

    /// The status drop-down list: the values of the schema, plus the current value when it is another one.
    fn status_combo(&mut self, ui: &mut Ui, label: egui::Id) -> bool {
        let mut options: Vec<&str> = CPA_STATUS_OPTIONS.to_vec();
        if !self.status.is_empty() && !options.contains(&self.status.as_str()) {
            options.insert(0, self.status.as_str());
        }
        let mut chosen = None;
        let response = egui::ComboBox::from_id_salt("general_status")
            .width(ui.available_width())
            .selected_text(self.status.as_str())
            .show_ui(ui, |ui| {
                for option in &options {
                    if ui.selectable_label(self.status == *option, *option).clicked() && self.status != *option {
                        chosen = Some(option.to_string());
                    }
                }
            });
        response.response.labelled_by(label);
        match chosen {
            Some(value) => {
                self.status = value;
                true
            }
            None => false,
        }
    }

    /// The error of a field below it, in the field column.
    fn field_error(ui: &mut Ui, error: &Option<String>) {
        if let Some(error) = error {
            ui.label("");
            ui.label(RichText::new(format!("Not saved in the CPA: {error}")).small().color(theme::error_color(ui)));
            ui.end_row();
        }
    }

    fn text_row(&mut self, ui: &mut Ui, index: usize, shared: &mut Shared) {
        let label = ui.label(self.fields[index].label).id;
        let width = ui.available_width();
        self.text_field(ui, index, label, width, shared);
        ui.end_row();
        Self::field_error(ui, &self.fields[index].error);
    }

    fn date_row(&mut self, ui: &mut Ui, index: usize, shared: &mut Shared) -> Option<DateAction> {
        let mut action = None;
        let label = ui.label(self.fields[index].label).id;
        ui.horizontal(|ui| {
            let spacing = ui.spacing().item_spacing.x;
            let width = (ui.available_width() - DATE_BUTTON_WIDTH - spacing).max(120.0);
            self.text_field(ui, index, label, width, shared);
            let size = egui::vec2(DATE_BUTTON_WIDTH, ui.spacing().interact_size.y);
            if index == START_DATE {
                let button = ui.add(egui::Button::new("Set to now").min_size(size));
                if button.on_hover_text("Set the start date to the current date and time (UTC)").clicked() {
                    action = Some(DateAction::StartNow);
                }
            } else {
                let button = ui.add(egui::Button::new("Set from certificates").min_size(size));
                let text = "Set the end date to the expiry date of the certificate that expires first";
                if button.on_hover_text(text).clicked() {
                    action = Some(DateAction::EndFromCertificates);
                }
            }
        });
        ui.end_row();

        // The same moment in Zulu time and in the timezone of this computer
        let description = describe_cpa_date(&self.fields[index].value, None);
        ui.label("");
        let text = if description.is_empty() { "No date set".to_string() } else { description.clone() };
        let rich = RichText::new(text).small();
        let rich = if description == INVALID_DATE_MESSAGE || description == NO_TIMEZONE_MESSAGE {
            rich.color(theme::error_color(ui))
        } else {
            rich.weak()
        };
        ui.label(rich);
        ui.end_row();
        // An end date in the past is the mistake to see here, not only in the log
        if index == END_DATE {
            if let Some(end) = moment(&self.fields[index].value).filter(|end| *end < Utc::now()) {
                let days = (Utc::now() - end).num_days();
                let ago = match days {
                    0 => "today".to_string(),
                    1 => "1 day ago".to_string(),
                    2..=59 => format!("{days} days ago"),
                    60..=729 => format!("{} months ago", days / 30),
                    _ => format!("{} years ago", days / 365),
                };
                ui.label("");
                let text = format!("The CPA expired {ago}. Set a later end date, or renew the certificates first.");
                ui.add(egui::Label::new(RichText::new(text).color(theme::warning_color(ui))).wrap());
                ui.end_row();
            }
        }
        Self::field_error(ui, &self.fields[index].error);
        action
    }

    fn agreement_sections(&mut self, ui: &mut Ui, shared: &mut Shared) -> Option<DateAction> {
        section_heading(ui, "Agreement", None);
        form_grid("general_agreement").show(ui, |ui| {
            self.text_row(ui, CPA_ID, shared);
            let label = ui.label("Status").id;
            if self.status_combo(ui, label) {
                self.write_status(shared);
            }
            ui.end_row();
            Self::field_error(ui, &self.status_error);
        });
        section_heading(ui, "Validity", None);
        let mut action = None;
        form_grid("general_validity").show(ui, |ui| {
            action = self.date_row(ui, START_DATE, shared);
            action = action.or(self.date_row(ui, END_DATE, shared));
        });
        action
    }

    fn partner_sections(&mut self, ui: &mut Ui, shared: &mut Shared) {
        for (partner, first) in [(0, 1), (1, 4)] {
            let title = if partner == 0 { "Partner A" } else { "Partner B" };
            section_heading(ui, title, Some(PARTNER_COLORS[partner]));
            form_grid(&format!("general_partner_{partner}")).show(ui, |ui| {
                for index in first..first + 3 {
                    self.text_row(ui, index, shared);
                }
                // A party with more PartyIds: the fields above are the first, the others are listed here
                let others = &self.other_party_ids[partner];
                if !others.is_empty() {
                    // On the line of the first id, not centred next to the list
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.label(format!("Other party IDs ({})", others.len()));
                    });
                    ui.vertical(|ui| {
                        for (id, kind) in others {
                            ui.horizontal(|ui| {
                                ui.add(egui::Label::new(RichText::new(id).monospace()).selectable(true));
                                let kind = if kind.is_empty() { "no type" } else { kind.as_str() };
                                ui.add(egui::Label::new(RichText::new(kind).weak()).truncate().selectable(true));
                            });
                        }
                        ui.label(RichText::new("Change these in the XML editor.").small().weak());
                    });
                    ui.end_row();
                }
            });
        }
    }

    /// The number of start and end dates that are filled in but not valid.
    fn invalid_dates(&self) -> usize {
        [START_DATE, END_DATE]
            .iter()
            .filter(|&&i| !self.fields[i].value.trim().is_empty() && moment(&self.fields[i].value).is_none())
            .count()
    }

    /// The CPA id, the partner names and the status and validity badges above the form.
    fn header(&self, ui: &mut Ui) {
        let cpa_id = self.fields[CPA_ID].value.trim();
        ui.horizontal(|ui| {
            let title = if cpa_id.is_empty() { "CPA without ID" } else { cpa_id };
            ui.label(RichText::new(title).size(20.0).strong());
            ui.add_space(4.0);
            if !self.status.is_empty() {
                theme::badge(ui, &capitalise(&self.status), status_color(ui, &self.status));
            }
            if let Some((text, color)) = validity(ui, &self.fields[START_DATE].value, &self.fields[END_DATE].value) {
                theme::badge(ui, &text, color);
            } else if self.invalid_dates() > 0 {
                theme::badge(ui, "Invalid dates", theme::error_color(ui));
            }
        });
        let names: Vec<&str> =
            [1, 4].iter().map(|&i| self.fields[i].value.trim()).filter(|name| !name.is_empty()).collect();
        if names.len() == 2 {
            ui.label(RichText::new(format!("Agreement between {} and {}", names[0], names[1])).weak());
        }
    }
}

fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Signed is final (green), agreed is accepted by both partners (accent), proposed is still open (amber).
fn status_color(ui: &Ui, status: &str) -> Color32 {
    match status {
        "signed" => theme::ok_color(ui),
        // Lighter / darker than the accent, so the badge text reaches 4.5:1
        "agreed" if ui.visuals().dark_mode => Color32::from_rgb(0x93, 0xb4, 0xfb),
        "agreed" => Color32::from_rgb(0x1e, 0x40, 0xaf),
        _ => theme::warning_color(ui),
    }
}

/// A CPA date as a moment, None when it is not a valid date with a timezone.
fn moment(value: &str) -> Option<DateTime<Utc>> {
    let parsed = parse_iso_datetime(value.trim())?;
    let offset = parsed.offset?;
    Some(offset.from_local_datetime(&parsed.naive).single()?.with_timezone(&Utc))
}

/// Whether the CPA is in force now, as a badge text and colour. None when a date is missing or not valid.
fn validity(ui: &Ui, start: &str, end: &str) -> Option<(String, Color32)> {
    let (start, end) = (moment(start)?, moment(end)?);
    let now = Utc::now();
    if end < now {
        return Some(("Expired".into(), theme::error_color(ui)));
    }
    if start > now {
        return Some(("Not yet in force".into(), theme::warning_color(ui)));
    }
    let days = (end - now).num_days();
    if days < EXPIRY_WARNING_DAYS {
        let text = if days == 1 { "Expires in 1 day".to_string() } else { format!("Expires in {days} days") };
        return Some((text, theme::warning_color(ui)));
    }
    Some(("In force".into(), theme::ok_color(ui)))
}

/// A section heading (`theme::section`); a partner section gets the coloured bar of the partner rows of the
/// Collaboration Role tab before its text.
fn section_heading(ui: &mut Ui, text: &str, bar: Option<Color32>) {
    ui.add_space(8.0);
    match bar {
        None => theme::section(ui, text),
        Some(color) => {
            ui.add_space(10.0);
            // Only as high as the text, so the heading lines up with the sections without bar
            let height = ui.text_style_height(&egui::TextStyle::Body) + 2.0;
            let layout = egui::Layout::left_to_right(egui::Align::Center);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), height), layout, |ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, height - 2.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, color);
                ui.label(RichText::new(text).size(15.0).strong().color(ui.visuals().strong_text_color()));
            });
            ui.separator();
            ui.add_space(2.0);
        }
    }
    ui.add_space(4.0);
}

fn form_grid(id: &str) -> egui::Grid {
    egui::Grid::new(id).num_columns(2).spacing([16.0, 8.0]).min_col_width(LABEL_WIDTH)
}

impl Tab for GeneralTab {
    fn title(&self) -> &'static str {
        "General"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            *self = GeneralTab { generation: shared.generation, ..Default::default() };
        }
        let Some(document) = shared.document.as_ref() else {
            no_cpa(ui, shared);
            return;
        };
        if self.built_for != Some(shared.revision) {
            self.load(document);
            self.built_for = Some(shared.revision);
        }
        shared.invalid_dates = self.invalid_dates();

        let mut action = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal_top(|ui| {
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    let available = ui.available_width() - 16.0;
                    self.header(ui);
                    if available >= 2.0 * COLUMN_MIN_WIDTH + COLUMN_GAP {
                        // Wide window: the agreement and its validity left, the two partners right
                        let width = ((available - COLUMN_GAP) / 2.0).min(FORM_WIDTH);
                        let spacing = ui.spacing().item_spacing.x;
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                action = self.agreement_sections(ui, shared);
                            });
                            ui.add_space(COLUMN_GAP - spacing);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                self.partner_sections(ui, shared);
                            });
                        });
                    } else {
                        ui.set_max_width(FORM_WIDTH.min(available));
                        action = self.agreement_sections(ui, shared);
                        self.partner_sections(ui, shared);
                    }
                    ui.add_space(12.0);
                });
            });
        });
        if let Some(action) = action {
            self.date_action(action, shared);
        }
    }
}
