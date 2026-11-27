//! Renders the screens offscreen with realistic data, for the UI review (see DESIGN.md). Not part of the normal test
//! run because it needs a GPU adapter:
//!
//! `REVIEW_ROUND=1 cargo test -p cpa-editor --test review_screens -- --ignored`
//!
//! The PNGs are written to `cpa-editor/tests/snapshots/review/<screen>-r<round>.png`.

use std::path::{Path, PathBuf};

use cpa_editor::app::CpaEditorApp;
use eframe::egui;
use egui_kittest::{kittest::Queryable, Harness};

fn resource(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name)
}

fn harness(file: Option<PathBuf>, theme: egui::Theme) -> Harness<'static, CpaEditorApp> {
    Harness::builder()
        .with_size([1500.0, 900.0])
        .with_pixels_per_point(1.0)
        .with_theme(theme)
        .build_eframe(move |cc| CpaEditorApp::new(cc, file, false))
}

fn save(harness: &mut Harness<'_, CpaEditorApp>, name: &str) {
    // The pointer of a simulated click is drawn; take it out of the picture
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
fn review_screens() {
    for (theme, suffix) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
        // Start: no CPA loaded
        let mut h = harness(None, theme);
        save(&mut h, &format!("start-{suffix}"));

        // Collaboration Role: nothing selected, then a Send action selected with the keyboard
        let mut h = harness(Some(resource("full_test_cpa.xml")), theme);
        h.state_mut().select_tab(1);
        save(&mut h, &format!("collaboration-role-{suffix}"));
        for _ in 0..3 {
            h.key_press(egui::Key::ArrowDown);
            h.run_steps(2);
        }
        save(&mut h, &format!("collaboration-role-selected-{suffix}"));

        // CPA Validation: before, with errors, and valid
        let mut h = harness(Some(resource("full_test_cpa.xml")), theme);
        h.state_mut().select_tab(6);
        save(&mut h, &format!("validation-{suffix}"));
        h.get_by_label("Validate CPA").click();
        save(&mut h, &format!("validation-errors-{suffix}"));
        let mut h = harness(Some(resource("full_cpa_p.xml")), theme);
        h.state_mut().select_tab(6);
        h.run_steps(2);
        h.get_by_label("Validate CPA").click();
        save(&mut h, &format!("validation-valid-{suffix}"));
    }
}

#[test]
#[ignore = "diagnostic"]
fn glyph_coverage() {
    let mut h = Harness::new_ui(|ui| {
        let font = egui::FontId::proportional(14.0);
        for c in "⏷⏵▶◀►◄▸▾→←＋+●•·…✓✗⚠×▼▲⬤".chars() {
            println!("{c} {:?}", ui.ctx().fonts_mut(|f| f.has_glyph(&font, c)));
        }
    });
    h.run_steps(1);
}
