//! The main window: the tabs, the log output and the Load and Save buttons below them, and the Help menu.

use std::path::{Path, PathBuf};

use cpa_core::xml::Document;
use eframe::egui::{self, RichText};

use crate::log::Level;
use crate::tabs::collaboration_role::CollaborationRoleTab;
use crate::tabs::validation::ValidationTab;
use crate::tabs::{Placeholder, Shared, Tab};

pub struct CpaEditorApp {
    shared: Shared,
    tabs: Vec<Box<dyn Tab>>,
    current_tab: usize,
}

impl CpaEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        let mut app = CpaEditorApp {
            shared: Shared::default(),
            tabs: vec![
                Box::new(Placeholder("General")),
                Box::new(CollaborationRoleTab::default()),
                Box::new(Placeholder("Transport")),
                Box::new(Placeholder("Comment")),
                Box::new(Placeholder("XML editor")),
                Box::new(Placeholder("Certificates")),
                Box::new(ValidationTab::default()),
            ],
            current_tab: 0,
        };
        if let Some(path) = initial_file {
            app.load_cpa(&path);
        }
        // CPA_EDITOR_TAB=<number> opens on that tab, used for screenshots of the documentation
        if let Some(tab) = std::env::var("CPA_EDITOR_TAB").ok().and_then(|t| t.parse::<usize>().ok()) {
            app.current_tab = tab.min(app.tabs.len() - 1);
        }
        app
    }

    fn load_cpa_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().add_filter("xml files", &["xml"]).pick_file() {
            self.load_cpa(&path);
        }
    }

    pub fn load_cpa(&mut self, path: &Path) {
        let result = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| Document::parse_bytes(&bytes).map_err(|e| e.to_string()));
        match result {
            Ok(document) => {
                self.shared.log.clear();
                self.shared.document = Some(document);
                self.shared.file_path = Some(path.to_path_buf());
                self.shared.revision += 1;
                self.shared.generation += 1;
                self.shared.log.info("CPA data loaded successfully.");
            }
            Err(e) => self.shared.log.error(format!("Error while loading CPA data: {e}")),
        }
    }

    fn save_cpa_dialog(&mut self) {
        if self.shared.document.is_none() {
            self.shared.log.error("No CPA loaded, nothing to save");
            return;
        }
        let mut dialog = rfd::FileDialog::new().add_filter("xml files", &["xml"]);
        if let Some(name) = self.shared.file_path.as_ref().and_then(|p| p.file_name()) {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        if let Some(mut path) = dialog.save_file() {
            if path.extension().is_none() {
                path.set_extension("xml");
            }
            self.save_cpa(&path);
        }
    }

    /// Writes the CPA as UTF-8 with an XML declaration. The file is only opened once the text is ready, so a failed
    /// save never empties an existing file.
    pub fn save_cpa(&mut self, path: &Path) {
        let Some(document) = &self.shared.document else { return };
        let data = document.to_xml();
        match std::fs::write(path, data.as_bytes()) {
            Ok(()) => self.shared.log.info(format!("CPA saved to {}", path.display())),
            Err(e) => self.shared.log.error(format!("Error while saving CPA: {e}")),
        }
    }

    fn bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("log").resizable(true).default_height(150.0).show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Save CPA").clicked() {
                    self.save_cpa_dialog();
                }
                if ui.button("Load CPA").clicked() {
                    self.load_cpa_dialog();
                }
                ui.checkbox(&mut self.shared.log.show_debug, "Show debug messages");
            });
            ui.separator();
            let scroll_to_end = std::mem::take(&mut self.shared.log.scroll_to_end);
            egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                for (level, line) in self.shared.log.lines() {
                    let text = RichText::new(line).monospace();
                    let text = match level {
                        Level::Error => text.color(ui.visuals().error_fg_color),
                        Level::Warning => text.color(ui.visuals().warn_fg_color),
                        Level::Debug => text.weak(),
                        Level::Info => text,
                    };
                    ui.add(egui::Label::new(text).wrap());
                }
                if scroll_to_end {
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });
        });
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Help", |ui| {
                    ui.label("Help is being ported.");
                });
            });
        });
    }
}

impl eframe::App for CpaEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Open a CPA that is dropped on the window
        let dropped = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone()));
        if let Some(path) = dropped {
            self.load_cpa(&path);
        }

        self.menu_bar(ctx);
        self.bottom_panel(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(self.current_tab == index, tab.title()).clicked()
                        && self.current_tab != index
                    {
                        self.current_tab = index;
                        self.shared.log.debug(format!("{} tab selected", tab.title()));
                    }
                }
            });
            ui.separator();
            let tab = &mut self.tabs[self.current_tab];
            tab.ui(ui, &mut self.shared);
        });
    }
}
