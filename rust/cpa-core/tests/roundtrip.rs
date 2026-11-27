//! A CPA loaded and saved without changes must be byte for byte what the Python version (lxml) writes.
//! The reference files are made by tools/make_reference.py. tests/fixtures holds files with CRLF line ends, a
//! byte order mark, ISO-8859-1, tabs and line ends in attribute values, CDATA and nodes outside the root.

use std::fs;
use std::path::Path;

use cpa_core::xml::Document;

#[test]
fn saving_unchanged_cpa_matches_lxml() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = [manifest.join("../../application/tests/resources/cpas"), manifest.join("tests/fixtures")];
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reference");
    let mut checked = 0;
    for path in sources.iter().flat_map(|source| fs::read_dir(source).unwrap()).map(|entry| entry.unwrap().path()) {
        if path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue;
        }
        let name = path.file_name().unwrap();
        let input = fs::read(&path).unwrap();
        let expected = fs::read_to_string(reference.join(name)).unwrap();
        let document = Document::parse_bytes(&input).unwrap_or_else(|e| panic!("{name:?}: {e}"));
        let written = document.to_xml();
        if written != expected {
            let position = written.chars().zip(expected.chars()).take_while(|(a, b)| a == b).count();
            let start = position.saturating_sub(80);
            panic!(
                "{name:?} differs at char {position}\n rust: {:?}\n lxml: {:?}",
                written.chars().skip(start).take(160).collect::<String>(),
                expected.chars().skip(start).take(160).collect::<String>()
            );
        }
        checked += 1;
    }
    assert!(checked >= 15, "only {checked} CPAs checked");
}
