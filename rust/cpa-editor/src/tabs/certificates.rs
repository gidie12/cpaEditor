//! Certificates tab (certificates_tab.py): per partner the certIds with their certificate chain, the expiry status,
//! which elements refer to each certId, and the actions of the Python right-click menu: upload a PEM chain that
//! replaces the KeyInfo, download the chain as PEM and copy the KeyInfo XML.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use cpa_core::certificates::{
    certificate_entries, certificate_usages, key_info_certificates, key_info_from_pem, key_info_to_pem, pem_file_name,
    replace_key_info, CertificateDetails, CertificateType,
};
use cpa_core::collaboration_roles::party_name;
use cpa_core::cpa::party_infos;
use cpa_core::dates::format_cpa_date;
use cpa_core::xml::NodeId;
use cpa_core::DS_NAMESPACE;
use eframe::egui::{self, Color32, RichText, Ui};

use super::{no_cpa, Shared, Tab};
use crate::theme::{self, PARTNER_COLORS};
use crate::tree_table::{RowKey, TreeRow, TreeTable};
use crate::widgets::{Answer, Confirm};

/// A certificate that expires within this many days is shown as expiring soon (amber).
const EXPIRING_SOON_DAYS: i64 = 30;

/// The expiry status of a certificate; the order is from best to worst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Valid,
    /// Expires within [`EXPIRING_SOON_DAYS`]; the number of whole days left.
    ExpiringSoon(std::cmp::Reverse<i64>),
    Expired,
}

impl Status {
    fn of(certificate: &CertificateDetails, now: DateTime<Utc>) -> Status {
        if certificate.expired {
            return Status::Expired;
        }
        let days = (certificate.not_after - now).num_days();
        if days < EXPIRING_SOON_DAYS {
            Status::ExpiringSoon(std::cmp::Reverse(days))
        } else {
            Status::Valid
        }
    }

    fn text(self) -> String {
        match self {
            Status::Valid => "Valid".to_string(),
            Status::Expired => "Expired".to_string(),
            Status::ExpiringSoon(std::cmp::Reverse(0)) => "Expires today".to_string(),
            Status::ExpiringSoon(std::cmp::Reverse(1)) => "Expires in 1 day".to_string(),
            Status::ExpiringSoon(std::cmp::Reverse(days)) => format!("Expires in {days} days"),
        }
    }

    fn color(self, ui: &Ui) -> Color32 {
        match self {
            Status::Valid => theme::ok_color(ui),
            Status::ExpiringSoon(_) => theme::warning_color(ui),
            Status::Expired => theme::error_color(ui),
        }
    }

    /// The next step for this certId is to upload a new certificate.
    fn needs_replacement(self) -> bool {
        self != Status::Valid
    }
}

/// The summary of a certificate as the Python tree showed it: `{cn} - expires {date}`, with ` (EXPIRED)`.
fn summary(certificate: &CertificateDetails) -> String {
    let mut text = format!("{} - expires {}", certificate.common_name, format_cpa_date(certificate.not_after));
    if certificate.expired {
        text.push_str(" (EXPIRED)");
    }
    text
}

fn plural(count: usize, one: &str, many: &str) -> String {
    if count == 1 { format!("1 {one}") } else { format!("{count} {many}") }
}

/// A Certificate element of a partner with what the tab shows of it.
struct Entry {
    partner: String,
    cert_id: String,
    certificate: NodeId,
    key_info: Option<NodeId>,
    certificates: Vec<CertificateDetails>,
    errors: Vec<String>,
    usages: Vec<String>,
}

impl Entry {
    /// The worst status of the chain; None without certificates.
    fn status(&self, now: DateTime<Utc>) -> Option<Status> {
        self.certificates.iter().map(|c| Status::of(c, now)).max()
    }

    fn first_expiring(&self) -> Option<&CertificateDetails> {
        self.certificates.iter().min_by_key(|c| c.not_after)
    }

    fn leaf(&self) -> Option<&CertificateDetails> {
        self.certificates.iter().find(|c| c.kind == CertificateType::Leaf).or(self.certificates.first())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Upload(usize),
    Download(usize),
    CopyKeyInfo(usize),
}

/// A PEM file that was read and waits for the confirmation to replace the KeyInfo.
#[derive(Clone)]
struct PendingUpload {
    certificate: NodeId,
    pem: String,
}

pub struct CertificatesTab {
    tree: TreeTable,
    rows: Vec<TreeRow>,
    entries: Vec<Entry>,
    /// Row key of a certId row to the index in `entries`.
    row_entry: HashMap<RowKey, usize>,
    /// The moment the status was computed.
    now: DateTime<Utc>,
    built_for: Option<u64>,
    generation: u64,
    confirm: Option<Confirm<PendingUpload>>,
}

impl Default for CertificatesTab {
    fn default() -> Self {
        CertificatesTab {
            tree: TreeTable::new(
                "certificates",
                &[("Partner / certId", 300.0), ("Status", 140.0), ("First expiry", 190.0), ("Leaf certificate", 200.0)],
            ),
            rows: Vec::new(),
            entries: Vec::new(),
            row_entry: HashMap::new(),
            now: Utc::now(),
            built_for: None,
            generation: 0,
            confirm: None,
        }
    }
}

fn key(node: NodeId) -> RowKey {
    node.index() as RowKey
}

impl CertificatesTab {
    /// Reads the certificates of the CPA and rebuilds the rows. Problems are logged here, once per change of the
    /// CPA, like the Python tab logs them when it is loaded.
    fn build(&mut self, shared: &mut Shared) {
        self.rows.clear();
        self.entries.clear();
        self.row_entry.clear();
        self.now = Utc::now();
        let Some(document) = shared.document.as_ref() else { return };
        let mut errors = Vec::new();
        let usages = certificate_usages(document);
        for (index, party_info) in party_infos(document).into_iter().enumerate() {
            let (entries, entry_errors) = certificate_entries(document, party_info);
            errors.extend(entry_errors);
            if entries.is_empty() {
                continue;
            }
            let partner = party_name(document, party_info);
            let row = TreeRow::new(key(party_info), 0, vec![format!("Partner {partner}")])
                .header(PARTNER_COLORS[index % PARTNER_COLORS.len()])
                .secondary(plural(entries.len(), "certId", "certIds"));
            self.rows.push(row);
            for entry in entries {
                let (certificates, certificate_errors) = key_info_certificates(document, entry.key_info, self.now);
                errors.extend(certificate_errors.iter().cloned());
                self.row_entry.insert(key(entry.certificate), self.entries.len());
                self.entries.push(Entry {
                    partner: partner.clone(),
                    usages: usages.get(&entry.cert_id).cloned().unwrap_or_default(),
                    cert_id: entry.cert_id,
                    certificate: entry.certificate,
                    key_info: entry.key_info,
                    certificates,
                    errors: certificate_errors,
                });
                self.rows.push(self.entry_row(self.entries.len() - 1));
            }
        }
        if self.entries.is_empty() {
            shared.log.error("No certificates found in the CPA");
        }
        for error in errors {
            shared.log.error(error);
        }
        if self.tree.selected.is_some_and(|key| !self.row_entry.contains_key(&key)) {
            self.tree.selected = None;
        }
    }

    fn entry_row(&self, index: usize) -> TreeRow {
        let entry = &self.entries[index];
        let status = entry.status(self.now);
        let cells = vec![
            entry.cert_id.clone(),
            status.map(Status::text).unwrap_or_else(|| "No certificate".to_string()),
            entry.first_expiring().map(|c| format_cpa_date(c.not_after)).unwrap_or_default(),
            entry.leaf().map(|c| c.common_name.clone()).unwrap_or_default(),
        ];
        let mut row = TreeRow::new(key(entry.certificate), 1, cells);
        if entry.usages.is_empty() {
            row = row.secondary("not used".to_string());
        }
        row
    }

    fn selected_entry(&self) -> Option<usize> {
        self.tree.selected.and_then(|key| self.row_entry.get(&key).copied())
    }

    /// The badges above the tree: how many certificates are expired or expire soon, and the next expiry.
    fn summary_bar(&self, ui: &mut Ui) {
        let mut expired = 0;
        let mut soon = 0;
        let mut total = 0;
        let mut next: Option<&CertificateDetails> = None;
        for certificate in self.entries.iter().flat_map(|e| &e.certificates) {
            total += 1;
            match Status::of(certificate, self.now) {
                Status::Expired => expired += 1,
                Status::ExpiringSoon(_) => soon += 1,
                Status::Valid => {}
            }
            if !certificate.expired && next.is_none_or(|n| certificate.not_after < n.not_after) {
                next = Some(certificate);
            }
        }
        ui.horizontal(|ui| {
            if expired > 0 {
                theme::badge(ui, &format!("{expired} expired"), theme::error_color(ui));
            }
            if soon > 0 {
                theme::badge(ui, &format!("{soon} expiring within {EXPIRING_SOON_DAYS} days"), theme::warning_color(ui));
            }
            if expired == 0 && soon == 0 && total > 0 {
                theme::badge(ui, "All valid", theme::ok_color(ui));
            }
            let mut text = format!(
                "{} in {}",
                plural(total, "certificate", "certificates"),
                plural(self.entries.len(), "certId", "certIds")
            );
            if let Some(next) = next {
                text.push_str(&format!(
                    "  ·  next expiry {} ({})",
                    format_cpa_date(next.not_after),
                    next.common_name
                ));
            }
            ui.label(RichText::new(text).weak());
        });
    }

    /// `dirty`: the CPA has unsaved changes, so Save CPA… is the primary button and Upload is not.
    fn detail_panel(&self, ui: &mut Ui, dirty: bool, action: &mut Option<Action>) {
        let Some(index) = self.selected_entry() else {
            theme::empty_state(
                ui,
                "No certId selected",
                "Select a certId on the left to see its certificate chain and where it is used.",
                |_| {},
            );
            return;
        };
        let entry = &self.entries[index];
        let status = entry.status(self.now);

        ui.add_space(4.0);
        ui.label(RichText::new(&entry.cert_id).heading().strong());
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            match status {
                Some(status) => theme::badge(ui, &status.text(), status.color(ui)),
                None => theme::badge(ui, "No certificate", theme::warning_color(ui)),
            }
            ui.label(RichText::new(format!("Partner {}", entry.partner)).weak());
        });
        for error in &entry.errors {
            ui.label(RichText::new(error).color(theme::error_color(ui)));
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let primary = !dirty && status.is_none_or(Status::needs_replacement);
            if theme::action_button(ui, "Upload certificate…", primary, true)
                .on_hover_text("Replace the KeyInfo with the certificate chain of a PEM file")
                .clicked()
            {
                *action = Some(Action::Upload(index));
            }
            let can_download = !entry.certificates.is_empty();
            if theme::action_button(ui, "Download PEM…", false, can_download)
                .on_disabled_hover_text("This KeyInfo has no certificates")
                .clicked()
            {
                *action = Some(Action::Download(index));
            }
            if theme::action_button(ui, "Copy KeyInfo", false, entry.key_info.is_some())
                .on_disabled_hover_text("This certId has no KeyInfo")
                .clicked()
            {
                *action = Some(Action::CopyKeyInfo(index));
            }
        });
        ui.label(
            RichText::new("Upload a PEM file with the chain, leaf first; it replaces the KeyInfo of this certId.")
                .small()
                .weak(),
        );

        theme::section(ui, "Used by");
        if entry.usages.is_empty() {
            ui.label(RichText::new("No element in the CPA refers to this certId.").weak());
        }
        for usage in &entry.usages {
            ui.add(egui::Label::new(format!("•  {usage}")).wrap());
        }
        theme::section(ui, "Certificate chain");
        if entry.certificates.is_empty() {
            let text = if entry.key_info.is_none() {
                "This certId has no KeyInfo."
            } else {
                "This KeyInfo has no certificates."
            };
            ui.label(RichText::new(text).weak());
        }
        for (position, certificate) in entry.certificates.iter().enumerate() {
            if position > 0 {
                ui.add_space(6.0);
            }
            let position_text = format!("{} of {}", position + 1, entry.certificates.len());
            certificate_card(ui, (index, position), &position_text, certificate, Status::of(certificate, self.now));
        }

        ui.add_space(8.0);
    }

    fn run(&mut self, action: Action, ui: &Ui, shared: &mut Shared) {
        match action {
            Action::Upload(index) => self.upload(index, shared),
            Action::Download(index) => self.download(index, shared),
            Action::CopyKeyInfo(index) => {
                let entry = &self.entries[index];
                if let (Some(document), Some(key_info)) = (shared.document.as_ref(), entry.key_info) {
                    ui.ctx().copy_text(document.element_to_xml(key_info));
                    shared.log.info(format!("KeyInfo of certId {} copied to the clipboard", entry.cert_id));
                }
            }
        }
    }

    /// "Upload Certificate": reads a PEM chain and, after confirmation when the KeyInfo has certificates, replaces
    /// the KeyInfo with it.
    fn upload(&mut self, index: usize, shared: &mut Shared) {
        let Some(path) = rfd::FileDialog::new().add_filter("Certificate files", &["cer", "crt", "pem"]).pick_file()
        else {
            return;
        };
        let entry = &self.entries[index];
        let pem = match read_text(&path) {
            Ok(pem) => pem,
            Err(e) => {
                shared.log.error(format!("Error while uploading certificate: {e}"));
                return;
            }
        };
        let pending = PendingUpload { certificate: entry.certificate, pem };
        if entry.certificates.is_empty() {
            self.apply_upload(pending, shared);
            return;
        }
        let count = pending.pem.matches("-----END CERTIFICATE-----").count();
        let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.confirm = Some(Confirm {
            title: "Replace certificate".to_string(),
            text: format!(
                "Replace the KeyInfo of certId {} with the {} in {file}? The current {} removed from the CPA.",
                entry.cert_id,
                plural(count, "certificate", "certificates"),
                if entry.certificates.len() == 1 { "certificate is" } else { "certificates are" },
            ),
            with_cancel: false,
            subject: pending,
        });
    }

    fn apply_upload(&mut self, pending: PendingUpload, shared: &mut Shared) {
        let Some(document) = shared.document.as_mut() else { return };
        match key_info_from_pem(document, &pending.pem) {
            Ok(new_key_info) => {
                let old = document.find(pending.certificate, Some(DS_NAMESPACE), "KeyInfo");
                replace_key_info(document, pending.certificate, old, new_key_info);
                shared.changed();
                shared.log.info("Certificate uploaded successfully and transformed into KeyInfo element");
                // The row of the certId keeps its key (the Certificate element), so it stays selected
                self.tree.selected = Some(key(pending.certificate));
            }
            Err(e) => shared.log.error(format!("Error while uploading certificate: {e}")),
        }
    }

    /// "Download Certificate": saves the chain of the KeyInfo as one PEM file, leaf first.
    fn download(&mut self, index: usize, shared: &mut Shared) {
        let Some(document) = shared.document.as_ref() else { return };
        let entry = &self.entries[index];
        let pem = match key_info_to_pem(document, entry.key_info) {
            Ok(pem) => pem,
            Err(e) => {
                shared.log.error(format!("Error while downloading certificate: {e}"));
                return;
            }
        };
        if pem.is_empty() {
            shared.log.error("No certificate found in this KeyInfo");
            return;
        }
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Certificate files", &["pem", "crt", "cer"])
            .set_file_name(pem_file_name(Some(&entry.cert_id)))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("pem");
        }
        match std::fs::write(&path, pem.as_bytes()) {
            Ok(()) => shared.log.info(format!(
                "{} certificate(s) of certId {} saved to {}",
                pem.matches("BEGIN CERTIFICATE").count(),
                entry.cert_id,
                path.display()
            )),
            Err(e) => shared.log.error(format!("Error while downloading certificate: {e}")),
        }
    }
}

fn read_text(path: &PathBuf) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// One certificate of the chain: type, common name, status and the details of the certificate.
fn certificate_card(ui: &mut Ui, id: (usize, usize), position: &str, certificate: &CertificateDetails, status: Status) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6)
        .inner_margin(10)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Certificate ({})", certificate.kind)).weak());
                ui.label(RichText::new(format!("·  {position}")).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::badge(ui, &status.text(), status.color(ui));
                });
            });
            ui.add(
                egui::Label::new(RichText::new(&certificate.common_name).strong().size(15.0))
                    .wrap()
                    .selectable(true),
            )
            .on_hover_text(summary(certificate));
            ui.add_space(4.0);
            egui::Grid::new(("certificate_card", id)).num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                let valid_until = format_cpa_date(certificate.not_after);
                for (name, value) in [
                    ("Subject", certificate.subject.clone()),
                    ("Issuer", certificate.issuer.clone()),
                    ("Serial number", certificate.serial_number.clone()),
                    ("Valid from", format_cpa_date(certificate.not_before)),
                    ("Valid until", valid_until),
                ] {
                    ui.label(RichText::new(name).weak());
                    let mut text = RichText::new(value);
                    if name == "Valid until" && status != Status::Valid {
                        text = text.color(status.color(ui));
                    }
                    ui.add(egui::Label::new(text).wrap().selectable(true));
                    ui.end_row();
                }
            });
        });
}

impl Tab for CertificatesTab {
    fn title(&self) -> &'static str {
        "Certificates"
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if self.generation != shared.generation {
            *self = CertificatesTab { generation: shared.generation, ..Default::default() };
        }
        if shared.document.is_none() {
            no_cpa(ui, shared);
            return;
        }
        if self.built_for != Some(shared.revision) {
            self.build(shared);
            self.built_for = Some(shared.revision);
        }
        if self.entries.is_empty() {
            theme::empty_state(
                ui,
                "No certificates in this CPA",
                "The parties of this CPA have no Certificate elements with a certId.",
                |_| {},
            );
            return;
        }

        // The status cell gets the status colour, unused certIds weak text
        let now = self.now;
        for row in &mut self.rows {
            if let Some(&index) = self.row_entry.get(&row.key) {
                let entry = &self.entries[index];
                // Only the status cell carries the status colour; a whole red tree would hide what matters
                row.text_color = entry.usages.is_empty().then(|| ui.visuals().weak_text_color());
                row.cell_colors = vec![None, entry.status(now).map(|status| status.color(ui))];
            }
        }

        ui.add_space(2.0);
        self.summary_bar(ui);
        ui.add_space(6.0);

        let mut action = None;
        let dirty = shared.is_dirty();
        let max_width = ui.available_width() * 0.5;
        egui::Panel::right("certificates_detail")
            .resizable(true)
            .default_size(460.0)
            .min_size(360.0)
            .max_size(max_width)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(4.0);
                    self.detail_panel(ui, dirty, &mut action);
                });
            });
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let row_entry = &self.row_entry;
            let entries = &self.entries;
            let menu = &mut action;
            self.tree.show(ui, &self.rows, |ui, row| {
                let index = row.and_then(|row| row_entry.get(&row).copied());
                let enabled = |check: fn(&Entry) -> bool| index.is_some_and(|i| check(&entries[i]));
                let mut item = |ui: &mut Ui, text: &str, enabled: bool, chosen: fn(usize) -> Action| {
                    if ui.add_enabled(enabled, egui::Button::new(text)).clicked() {
                        if let Some(index) = index {
                            *menu = Some(chosen(index));
                        }
                        ui.close();
                    }
                };
                item(ui, "Upload certificate…", index.is_some(), Action::Upload);
                item(ui, "Download certificate…", enabled(|e| !e.certificates.is_empty()), Action::Download);
                item(ui, "Copy KeyInfo", enabled(|e| e.key_info.is_some()), Action::CopyKeyInfo);
            });
        });
        if let Some(action) = action {
            self.run(action, ui, shared);
        }

        if let Some(confirm) = &self.confirm {
            if let Some((answer, pending)) = confirm.show(ui.ctx()) {
                self.confirm = None;
                if answer == Answer::Yes {
                    self.apply_upload(pending, shared);
                }
            }
        }
    }
}
