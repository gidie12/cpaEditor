//! CPA Editor: desktop application for viewing and editing ebXML CPA files (cpp-cpa-2_0.xsd). The binary in
//! main.rs starts [`app::CpaEditorApp`]; the library exists so the screenshot tests can drive the same window.

pub mod app;
pub mod diff_window;
pub mod new_cpa;
pub mod help_window;
pub mod log;
pub mod tabs;
pub mod theme;
pub mod tree_table;
pub mod widgets;
