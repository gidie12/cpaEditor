//! The main window: the File and Help menus, the tab bar, and the bottom bar with the file actions and the log.

use std::path::{Path, PathBuf};

use cpa_core::xml::Document;
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, RichText, Stroke};

use crate::help_window::{HelpWindows, UpdateCheck};
use crate::log::Level;
use crate::tabs::collaboration_role::CollaborationRoleTab;
use crate::tabs::validation::ValidationTab;
use crate::tabs::{Placeholder, Shared, Tab};
use crate::theme;

const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

pub struct CpaEditorApp {
    pub shared: Shared,
    tabs: Vec<Box<dyn Tab>>,
    current_tab: usize,
    help: HelpWindows,
    update_check: UpdateCheck,
    log_open: bool,
    window_title: String,
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
                Box::new(Placeholder("General")),
                Box::new(CollaborationRoleTab::default()),
                Box::new(Placeholder("Transport")),
                Box::new(Placeholder("Comment")),
                Box::new(Placeholder("XML editor")),
                Box::new(Placeholder("Certificates")),
                Box::new(ValidationTab::default()),
            ],
            current_tab: 0,
            help: HelpWindows::default(),
            update_check: UpdateCheck::default(),
            log_open: true,
            window_title: String::new(),
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
                self.shared.document = Some(document);
                self.shared.file_path = Some(path.to_path_buf());
                self.shared.revision += 1;
                self.shared.generation += 1;
                self.shared.saved_revision = self.shared.revision;
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

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        // A collapsed log has its own panel id, so the height of the open log is kept
        let panel = if self.log_open {
            egui::Panel::bottom("log").resizable(true).default_size(150.0).min_size(90.0)
        } else {
            egui::Panel::bottom("log_closed").resizable(false)
        };
        panel.show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Load CPA…").on_hover_text(ui.ctx().format_shortcut(&OPEN_SHORTCUT)).clicked() {
                    self.load_cpa_dialog();
                }
                let can_save = self.shared.document.is_some();
                let save = ui
                    .add_enabled(can_save, theme::primary_button("Save CPA…"))
                    .on_hover_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT));
                if save.clicked() {
                    self.save_cpa_dialog();
                }
                ui.add_space(8.0);
                match self.file_name() {
                    Some(name) => {
                        ui.label(RichText::new(name).strong());
                        if self.shared.is_dirty() {
                            ui.label(RichText::new("• Unsaved changes").color(theme::warning_color(ui)));
                        }
                    }
                    None => {
                        ui.label(RichText::new("No CPA loaded").weak());
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.shared.log.show_debug, "Show debug messages");
                });
            });
            ui.add_space(4.0);
            let count = self.shared.log.lines().len();
            let header = ui
                .horizontal(|ui| {
                    let (icon, _) = ui.allocate_exact_size(egui::vec2(14.0, 18.0), egui::Sense::hover());
                    let arrow = if self.log_open { theme::Arrow::Down } else { theme::Arrow::Right };
                    theme::paint_arrow(ui.painter(), icon.center(), 8.0, arrow, ui.visuals().text_color());
                    ui.add(egui::Button::new(RichText::new(format!("Log ({count})")).strong()).frame(false))
                })
                .inner;
            if header.clicked() {
                self.log_open = !self.log_open;
            }
            if !self.log_open {
                return;
            }
            let scroll_to_end = std::mem::take(&mut self.shared.log.scroll_to_end);
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(4)
                .inner_margin(6)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                        for (level, line) in self.shared.log.lines() {
                            let text = RichText::new(line).monospace();
                            let text = match level {
                                Level::Error => text.color(theme::error_color(ui)),
                                Level::Warning => text.color(theme::warning_color(ui)),
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
        self.help.show(&ctx);
        self.update_check.show(&ctx, &mut self.shared.log);
    }
}
