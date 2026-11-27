//! The tabs of the main window. Each tab draws itself from the shared CPA document and changes it directly; a
//! change bumps `Shared::revision`, so the other tabs rebuild their view when they are shown again (the Python
//! version reloads a tab when the user switches to it).

use std::path::PathBuf;

use cpa_core::xml::Document;
use eframe::egui::Ui;

use crate::log::Log;

pub mod certificates;
pub mod collaboration_role;
pub mod comment;
pub mod general;
pub mod transport;
pub mod validation;
pub mod xml_editor;

/// State shared by all tabs.
#[derive(Default)]
pub struct Shared {
    pub document: Option<Document>,
    pub log: Log,
    /// Increased on every change of the document and on loading another CPA.
    pub revision: u64,
    /// Increased only on loading another CPA: a tab forgets its selection and edit fields.
    pub generation: u64,
    pub file_path: Option<PathBuf>,
    /// The revision that was loaded or saved last; a different revision means unsaved changes.
    pub saved_revision: u64,
    /// Set by a tab to ask the main window for the Open dialog.
    pub request_open: bool,
    /// Set by a tab to ask the main window for the Save dialog.
    pub request_save: bool,
    /// Set by the General tab: a date is not valid, so saving is not the next step.
    pub invalid_dates: usize,
    /// The schema validation of the CPA, run by the main window after every change.
    pub validation: Option<Validation>,
}

/// The outcome of the automatic validation of one revision of the CPA.
pub struct Validation {
    pub revision: u64,
    pub time: chrono::DateTime<chrono::Local>,
    /// The schema errors, or why the validation itself failed.
    pub outcome: Result<Vec<cpa_core::validation::ValidationError>, String>,
}

impl Validation {
    /// The number of errors; None when the validation failed.
    pub fn error_count(&self) -> Option<usize> {
        self.outcome.as_ref().ok().map(Vec::len)
    }
}

impl Shared {
    /// Call after every change of the document.
    pub fn changed(&mut self) {
        self.revision += 1;
    }

    pub fn is_dirty(&self) -> bool {
        self.document.is_some() && self.revision != self.saved_revision
    }
}

pub trait Tab {
    fn title(&self) -> &'static str;
    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared);
}

/// Empty state of a tab without CPA, with the action to open one.
pub fn no_cpa(ui: &mut Ui, shared: &mut Shared) {
    crate::theme::empty_state(ui, "No CPA loaded", "Open a CPA file to view and edit it here.", |ui| {
        if ui.add(crate::theme::primary_button("Open CPA…")).clicked() {
            shared.request_open = true;
        }
    });
}

/// A tab that is not ported yet.
pub struct Placeholder(pub &'static str);

impl Tab for Placeholder {
    fn title(&self) -> &'static str {
        self.0
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if shared.document.is_none() {
            no_cpa(ui, shared);
        } else {
            crate::theme::empty_state(ui, self.0, "This screen is being ported to the new version.", |_| {});
        }
    }
}
