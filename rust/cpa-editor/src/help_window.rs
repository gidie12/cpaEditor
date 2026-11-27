//! Read-only windows with the help files (help_window.py) and the update check of the Help menu.

use std::sync::mpsc::{channel, Receiver};

use cpa_core::help::{parse_markdown, HelpLine, LineKind, SpanStyle, HELP_FILES};
use cpa_core::update_check::{check_for_update, UpdateResult, DEFAULT_TIMEOUT, LATEST_RELEASE_PAGE_URL};
use eframe::egui::{self, RichText, TextStyle};

use crate::log::Log;

/// The help windows that are open, one per help file.
pub struct HelpWindows {
    pages: Vec<(&'static str, Vec<HelpLine>, bool)>,
}

impl Default for HelpWindows {
    fn default() -> Self {
        HelpWindows { pages: HELP_FILES.iter().map(|(label, text)| (*label, parse_markdown(text), false)).collect() }
    }
}

impl HelpWindows {
    pub fn labels(&self) -> Vec<&'static str> {
        self.pages.iter().map(|(label, _, _)| *label).collect()
    }

    pub fn open(&mut self, label: &str) {
        if let Some(page) = self.pages.iter_mut().find(|(l, _, _)| *l == label) {
            page.2 = true;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        for (label, lines, open) in &mut self.pages {
            if !*open {
                continue;
            }
            egui::Window::new(*label).open(open).default_size([900.0, 650.0]).vscroll(true).show(ctx, |ui| {
                show_lines(ui, lines);
            });
        }
    }
}

fn show_lines(ui: &mut egui::Ui, lines: &[HelpLine]) {
    let body = TextStyle::Body.resolve(ui.style()).size;
    for line in lines {
        match line.kind {
            LineKind::Title => {
                ui.add_space(4.0);
                ui.label(RichText::new(line.text()).size(body + 8.0).strong());
                ui.add_space(8.0);
            }
            LineKind::Heading => {
                ui.add_space(10.0);
                ui.label(RichText::new(line.text()).size(body + 3.0).strong());
                ui.add_space(4.0);
            }
            LineKind::Code => {
                ui.label(RichText::new(line.text()).monospace());
            }
            LineKind::Normal => {
                if line.spans.is_empty() {
                    ui.add_space(body * 0.6);
                    continue;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for span in &line.spans {
                        let text = RichText::new(&span.text);
                        ui.label(match span.style {
                            SpanStyle::Normal => text,
                            SpanStyle::Bold => text.strong(),
                            SpanStyle::Code => text.code(),
                        });
                    }
                });
            }
        }
    }
}

/// The update check runs in a thread, so a slow network does not freeze the window.
#[derive(Default)]
pub struct UpdateCheck {
    running: Option<(bool, Receiver<UpdateResult>)>,
    available: Option<UpdateResult>,
}

impl UpdateCheck {
    /// Starts a check. The automatic check at start only shows something when a newer version exists.
    pub fn start(&mut self, ctx: &egui::Context, automatic: bool, log: &mut Log) {
        if self.running.is_some() {
            return;
        }
        if !automatic {
            log.info("Checking for updates...");
        }
        let (sender, receiver) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(check_for_update(cpa_core::VERSION, DEFAULT_TIMEOUT));
            ctx.request_repaint();
        });
        self.running = Some((automatic, receiver));
    }

    pub fn show(&mut self, ctx: &egui::Context, log: &mut Log) {
        if let Some((automatic, receiver)) = &self.running {
            if let Ok(result) = receiver.try_recv() {
                let automatic = *automatic;
                self.running = None;
                if let Some(error) = &result.error {
                    // At start a failed check (no network) is not worth interrupting for
                    let message = format!("Could not check for updates: {error}");
                    if automatic {
                        log.debug(message);
                    } else {
                        log.error(message);
                    }
                } else if result.update_available {
                    let latest = result.latest.clone().unwrap_or_default();
                    log.info(format!("Version v{latest} is available, this is version v{}", result.current));
                    self.available = Some(result);
                } else if automatic {
                    log.debug(format!("You are using the latest version (v{})", result.current));
                } else {
                    log.info(format!("You are using the latest version (v{})", result.current));
                }
            }
        }
        let Some(result) = &self.available else { return };
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("update_available")).show(ctx, |ui| {
            ui.heading("Update available");
            ui.add_space(6.0);
            ui.label(format!(
                "Version v{} of the CPA Editor is available.\nYou are using version v{}.",
                result.latest.as_deref().unwrap_or(""),
                result.current
            ));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Download").clicked() {
                    // The browser does the download: the application itself never downloads or runs a file.
                    // The release page, because the direct download link is the executable of the Python version.
                    ctx.open_url(egui::OpenUrl::new_tab(LATEST_RELEASE_PAGE_URL));
                    close = true;
                }
                if ui.button("Skip").clicked() {
                    close = true;
                }
            });
        });
        if close || modal.should_close() {
            self.available = None;
        }
    }
}
