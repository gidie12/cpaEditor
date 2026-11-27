//! Help files of the application and the Markdown subset they use, independent of the user interface.
//!
//! Port of `insert_markdown` in `application/app_classes/help_window.py`: headings, fenced code blocks, lists,
//! **bold** and `code`. Image lines (screenshots) are left out together with the empty line after them.

use std::sync::LazyLock;

use regex::Regex;

/// The README of the project.
pub const README: &str = include_str!("../../../README.md");
/// The English user guide.
pub const HOW_TO_USE_EN: &str = include_str!("../../../docs/how-to-use.en.md");
/// The Dutch user guide.
pub const HOW_TO_USE_NL: &str = include_str!("../../../docs/how-to-use.nl.md");

/// The entries of the Help menu: (menu label and window title, Markdown text), in menu order.
pub const HELP_FILES: [(&str, &str); 3] =
    [("How to use (English)", HOW_TO_USE_EN), ("Handleiding (Nederlands)", HOW_TO_USE_NL), ("Readme", README)];

/// **bold** and `code` inside a line.
static INLINE_MARKUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*.+?\*\*|`.+?`").expect("valid inline markup pattern"));
/// A line with only an image: ![alt text](path).
static IMAGE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^!\[[^\]]*\]\([^)]*\)$").expect("valid image line pattern"));

/// The kind of a line, which sets its font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A level 1 heading ('# ').
    Title,
    /// A heading of level 2 or deeper ('## ', '### ').
    Heading,
    /// A line of a fenced code block, kept literally.
    Code,
    /// Any other line, with inline markup.
    Normal,
}

/// The style of a part of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStyle {
    Normal,
    Bold,
    Code,
}

/// A part of a line with one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: SpanStyle,
}

impl Span {
    fn new(text: &str, style: SpanStyle) -> Self {
        Span { text: text.to_string(), style }
    }
}

/// A line of the help text. Title and Heading lines have one Normal span, Code lines one Code span, and Normal lines
/// the spans of their inline markup (none for an empty line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpLine {
    pub kind: LineKind,
    pub spans: Vec<Span>,
}

impl HelpLine {
    /// The text of the line without markup.
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

/// The text of the lines without markup, one line per line.
pub fn plain_text(lines: &[HelpLine]) -> String {
    lines.iter().map(HelpLine::text).collect::<Vec<_>>().join("\n")
}

/// Splits a line into spans of **bold**, `code` and normal text.
fn inline_spans(line: &str) -> Vec<Span> {
    let mut parts = Vec::new();
    let mut end = 0;
    for found in INLINE_MARKUP.find_iter(line) {
        parts.push(&line[end..found.start()]);
        parts.push(found.as_str());
        end = found.end();
    }
    parts.push(&line[end..]);
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.starts_with("**") && part.ends_with("**") && part.len() > 4 {
                Span::new(&part[2..part.len() - 2], SpanStyle::Bold)
            } else if part.starts_with('`') && part.ends_with('`') && part.len() > 2 {
                Span::new(&part[1..part.len() - 1], SpanStyle::Code)
            } else {
                Span::new(part, SpanStyle::Normal)
            }
        })
        .collect()
}

/// Converts the Markdown of a help file into lines with styled spans.
pub fn parse_markdown(text: &str) -> Vec<HelpLine> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut after_image = false;
    for line in text.lines() {
        if !in_code_block && IMAGE_LINE.is_match(line.trim()) {
            after_image = true;
            continue;
        }
        if after_image {
            after_image = false;
            if line.trim().is_empty() {
                continue;
            }
        }
        if line.starts_with("```") {
            in_code_block = !in_code_block;
        } else if in_code_block {
            lines.push(HelpLine { kind: LineKind::Code, spans: vec![Span::new(line, SpanStyle::Code)] });
        } else if line.starts_with('#') {
            let without_hashes = line.trim_start_matches('#');
            let level = line.len() - without_hashes.len();
            let kind = if level == 1 { LineKind::Title } else { LineKind::Heading };
            lines.push(HelpLine { kind, spans: vec![Span::new(without_hashes.trim(), SpanStyle::Normal)] });
        } else {
            let spans = match line.strip_prefix("- ") {
                Some(item) => inline_spans(&format!("• {item}")),
                None => inline_spans(line),
            };
            lines.push(HelpLine { kind: LineKind::Normal, spans });
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The texts with this style, like the tag ranges of the Text widget in the Python tests.
    fn styled(lines: &[HelpLine], style: SpanStyle) -> Vec<String> {
        lines
            .iter()
            .flat_map(|line| &line.spans)
            .filter(|span| span.style == style)
            .map(|span| span.text.clone())
            .collect()
    }

    fn of_kind(lines: &[HelpLine], kind: LineKind) -> Vec<String> {
        lines.iter().filter(|line| line.kind == kind).map(HelpLine::text).collect()
    }

    #[test]
    fn test_parse_markdown_formats_headings_and_inline_markup() {
        let lines = parse_markdown("# Title\n## Heading\n- Click **Load CPA** or run `main.py`");
        assert_eq!(plain_text(&lines), "Title\nHeading\n• Click Load CPA or run main.py");
        assert_eq!(of_kind(&lines, LineKind::Title), ["Title"]);
        assert_eq!(of_kind(&lines, LineKind::Heading), ["Heading"]);
        assert_eq!(styled(&lines, SpanStyle::Bold), ["Load CPA"]);
        assert_eq!(styled(&lines, SpanStyle::Code), ["main.py"]);
        assert_eq!(
            lines[2].spans,
            [
                Span::new("• Click ", SpanStyle::Normal),
                Span::new("Load CPA", SpanStyle::Bold),
                Span::new(" or run ", SpanStyle::Normal),
                Span::new("main.py", SpanStyle::Code),
            ]
        );
    }

    #[test]
    fn test_parse_markdown_keeps_code_block_literal() {
        let lines = parse_markdown("```bash\n# not a **heading**\n```\nafter");
        assert_eq!(plain_text(&lines), "# not a **heading**\nafter");
        assert_eq!(of_kind(&lines, LineKind::Code), ["# not a **heading**"]);
        assert_eq!(styled(&lines, SpanStyle::Code), ["# not a **heading**"]);
        assert!(styled(&lines, SpanStyle::Bold).is_empty());
    }

    #[test]
    fn test_parse_markdown_skips_images() {
        let lines = parse_markdown("### General\n\n![General tab](images/general.png)\n\n- Shows the CPA id");
        assert_eq!(plain_text(&lines), "General\n\n• Shows the CPA id");
    }

    #[test]
    fn test_parse_markdown_edge_cases() {
        // Empty markup is not markup, an image inside a code block is kept, CRLF line ends are accepted
        let lines = parse_markdown("**** and `` stay\r\n```\r\n![x](y.png)\r\n```\r\n  - not a bullet");
        assert_eq!(plain_text(&lines), "**** and `` stay\n![x](y.png)\n  - not a bullet");
        assert!(styled(&lines, SpanStyle::Bold).is_empty());
        assert_eq!(lines[0].spans.len(), 1);
    }

    #[test]
    fn test_help_files() {
        let expected = [
            ("How to use (English)", "How to use the CPA Editor"),
            ("Handleiding (Nederlands)", "Handleiding CPA Editor"),
            ("Readme", "Set end date from certificates"),
        ];
        for ((label, text), (expected_label, expected_text)) in HELP_FILES.iter().zip(expected) {
            assert_eq!(*label, expected_label);
            assert!(plain_text(&parse_markdown(text)).contains(expected_text), "{label}");
        }
    }

    #[test]
    fn test_how_to_use_guides_have_same_structure() {
        fn structure(text: &str) -> Vec<&str> {
            // Dutch has one extra paragraph about the English button names
            text.lines()
                .filter(|line| line.starts_with('#') || line.starts_with("- "))
                .map(|line| line.split(' ').next().unwrap_or(""))
                .collect()
        }
        assert_eq!(structure(HOW_TO_USE_EN), structure(HOW_TO_USE_NL));
    }
}
