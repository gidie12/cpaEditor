// No console window next to the application on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CPA Editor: desktop application for viewing and editing ebXML CPA files (cpp-cpa-2_0.xsd).

mod app;
mod log;
mod tabs;
mod tree_table;

use eframe::egui;

fn main() -> eframe::Result {
    // A CPA file given on the command line is opened at start
    let initial_file = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("CPA Editor v{}", cpa_core::VERSION))
            .with_inner_size([1500.0, 900.0])
            .with_min_inner_size([800.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "CPA Editor",
        options,
        Box::new(|cc| Ok(Box::new(app::CpaEditorApp::new(cc, initial_file)))),
    )
}
