//! The main window: the File and Help menus, the tab bar, and the bottom bar with the file actions and the log.

use std::path::{Path, PathBuf};

use cpa_core::xml::Document;
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, RichText, Stroke};

use crate::help_window::{HelpWindows, UpdateCheck};
use crate::log::Level;
use crate::tabs::certificates::CertificatesTab;
use crate::tabs::collaboration_role::CollaborationRoleTab;
use crate::tabs::comment::CommentTab;
use crate::tabs::general::GeneralTab;
use crate::tabs::transport::TransportTab;
use crate::tabs::validation::ValidationTab;
use crate::tabs::xml_editor::XmlEditorTab;
use crate::tabs::{Shared, Tab};
use crate::theme;

const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

pub struct CpaEditorApp {
    pub shared: Shared,
    tabs: Vec<Box<dyn Tab>>,
    current_tab: usize,
    help: HelpWindows,
    update_check: UpdateCheck,
    window_title: String,
    /// The CPA as indented text when it was loaded: the base of Show changes, also after saving.
    loaded_text: Option<String>,
    /// Show changes, or the question before saving.
    diff: Option<crate::diff_window::DiffWindow>,
    /// New CPA from this one.
    new_cpa: Option<crate::new_cpa::NewCpaDialog>,
}

impl CpaEditorApp {
    /// `check_for_updates` is false in tests, which must not use the network.
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>, check_for_updates: bool) -> Self {
        Self::with_context(&cc.egui_ctx, initial_file, check_for_updates)
    }

    pub fn with_context(ctx: &egui::Context, initial_file: Option<PathBuf>, check_for_updates: bool) -> Self {
        theme::apply(ctx);
        let mut app = CpaEditorApp {
            shared: Shared::default(),
            tabs: vec![
                Box::new(GeneralTab::default()),
                Box::new(CollaborationRoleTab::default()),
                Box::new(TransportTab::default()),
                Box::new(CommentTab::default()),
                Box::new(XmlEditorTab::default()),
                Box::new(CertificatesTab::default()),
                Box::new(ValidationTab),
            ],
            current_tab: 0,
            help: HelpWindows::default(),
            update_check: UpdateCheck::default(),
            window_title: String::new(),
            loaded_text: None,
            diff: None,
            new_cpa: None,
        };
        if check_for_updates {
            // Look for a newer version once the window is shown
            app.update_check.start(ctx, true, &mut app.shared.log);
        }
        if let Some(path) = initial_file {
            app.load_cpa(&path);
        }
        // CPA_EDITOR_TAB=<number> opens on that tab, used for screenshots of the documentation
        if let Some(tab) = std::env::var("CPA_EDITOR_TAB").ok().and_then(|t| t.parse::<usize>().ok()) {
            app.select_tab(tab);
        }
        app
    }

    pub fn tab_titles(&self) -> Vec<&'static str> {
        self.tabs.iter().map(|tab| tab.title()).collect()
    }

    pub fn select_tab(&mut self, index: usize) {
        self.current_tab = index.min(self.tabs.len() - 1);
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
                self.loaded_text = Some(cpa_core::diff::cpa_text(&document));
                self.diff = None;
                self.new_cpa = None;
                self.shared.document = Some(document);
                self.shared.file_path = Some(path.to_path_buf());
                self.shared.revision += 1;
                self.shared.generation += 1;
                self.shared.saved_revision = self.shared.revision;
        self.shared.invalid_dates = 0;
        self.shared.validation = None;
                self.shared.log.info("CPA data loaded successfully.");
            }
            Err(e) => self.shared.log.error(format!("Error while loading CPA data: {e}")),
        }
    }

    /// The changes since the CPA was loaded; empty without a CPA or without changes.
    fn changes(&self) -> Vec<cpa_core::diff::Hunk> {
        match (&self.loaded_text, &self.shared.document) {
            (Some(loaded), Some(document)) => cpa_core::diff::cpa_changes(loaded, &cpa_core::diff::cpa_text(document)),
            _ => Vec::new(),
        }
    }

    /// New CPA from this one: asks for the new CPAId and dates.
    fn ask_new_cpa(&mut self) {
        match &self.shared.document {
            Some(document) => self.new_cpa = Some(crate::new_cpa::NewCpaDialog::new(document)),
            None => self.shared.log.error("No CPA loaded, open the CPA to start from first"),
        }
    }

    /// Replaces the loaded CPA with a new one made from it. The loaded CPA stays the base of Show changes, so the
    /// differences with the source can be checked before saving.
    pub fn create_new_cpa(&mut self, options: &cpa_core::cpa::NewCpaOptions) {
        let Some(document) = &self.shared.document else { return };
        let source = cpa_core::cpa::read_general(document).cpa_id;
        let renames = if options.rename_ids {
            cpa_core::cpa::id_renames(document, &options.party_ids).unwrap_or_default()
        } else {
            Vec::new()
        };
        match cpa_core::cpa::new_cpa_from(document, options) {
            Ok((new, report)) => {
                self.shared.document = Some(new);
                let name = cpa_core::cpa::cpa_file_name(&options.cpa_id);
                self.shared.file_path = Some(match self.shared.file_path.as_ref().and_then(|p| p.parent()) {
                    Some(directory) => directory.join(name),
                    None => PathBuf::from(name),
                });
                self.shared.revision += 1;
                self.shared.generation += 1;
                self.shared.invalid_dates = 0;
                self.shared.validation = None;
                self.shared.log.info(format!(
                    "New CPA {} made from {source}: valid from {} to {}",
                    options.cpa_id.trim(),
                    options.start_date.trim(),
                    options.end_date.trim()
                ));
                if options.reset_status {
                    self.shared.log.info("Status set to proposed");
                }
                if !renames.is_empty() {
                    self.shared.log.info(format!("Renamed {} ids and references with the PartyIds", renames.len()));
                    for (old, new) in &renames {
                        self.shared.log.debug(format!("Renamed {old} to {new}"));
                    }
                }
                if let Some(report) = report {
                    crate::tabs::certificates::log_strip_report(&mut self.shared.log, &report);
                }
                self.shared.log.info("Not saved yet: Show changes… lists the differences with the source CPA");
            }
            Err(e) => self.shared.log.error(format!("Error while making the new CPA: {e}")),
        }
    }

    /// Show changes: the window with the changes of this moment.
    fn show_changes(&mut self) {
        if self.shared.document.is_none() {
            self.shared.log.error("No CPA loaded, no changes to show");
            return;
        }
        let hunks = self.changes();
        if hunks.is_empty() {
            self.shared.log.info("No changes since the CPA was loaded");
            return;
        }
        self.diff = Some(crate::diff_window::DiffWindow::new(hunks, false));
    }

    /// Save CPA…: first the changes since the CPA was loaded, then the file dialog when the user chooses Save….
    fn save_cpa_dialog(&mut self) {
        if self.shared.document.is_none() {
            self.shared.log.error("No CPA loaded, nothing to save");
            return;
        }
        let hunks = self.changes();
        if hunks.is_empty() {
            self.shared.log.info("No changes since the CPA was loaded");
            self.save_cpa_file_dialog();
            return;
        }
        self.diff = Some(crate::diff_window::DiffWindow::new(hunks, true));
    }

    fn save_cpa_file_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("xml files", &["xml"]);
        if let Some(path) = &self.shared.file_path {
            if let Some(directory) = path.parent() {
                dialog = dialog.set_directory(directory);
            }
            if let Some(name) = path.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy());
            }
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
            Ok(()) => {
                self.shared.log.info(format!("CPA saved to {}", path.display()));
                self.shared.file_path = Some(path.to_path_buf());
                self.shared.saved_revision = self.shared.revision;
            }
            Err(e) => self.shared.log.error(format!("Error while saving CPA: {e}")),
        }
    }

    fn file_name(&self) -> Option<String> {
        self.shared.file_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned())
    }

    fn update_window_title(&mut self, ctx: &egui::Context) {
        let mut title = format!("CPA Editor v{}", cpa_core::VERSION);
        if let Some(name) = self.file_name() {
            title = format!("{name}{} – {title}", if self.shared.is_dirty() { " ●" } else { "" });
        }
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }

    /// The outcome of the automatic validation; errors link to the CPA Validation tab.
    fn validation_status(&mut self, ui: &mut egui::Ui) {
        let Some(validation) = &self.shared.validation else { return };
        let tab = self.tabs.iter().position(|t| t.title() == "CPA Validation");
        match validation.error_count() {
            Some(0) => {
                ui.label(RichText::new("• Valid").color(theme::ok_color(ui)))
                    .on_hover_text("Valid against the ebXML CPP/CPA 2.0 schema; checked after every change");
            }
            count => {
                let text = match count {
                    Some(1) => "• 1 schema error".to_string(),
                    Some(n) => format!("• {n} schema errors"),
                    None => "• Validation failed".to_string(),
                };
                let link = ui.add(egui::Link::new(RichText::new(text).color(theme::error_color(ui))));
                if link.on_hover_text("Show on the CPA Validation tab").clicked() {
                    if let Some(tab) = tab {
                        self.current_tab = tab;
                    }
                }
            }
        }
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status").resizable(false).show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Open CPA…").on_hover_text(ui.ctx().format_shortcut(&OPEN_SHORTCUT)).clicked() {
                    self.load_cpa_dialog();
                }
                // Primary only with unsaved changes: then saving is the next step
                let can_save = self.shared.document.is_some();
                // Not the next step while the General tab has an invalid date
                let invalid = self.shared.invalid_dates;
                let primary = self.shared.is_dirty() && invalid == 0;
                let save = theme::action_button(ui, "Save CPA…", primary, can_save)
                    .on_hover_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT));
                if save.clicked() {
                    self.save_cpa_dialog();
                }
                let show = ui.add_enabled(can_save, egui::Button::new("Show changes…").min_size(egui::vec2(96.0, 28.0)));
                if show.on_hover_text("The changes since the CPA was loaded").clicked() {
                    self.show_changes();
                }
                ui.add_space(8.0);
                if let Some(name) = self.file_name() {
                ui.label(RichText::new(name).strong());
                if self.shared.is_dirty() {
                    ui.label(RichText::new("• Unsaved changes").color(theme::warning_color(ui)));
                }
                self.validation_status(ui);
                if invalid > 0 {
                    let text = if invalid == 1 { "1 invalid date".to_string() } else { format!("{invalid} invalid dates") };
                    theme::badge(ui, &text, theme::error_color(ui));
                    ui.label(RichText::new("on the General tab").weak());
                }
                }
                self.last_message(ui);
            });
            ui.add_space(4.0);
        });
    }

    /// The last message on the right of the bottom bar: an info message for a few seconds, a warning or error until
    /// the next message. The whole text is in the tooltip.
    fn last_message(&self, ui: &mut egui::Ui) {
        const INFO_SECONDS: u64 = 8;
        let Some((level, text, age)) = self.shared.log.last() else { return };
        if level == Level::Info && age.as_secs() >= INFO_SECONDS {
            return;
        }
        if level == Level::Info {
            ui.ctx().request_repaint_after(std::time::Duration::from_secs(INFO_SECONDS).saturating_sub(age));
        }
        let color = match level {
            Level::Error => theme::error_color(ui),
            Level::Warning => theme::warning_color(ui),
            _ => ui.visuals().weak_text_color(),
        };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(RichText::new(text).color(color)).truncate()).on_hover_text(text);
        });
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let open = egui::Button::new("Open CPA…").shortcut_text(ui.ctx().format_shortcut(&OPEN_SHORTCUT));
                    if ui.add(open).clicked() {
                        ui.close();
                        self.load_cpa_dialog();
                    }
                    let save = egui::Button::new("Save CPA…").shortcut_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT));
                    if ui.add_enabled(self.shared.document.is_some(), save).clicked() {
                        ui.close();
                        self.save_cpa_dialog();
                    }
                    if ui.add_enabled(self.shared.document.is_some(), egui::Button::new("New CPA from this one…")).clicked() {
                        ui.close();
                        self.ask_new_cpa();
                    }
                    if ui.add_enabled(self.shared.document.is_some(), egui::Button::new("Show changes…")).clicked() {
                        ui.close();
                        self.show_changes();
                    }
                });
                ui.menu_button("Help", |ui| {
                    for label in self.help.labels() {
                        if ui.button(label).clicked() {
                            self.help.open(label);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Check for updates").clicked() {
                        self.update_check.start(ui.ctx(), false, &mut self.shared.log);
                        ui.close();
                    }
                });
            });
        });
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 18.0;
            for index in 0..self.tabs.len() {
                let title = self.tabs[index].title();
                let active = self.current_tab == index;
                let text = if active {
                    RichText::new(title).strong().color(ui.visuals().strong_text_color())
                } else {
                    RichText::new(title).color(ui.visuals().widgets.noninteractive.fg_stroke.color.gamma_multiply(0.8))
                };
                let response = ui.add(egui::Button::new(text).frame(false).min_size(egui::vec2(0.0, 32.0)));
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if active {
                    let rect = response.rect;
                    ui.painter().hline(rect.x_range(), rect.bottom() - 1.0, Stroke::new(3.0, theme::ACCENT));
                }
                if response.clicked() && !active {
                    self.current_tab = index;
                    self.shared.log.debug(format!("{title} tab selected"));
                }
            }
        });
        ui.separator();
    }
}

impl eframe::App for CpaEditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl CpaEditorApp {
    /// The whole window; also used by the screenshot tests.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.validate_if_changed();
        // Open a CPA that is dropped on the window
        let dropped = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path().to_path_buf()));
        if let Some(path) = dropped.filter(|p| !p.as_os_str().is_empty()) {
            self.load_cpa(&path);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&OPEN_SHORTCUT)) {
            self.load_cpa_dialog();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&SAVE_SHORTCUT)) {
            self.save_cpa_dialog();
        }
        self.update_window_title(&ctx);

        self.menu_bar(ui);
        self.bottom_panel(ui);
        egui::CentralPanel::default().show(ui, |ui| {
            self.tab_bar(ui);
            let tab = &mut self.tabs[self.current_tab];
            tab.ui(ui, &mut self.shared);
        });
        if std::mem::take(&mut self.shared.request_open) {
            self.load_cpa_dialog();
        }
        if std::mem::take(&mut self.shared.request_save) {
            self.save_cpa_dialog();
        }
        if let (Some(dialog), Some(document)) = (&mut self.new_cpa, &self.shared.document) {
            if let Some(answer) = dialog.show(&ctx, document) {
                self.new_cpa = None;
                if let crate::new_cpa::NewCpaAnswer::Create(options) = answer {
                    self.create_new_cpa(&options);
                }
            }
        }
        if let Some(diff) = &self.diff {
            if let Some(answer) = diff.show(&ctx) {
                let confirm = diff.is_confirm();
                self.diff = None;
                match answer {
                    crate::diff_window::DiffAnswer::Save => self.save_cpa_file_dialog(),
                    crate::diff_window::DiffAnswer::Close if confirm => self.shared.log.info("Saving cancelled"),
                    crate::diff_window::DiffAnswer::Close => {}
                }
            }
        }
        self.help.show(&ctx);
        self.update_check.show(&ctx, &mut self.shared.log);
        // A change in this frame is validated in the next one, so the status shows it at once
        if self.shared.document.is_some() && self.shared.validation.as_ref().map(|v| v.revision) != Some(self.shared.revision) {
            ctx.request_repaint();
        }
    }

    /// Validates the CPA against the schema when it changed since the last validation (a few milliseconds). The log
    /// only reports a change of the outcome, not every validation.
    fn validate_if_changed(&mut self) {
        let Some(document) = &self.shared.document else {
            self.shared.validation = None;
            return;
        };
        if self.shared.validation.as_ref().is_some_and(|v| v.revision == self.shared.revision) {
            return;
        }
        let outcome = cpa_core::validation::validate_document(document);
        let validation = crate::tabs::Validation { revision: self.shared.revision, time: chrono::Local::now(), outcome };
        // The bottom bar shows the outcome; only a validation that could not run is a message
        if let Err(e) = &validation.outcome {
            self.shared.log.error(format!("Validation failed: {e}"));
        }
        self.shared.validation = Some(validation);
    }
}
