//! Renders the Transport and XML editor tabs offscreen with realistic data, for the UI review (see DESIGN.md). Not
//! part of the normal test run because it needs a GPU adapter:
//!
//! `REVIEW_ROUND=1 cargo test -p cpa-editor --test review_transport_xml -- --ignored`
//!
//! The PNGs are written to `cpa-editor/tests/snapshots/review/<screen>-r<round>.png`.

use std::path::{Path, PathBuf};

use cpa_editor::app::CpaEditorApp;
use eframe::egui;
use egui_kittest::Harness;

const TRANSPORT_TAB: usize = 2;
const XML_EDITOR_TAB: usize = 4;

fn resource(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name)
}

fn harness(tab: usize, theme: egui::Theme) -> Harness<'static, CpaEditorApp> {
    let file = resource("full_test_cpa.xml");
    let mut harness = Harness::builder()
        .with_size([1500.0, 900.0])
        .with_pixels_per_point(1.0)
        .with_theme(theme)
        .with_render_cursor(false)
        .build_eframe(move |cc| CpaEditorApp::new(cc, Some(file), false));
    harness.state_mut().select_tab(tab);
    harness.run_steps(2);
    harness
}

/// Moves the selection of the tree down `count` rows with the keyboard.
fn arrow_down(harness: &mut Harness<'_, CpaEditorApp>, count: usize) {
    for _ in 0..count {
        harness.key_press(egui::Key::ArrowDown);
        harness.run_steps(1);
    }
}

fn save(harness: &mut Harness<'_, CpaEditorApp>, name: &str) {
    harness.input_mut().events.push(egui::Event::PointerGone);
    harness.run_steps(4);
    let round = std::env::var("REVIEW_ROUND").unwrap_or_else(|_| "0".into());
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/review");
    std::fs::create_dir_all(&directory).unwrap();
    let image = harness.render().expect("render with wgpu");
    image.save(directory.join(format!("{name}-r{round}.png"))).unwrap();
}

#[test]
#[ignore = "needs a GPU adapter; run for the UI review"]
fn review_transport_and_xml_editor() {
    for (theme, suffix) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
        // Transport: nothing selected, the Transport row (clone), and the certId of a ClientCertificateRef
        let mut h = harness(TRANSPORT_TAB, theme);
        save(&mut h, &format!("transport-{suffix}"));
        arrow_down(&mut h, 2);
        save(&mut h, &format!("transport-element-{suffix}"));
        arrow_down(&mut h, 9);
        save(&mut h, &format!("transport-selected-{suffix}"));

        // XML editor: nothing selected, then the Start element (root, 3 attributes, Status, @value, Start)
        let mut h = harness(XML_EDITOR_TAB, theme);
        save(&mut h, &format!("xml-editor-{suffix}"));
        arrow_down(&mut h, 7);
        save(&mut h, &format!("xml-editor-selected-{suffix}"));
        arrow_down(&mut h, 5);
        save(&mut h, &format!("xml-editor-element-{suffix}"));
    }
}
