//! The tabs of the main window. Each tab draws itself from the shared CPA document and changes it directly; a
//! change bumps `Shared::revision`, so the other tabs rebuild their view when they are shown again (the Python
//! version reloads a tab when the user switches to it).

use std::path::PathBuf;

use cpa_core::xml::Document;
use eframe::egui::Ui;

use crate::log::Log;

pub mod validation;

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
}

impl Shared {
    /// Call after every change of the document.
    pub fn changed(&mut self) {
        self.revision += 1;
    }
}

pub trait Tab {
    fn title(&self) -> &'static str;
    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared);
}

/// Text for a tab without CPA.
pub fn no_cpa(ui: &mut Ui) {
    ui.centered_and_justified(|ui| ui.label("Use Load CPA to open a CPA file."));
}

/// A tab that is not ported yet.
pub struct Placeholder(pub &'static str);

impl Tab for Placeholder {
    fn title(&self) -> &'static str {
        self.0
    }

    fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) {
        if shared.document.is_none() {
            no_cpa(ui);
        } else {
            ui.label("This tab is being ported.");
        }
    }
}
