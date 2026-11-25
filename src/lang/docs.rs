use std::collections::BTreeMap;

use trustworthiness_checker::lang::dsrv::span::Span;

use crate::lang::syntax::lexer::get_comments;

/// One doc line: what it says and where it is written.
#[derive(Debug, Clone)]
struct DocLine {
    content: String,
    span: Span,
}

/// Every doc line in a source, indexed by the line it is on.
#[derive(Debug, Clone, Default)]
pub struct DocComments {
    lines: BTreeMap<u32, DocLine>,
}

impl DocComments {
    /// Find the doc lines in `source`.
    pub fn extract(source: &str) -> Self {
        let lines = get_comments(source)
            .into_iter()
            .filter(|comment| is_doc_marker(&comment.content))
            .filter_map(|comment| {
                let line_start = source[..comment.span.start]
                    .rfind('\n')
                    .map_or(0, |index| index + 1);
                if !source[line_start..comment.span.start]
                    .chars()
                    .all(char::is_whitespace)
                {
                    return None;
                }

                let line = line_of(source, comment.span.start);

                Some((
                    line,
                    DocLine {
                        content: strip_doc_marker(&comment.content).to_owned(),
                        span: Span {
                            start: comment.span.start as u32,
                            end: comment.span.end as u32,
                        },
                    },
                ))
            })
            .collect();

        Self { lines }
    }

    /// The documentation attached to an item beginning on `line`.
    fn for_item_at_line(&self, line: u32) -> Option<String> {
        let mut collected: Vec<&str> = Vec::new();
        let mut current = line.checked_sub(1)?;

        while let Some(doc) = self.lines.get(&current) {
            collected.push(&doc.content);
            match current.checked_sub(1) {
                Some(previous) => current = previous,
                None => break,
            }
        }

        if collected.is_empty() {
            return None;
        }

        collected.reverse();
        Some(collected.join("\n"))
    }

    /// The documentation attached to an item beginning at `offset`.
    pub fn for_item_at(&self, source: &str, offset: usize) -> Option<String> {
        let line = line_of(source, offset);
        self.for_item_at_line(line)
    }

    /// The spans of the doc blocks that documented nothing.
    pub fn unattached(&self, source: &str) -> Vec<Span> {
        let lines: Vec<&str> = source.lines().collect();

        self.lines
            .iter()
            .filter_map(|(line, last)| {
                let below = line + 1;

                if self.lines.contains_key(&below) {
                    return None;
                }

                if !holds_no_item(lines.get(below as usize).copied()) {
                    return None;
                }

                Some(Span {
                    start: self.block_start_offset(*line, last),
                    end: last.span.end,
                })
            })
            .collect()
    }

    /// Where the block of doc lines ending at `last_line` begins.
    fn block_start_offset(&self, last_line: u32, last: &DocLine) -> u32 {
        let mut start = last.span.start;
        let mut line = last_line;

        while let Some(above) = line.checked_sub(1) {
            match self.lines.get(&above) {
                Some(doc) => {
                    start = doc.span.start;
                    line = above;
                }
                None => break,
            }
        }

        start
    }
}

fn is_doc_marker(content: &str) -> bool {
    content.starts_with("///") && !content.starts_with("////")
}

fn strip_doc_marker(content: &str) -> &str {
    let rest = &content[3..];
    rest.strip_prefix(' ').unwrap_or(rest)
}

fn line_of(source: &str, offset: usize) -> u32 {
    source[..offset].matches("\n").count() as u32
}

fn holds_no_item(line: Option<&str>) -> bool {
    match line {
        None => true,
        Some(text) => {
            let text = text.trim_start();
            text.is_empty() || text.starts_with("//") || text.starts_with("(*")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// The documentation attached to the item on `line` of `source`.
    fn doc_at(source: &str, line: u32) -> Option<String> {
        DocComments::extract(source).for_item_at_line(line)
    }

    fn unattached_of(source: &str) -> Vec<Span> {
        DocComments::extract(source).unattached(source)
    }

    #[test]
    fn three_slashes_open_a_doc_comment() {
        assert!(is_doc_marker("///"));
        assert!(is_doc_marker("/// text"));
        assert!(is_doc_marker("///text"));
    }

    #[test]
    fn other_comment_forms_do_not() {
        assert!(!is_doc_marker("// text"));
        assert!(!is_doc_marker("//! text"));
        // A separator line, not documentation.
        assert!(!is_doc_marker("//// text"));
        assert!(!is_doc_marker("////////////////"));
    }

    #[test]
    fn stripping_removes_the_marker_and_one_space() {
        assert_eq!(strip_doc_marker("///text"), "text");
        assert_eq!(strip_doc_marker("/// text"), "text");
        // The second space survives: it is the author's indentation, and four
        // of them would be a code block in Markdown.
        assert_eq!(strip_doc_marker("///  text"), " text");
        assert_eq!(strip_doc_marker("///"), "");
    }

    #[test]
    fn a_single_line_attaches_to_the_item_below() {
        let source = "/// The sensor reading.\nin x: Int";
        assert_eq!(doc_at(source, 1).as_deref(), Some("The sensor reading."));
    }

    #[test]
    fn a_run_attaches_in_source_order() {
        let source = "/// First.\n/// Second.\n/// Third.\nin x: Int";
        assert_eq!(
            doc_at(source, 3).as_deref(),
            Some("First.\nSecond.\nThird.")
        );
    }

    #[test]
    fn a_bare_marker_becomes_an_empty_line() {
        // Which Markdown reads as a paragraph break.
        let source = "/// Summary.\n///\n/// Detail.\nin x: Int";
        assert_eq!(doc_at(source, 3).as_deref(), Some("Summary.\n\nDetail."));
    }

    #[test]
    fn a_blank_line_detaches_the_run() {
        let source = "/// Not about x.\n\nin x: Int";
        assert_eq!(doc_at(source, 2), None);
    }

    #[test]
    fn an_ordinary_comment_detaches_the_run() {
        let source = "/// Not about x.\n// A note to self.\nin x: Int";
        assert_eq!(doc_at(source, 2), None);
    }

    #[test]
    fn indentation_before_the_marker_is_allowed() {
        let source = "    /// The sensor reading.\nin x: Int";
        assert_eq!(doc_at(source, 1).as_deref(), Some("The sensor reading."));
    }

    #[test]
    fn a_trailing_comment_is_not_documentation() {
        let source = "in x: Int /// The sensor reading.\nout z: Int";
        assert_eq!(doc_at(source, 1), None);
    }

    #[test]
    fn a_marker_inside_a_block_comment_is_not_documentation() {
        // Commenting out a documented declaration is an ordinary thing to do,
        // and the doc lines inside must not attach to the next real item.
        let source = "(*\n/// The old total.\nout old: Int\n*)\nout total: Int";
        assert_eq!(doc_at(source, 4), None);
    }

    #[test]
    fn an_item_with_nothing_above_it_has_no_documentation() {
        assert_eq!(doc_at("in x: Int", 0), None);
        assert_eq!(doc_at("\nin x: Int", 1), None);
    }

    #[test]
    fn a_run_reaching_the_top_of_the_file_still_attaches() {
        let source = "/// First.\n/// Second.\nin x: Int";
        assert_eq!(doc_at(source, 2).as_deref(), Some("First.\nSecond."));
    }

    #[test]
    fn a_block_at_the_end_of_the_file_attaches_to_nothing() {
        let source = "in x: Int\n/// Nothing follows this.";
        let spans = unattached_of(source);

        assert_eq!(spans.len(), 1, "{spans:?}");
        assert_eq!(spans[0].start as usize, source.find("///").unwrap());
    }

    #[test]
    fn a_blank_line_leaves_the_block_unattached() {
        let source = "/// Not about x.\n\nin x: Int";
        assert_eq!(unattached_of(source).len(), 1);
    }

    #[test]
    fn an_ordinary_comment_leaves_the_block_unattached() {
        let source = "/// Not about x.\n// A note to self.\nin x: Int";
        assert_eq!(unattached_of(source).len(), 1);
    }

    #[test]
    fn an_attached_block_is_not_reported() {
        let source = "/// The sensor reading.\nin x: Int\nout z: Int\nz = x + 1";
        assert!(unattached_of(source).is_empty());
    }

    #[test]
    fn a_three_line_block_is_reported_once_over_all_three() {
        let source = "/// One.\n/// Two.\n/// Three.\n\nin x: Int";
        let spans = unattached_of(source);

        assert_eq!(spans.len(), 1, "{spans:?}");

        let last = source.find("/// Three.").unwrap();
        assert_eq!(spans[0].start as usize, source.find("/// One.").unwrap());
        assert_eq!(spans[0].end as usize, last + "/// Three.".len());
    }

    #[test]
    fn a_file_level_comment_is_not_reported() {
        let source = "//! What this specification does.\n\nin x: Int";
        assert!(unattached_of(source).is_empty());
    }

    #[test]
    fn a_doc_above_a_def_is_not_reported() {
        let source = "/// A moving average over three ticks.\n\
                            def smooth(s: Int) = (s + s[1] + s[2]) / 3";
        assert!(unattached_of(source).is_empty());
    }

    #[test]
    fn a_doc_above_a_generic_type_alias_is_not_reported() {
        let source = "/// A list of lists.\n\
                            type NestedList<X> = List<List<X>>";
        assert!(unattached_of(source).is_empty());
    }
}
