//! The changes of a CPA since it was loaded (diff_window.py): the CPA as indented text, the changed lines in hunks
//! with unchanged lines around them, and per changed line the words that differ.

use similar::{capture_diff_slices, Algorithm, ChangeTag, DiffOp, TextDiff};

use crate::xml::{Document, NodeId, NodeKind};

/// Unchanged lines shown before and after a change (difflib's default).
const CONTEXT_LINES: usize = 3;

/// The CPA as indented XML text (lxml `indent`, two spaces), so a change shows as a change of lines and not of
/// layout.
pub fn cpa_text(document: &Document) -> String {
    let mut indented = document.clone();
    if let Some(root) = indented.root() {
        indent(&mut indented, root, 0);
    }
    indented.to_xml()
}

/// Gives every element with child elements (or comments) the whitespace of lxml's `indent`: a new line and two
/// spaces per level before each child and before the closing tag. Text with content is left alone.
fn indent(document: &mut Document, element: NodeId, level: usize) {
    indent_with(document, element, level, "  ");
}

/// [`indent`] with another unit per level, for example the four spaces of a CPA that is indented that way.
pub(crate) fn indent_with(document: &mut Document, element: NodeId, level: usize, unit: &str) {
    let children: Vec<NodeId> = document
        .children(element)
        .iter()
        .copied()
        .filter(|&child| matches!(document.kind(child), NodeKind::Element(_) | NodeKind::Comment(_)))
        .collect();
    if children.is_empty() {
        return;
    }
    let blank = |text: Option<String>| text.is_none_or(|t| t.trim().is_empty());
    let inner = format!("\n{}", unit.repeat(level + 1));
    let outer = format!("\n{}", unit.repeat(level));
    if blank(document.text(element)) {
        document.set_text(element, Some(&inner));
    }
    for (position, &child) in children.iter().enumerate() {
        if document.is_element(child) {
            indent_with(document, child, level + 1, unit);
        }
        if blank(document.tail(child)) {
            let after = if position + 1 == children.len() { &outer } else { &inner };
            document.set_tail(child, Some(after));
        }
    }
}

/// One line of a hunk. `parts` are the byte ranges of the words that differ from the line it replaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffLine {
    Context(String),
    Removed { text: String, parts: Vec<(usize, usize)> },
    Added { text: String, parts: Vec<(usize, usize)> },
}

/// Changed lines with the unchanged lines around them. `line` is the first line number in the current CPA.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub line: usize,
    pub lines: Vec<DiffLine>,
}

/// The changes between the loaded and the current CPA text; empty when they are equal.
pub fn cpa_changes(old_text: &str, new_text: &str) -> Vec<Hunk> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let diff = TextDiff::configure().algorithm(Algorithm::Myers).diff_slices(&old_lines, &new_lines);
    let mut hunks = Vec::new();
    for group in diff.grouped_ops(CONTEXT_LINES) {
        let Some(first) = group.first() else { continue };
        let mut hunk = Hunk { line: first.new_range().start + 1, lines: Vec::new() };
        let (mut removed, mut added): (Vec<&str>, Vec<&str>) = (Vec::new(), Vec::new());
        for op in &group {
            for change in diff.iter_changes(op) {
                match change.tag() {
                    ChangeTag::Delete => removed.push(change.value()),
                    ChangeTag::Insert => added.push(change.value()),
                    ChangeTag::Equal => {
                        push_change(&mut hunk.lines, &mut removed, &mut added);
                        hunk.lines.push(DiffLine::Context(change.value().to_string()));
                    }
                }
            }
        }
        push_change(&mut hunk.lines, &mut removed, &mut added);
        hunks.push(hunk);
    }
    hunks
}

/// The number of changed lines: a removed and an added line at the same place count as one.
pub fn changed_line_count(hunks: &[Hunk]) -> usize {
    let mut count = 0;
    let end = DiffLine::Context(String::new());
    for hunk in hunks {
        let (mut removed, mut added) = (0, 0);
        for line in hunk.lines.iter().chain([&end]) {
            match line {
                DiffLine::Removed { .. } => removed += 1,
                DiffLine::Added { .. } => added += 1,
                DiffLine::Context(_) => {
                    count += removed.max(added);
                    (removed, added) = (0, 0);
                }
            }
        }
    }
    count
}

/// Adds a block of removed and added lines. A removed and an added line at the same place are one changed line:
/// the words that differ are marked.
fn push_change(lines: &mut Vec<DiffLine>, removed: &mut Vec<&str>, added: &mut Vec<&str>) {
    for (index, text) in removed.iter().enumerate() {
        let parts = added.get(index).map(|new| changed_parts(text, new).0).unwrap_or_default();
        lines.push(DiffLine::Removed { text: text.to_string(), parts });
    }
    for (index, text) in added.iter().enumerate() {
        let parts = removed.get(index).map(|old| changed_parts(old, text).1).unwrap_or_default();
        lines.push(DiffLine::Added { text: text.to_string(), parts });
    }
    removed.clear();
    added.clear();
}

/// Words (letters, digits, underscore), and every other character on its own: a changed value is marked as a
/// whole, not letter by letter. Returns the byte ranges.
fn words(line: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    for (index, character) in line.char_indices() {
        if character.is_alphanumeric() || character == '_' {
            start.get_or_insert(index);
        } else {
            if let Some(word_start) = start.take() {
                spans.push((word_start, index));
            }
            spans.push((index, index + character.len_utf8()));
        }
    }
    if let Some(word_start) = start {
        spans.push((word_start, line.len()));
    }
    spans
}

/// Byte ranges (start, end) in a line.
pub type Ranges = Vec<(usize, usize)>;

/// The byte ranges of the words that differ in a removed and an added line (diff_window.changed_parts).
pub fn changed_parts(old_line: &str, new_line: &str) -> (Ranges, Ranges) {
    let old_words = words(old_line);
    let new_words = words(new_line);
    let old_tokens: Vec<&str> = old_words.iter().map(|&(s, e)| &old_line[s..e]).collect();
    let new_tokens: Vec<&str> = new_words.iter().map(|&(s, e)| &new_line[s..e]).collect();
    let (mut old_parts, mut new_parts) = (Vec::new(), Vec::new());
    for op in capture_diff_slices(Algorithm::Myers, &old_tokens, &new_tokens) {
        if matches!(op, DiffOp::Equal { .. }) {
            continue;
        }
        let (old_range, new_range) = (op.old_range(), op.new_range());
        if !old_range.is_empty() {
            old_parts.push((old_words[old_range.start].0, old_words[old_range.end - 1].1));
        }
        if !new_range.is_empty() {
            new_parts.push((new_words[new_range.start].0, new_words[new_range.end - 1].1));
        }
    }
    (old_parts, new_parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CPA: &str = "<a xmlns=\"urn:x\"><b x=\"1\">text</b><c/></a>";

    #[test]
    fn cpa_text_ignores_layout() {
        let compact = Document::parse(CPA).unwrap();
        let spread = Document::parse("<a xmlns=\"urn:x\">\n\n      <b x=\"1\">text</b>\n <c/>\n</a>").unwrap();
        assert_eq!(cpa_text(&compact), cpa_text(&spread));
        assert!(cpa_text(&compact).contains("\n  <b x=\"1\">text</b>\n  <c/>\n</a>"), "{}", cpa_text(&compact));
    }

    #[test]
    fn no_changes_without_differences() {
        assert!(cpa_changes("a\nb\n", "a\nb\n").is_empty());
    }

    #[test]
    fn a_changed_line_is_removed_and_added_with_its_changed_word() {
        let old = "1\n2\n3\n4\n<End>2026-10-06</End>\n6\n7\n8\n9\n10";
        let new = "1\n2\n3\n4\n<End>2027-10-06</End>\n6\n7\n8\n9\n10";
        let hunks = cpa_changes(old, new);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].line, 2);
        assert_eq!(hunks[0].lines.len(), 8);
        assert_eq!(hunks[0].lines[3], DiffLine::Removed { text: "<End>2026-10-06</End>".into(), parts: vec![(5, 9)] });
        assert_eq!(hunks[0].lines[4], DiffLine::Added { text: "<End>2027-10-06</End>".into(), parts: vec![(5, 9)] });
        assert_eq!(changed_line_count(&hunks), 1);
    }

    #[test]
    fn changed_parts_marks_whole_words() {
        let (old, new) = changed_parts("tns:uri=\"https://a\"", "tns:uri=\"http://a\"");
        assert_eq!(old, vec![(9, 14)]);
        assert_eq!(new, vec![(9, 13)]);
    }

    #[test]
    fn added_lines_without_a_counterpart_are_not_marked() {
        let hunks = cpa_changes("a\nb", "a\nb\nc\nd");
        assert_eq!(
            hunks[0].lines[2..],
            [DiffLine::Added { text: "c".into(), parts: vec![] }, DiffLine::Added { text: "d".into(), parts: vec![] }]
        );
        assert_eq!(changed_line_count(&hunks), 2);
    }
}
