//! Renders the Certificates tab offscreen with realistic data, for the UI review (see DESIGN.md). Not part of the
//! normal test run because it needs a GPU adapter:
//!
//! `REVIEW_ROUND=1 cargo test -p cpa-editor --test review_certificates -- --ignored`
//!
//! The PNGs are written to `cpa-editor/tests/snapshots/review/certificates-*-r<round>.png`. The status of the
//! certificates depends on the date: on 2026-11-27 every leaf certificate of full_test_cpa.xml is expired.

use egui_kittest::kittest::Queryable;
use std::path::{Path, PathBuf};

use cpa_editor::app::CpaEditorApp;
use eframe::egui;
use egui_kittest::Harness;

/// The index of the Certificates tab.
const CERTIFICATES_TAB: usize = 5;

fn resource(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name)
}

fn harness(file: PathBuf, theme: egui::Theme) -> Harness<'static, CpaEditorApp> {
    Harness::builder()
        .with_size([1500.0, 900.0])
        .with_pixels_per_point(1.0)
        .with_theme(theme)
        .with_render_cursor(false)
        .build_eframe(move |cc| CpaEditorApp::new(cc, Some(file), false))
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
fn review_certificates() {
    for (theme, suffix) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
        // Nothing selected
        let mut h = harness(resource("full_test_cpa.xml"), theme);
        h.state_mut().select_tab(CERTIFICATES_TAB);
        h.run_steps(2);
        save(&mut h, &format!("certificates-{suffix}"));

        // The first certId selected with the keyboard: the first press selects the partner row
        for _ in 0..2 {
            h.key_press(egui::Key::ArrowDown);
            h.run_steps(2);
        }
        save(&mut h, &format!("certificates-selected-{suffix}"));

        // Remove all certificates: the question, then the result on the Transport tab and in the log
        h.get_by_label("Remove all certificates…").click();
        h.run_steps(2);
        save(&mut h, &format!("certificates-strip-confirm-{suffix}"));
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Remove all certificates").click();
        h.run_steps(2);
        save(&mut h, &format!("certificates-stripped-{suffix}"));
        h.state_mut().select_tab(2);
        h.run_steps(2);
        save(&mut h, &format!("certificates-stripped-transport-{suffix}"));

        // Show changes, then Save CPA… asks with the same changes
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Show changes…").click();
        h.run_steps(3);
        save(&mut h, &format!("show-changes-{suffix}"));
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Close window").click();
        h.run_steps(2);
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Save CPA…").click();
        h.run_steps(3);
        save(&mut h, &format!("save-confirm-changes-{suffix}"));
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Cancel").click();
        h.run_steps(2);

        // A CPA without certificates
        let mut h = harness(resource("full_cpa_p.xml"), theme);
        h.state_mut().select_tab(CERTIFICATES_TAB);
        h.run_steps(2);
        save(&mut h, &format!("certificates-empty-{suffix}"));

        // Add certificate from the empty state: partner and certId, then the PEM file
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Add certificate…").click();
        h.run_steps(2);
        save(&mut h, &format!("certificates-add-{suffix}"));
    }
}
