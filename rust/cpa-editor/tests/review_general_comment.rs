//! Renders the General and Comment tabs offscreen with realistic data, for the UI review (see DESIGN.md). Not part
//! of the normal test run because it needs a GPU adapter:
//!
//! `REVIEW_ROUND=1 cargo test -p cpa-editor --test review_general_comment -- --ignored`
//!
//! The PNGs are written to `cpa-editor/tests/snapshots/review/<screen>-r<round>.png`.

use std::path::{Path, PathBuf};

use cpa_core::cpa::{comments, delete_comment, set_general, GeneralField};
use cpa_editor::app::CpaEditorApp;
use eframe::egui;
use egui_kittest::{kittest::Queryable, Harness};

const GENERAL: usize = 0;
const COMMENT: usize = 3;

fn resource(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../application/tests/resources/cpas").join(name)
}

fn harness(theme: egui::Theme, tab: usize) -> Harness<'static, CpaEditorApp> {
    harness_with(theme, tab, resource("full_test_cpa.xml"))
}

fn harness_with(theme: egui::Theme, tab: usize, file: PathBuf) -> Harness<'static, CpaEditorApp> {
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

/// Changes the CPA as another tab would.
fn change(harness: &mut Harness<'_, CpaEditorApp>, edit: impl FnOnce(&mut cpa_core::xml::Document)) {
    let shared = &mut harness.state_mut().shared;
    edit(shared.document.as_mut().unwrap());
    shared.changed();
    harness.run_steps(2);
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
fn review_general_comment() {
    for (theme, suffix) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
        // General: as loaded, then the end date from the certificates (they have expired in the test CPA)
        let mut h = harness(theme, GENERAL);
        save(&mut h, &format!("general-{suffix}"));
        h.get_by_label("Set from certificates").click();
        save(&mut h, &format!("general-from-certificates-{suffix}"));

        // General: partner A known under more PartyIds (an OIN and a KvK number)
        let xml = std::fs::read_to_string(resource("full_test_cpa.xml")).unwrap();
        let first = "<tns:PartyId tns:type=\"urn:example:partyid\">PARTNER_A</tns:PartyId>";
        let more = format!(
            "{first}\n        <tns:PartyId tns:type=\"urn:osb:oin\">00000001234567890000</tns:PartyId>\n        \
             <tns:PartyId tns:type=\"urn:example:kvk\">12345678</tns:PartyId>"
        );
        assert!(xml.contains(first), "the test CPA has the PartyId of partner A");
        let path = std::env::temp_dir().join(format!("cpa_editor_review_party_ids_{suffix}.xml"));
        std::fs::write(&path, xml.replacen(first, &more, 1)).unwrap();
        let mut h = harness_with(theme, GENERAL, path);
        save(&mut h, &format!("general-party-ids-{suffix}"));

        // General: a start date without timezone and an end date that is not a date
        let mut h = harness(theme, GENERAL);
        change(&mut h, |doc| {
            set_general(doc, GeneralField::StartDate, "2026-01-01T00:00:00").unwrap();
            set_general(doc, GeneralField::EndDate, "next year").unwrap();
        });
        save(&mut h, &format!("general-invalid-dates-{suffix}"));

        // Comment: nothing selected, a comment selected with the keyboard, the delete question
        let mut h = harness(theme, COMMENT);
        save(&mut h, &format!("comment-{suffix}"));
        for _ in 0..2 {
            h.key_press(egui::Key::ArrowDown);
            h.run_steps(2);
        }
        save(&mut h, &format!("comment-selected-{suffix}"));
        // Change the selected comment: Save changes becomes the primary button
        h.get_by_role(egui::accesskit::Role::MultilineTextInput).click();
        h.run_steps(2);
        h.input_mut().events.push(egui::Event::Text(" Version 2.".into()));
        h.run_steps(2);
        save(&mut h, &format!("comment-edited-{suffix}"));
        h.get_by_label("Discard").click();
        h.run_steps(2);
        h.get_by_label("Delete comment…").click();
        save(&mut h, &format!("comment-delete-{suffix}"));

        // Comment: a new comment being written
        let mut h = harness(theme, COMMENT);
        h.get_by_label("+ New comment").click();
        h.run_steps(2);
        // The text field has the focus after "New comment"
        h.input_mut().events.push(egui::Event::Text("Reviewed by the integration team".into()));
        h.run_steps(2);
        save(&mut h, &format!("comment-new-{suffix}"));

        // Comment: a CPA without comments
        let mut h = harness(theme, COMMENT);
        change(&mut h, |doc| {
            for comment in comments(doc) {
                delete_comment(doc, comment).unwrap();
            }
        });
        save(&mut h, &format!("comment-empty-{suffix}"));
    }
}
