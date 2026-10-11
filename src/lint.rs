use anyhow::{Context, bail};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use yaml_rust2::parser::{Event as YamlEvent, MarkedEventReceiver, Parser as YamlParser};
use yaml_rust2::scanner::Marker;

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub code: &'static str,
    pub message: &'static str,
}

impl Diagnostic {
    fn draft(line: usize) -> Self {
        Self {
            line,
            code: "IDR001",
            message: "Draft metadata remains; remove the `draft` field when finalizing",
        }
    }

    fn comment(line: usize) -> Self {
        Self {
            line,
            code: "IDR002",
            message: "Markdown comment remains",
        }
    }

    fn working_notes(line: usize) -> Self {
        Self {
            line,
            code: "IDR003",
            message: "Working notes section remains",
        }
    }
}

/// Check finalization markers without modifying the document or assessing its design.
pub fn lint_idr(text: &str) -> anyhow::Result<Vec<Diagnostic>> {
    let mut findings = Vec::new();
    let (body, body_offset) = frontmatter(text, &mut findings)?;
    let mut line_starts = vec![0];
    line_starts.extend(text.match_indices('\n').map(|(offset, _)| offset + 1));
    let line_at = |offset| line_starts.partition_point(|&start| start <= offset);

    let mut heading: Option<(usize, String)> = None;
    let mut in_comment = false;
    for (event, range) in Parser::new(body).into_offset_iter() {
        let offset = body_offset + range.start;
        match event {
            Event::Start(Tag::Heading { .. }) => {
                heading = Some((line_at(offset), String::new()));
            }
            Event::Text(value) | Event::Code(value) => {
                if let Some((_, title)) = &mut heading {
                    title.push_str(&value);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, title)) = &mut heading {
                    title.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((line, title)) = heading.take() {
                    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
                    if title.eq_ignore_ascii_case("Working notes")
                        || title.eq_ignore_ascii_case("Working notes (ephemeral)")
                    {
                        findings.push(Diagnostic::working_notes(line));
                    }
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                // Block HTML may arrive a line at a time. Keep comment state so
                // a multiline comment produces one finding at its opening line.
                let mut cursor = 0;
                while cursor < html.len() {
                    let needle = if in_comment { "-->" } else { "<!--" };
                    let Some(relative) = html[cursor..].find(needle) else {
                        break;
                    };
                    let start = cursor + relative;
                    if !in_comment {
                        findings.push(Diagnostic::comment(line_at(offset + start)));
                    }
                    in_comment = !in_comment;
                    cursor = start + needle.len();
                }
            }
            // Code blocks emit text, not HTML or heading events, so examples
            // (including nested fences and indented code) cannot trigger rules.
            _ => {}
        }
    }
    findings.sort_by_key(|finding| (finding.line, finding.code));
    Ok(findings)
}

/// Only the document's initial YAML block is metadata. Body examples and
/// nested YAML keys do not constitute a draft field on the IDR itself.
fn frontmatter<'a>(
    text: &'a str,
    findings: &mut Vec<Diagnostic>,
) -> anyhow::Result<(&'a str, usize)> {
    let text_without_bom = text.trim_start_matches('\u{feff}');
    let bom_len = text.len() - text_without_bom.len();
    let mut lines = text_without_bom.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return Ok((text, 0));
    };
    if first.trim_end() != "---" {
        return Ok((text, 0));
    }
    let yaml_start = bom_len + first.len();
    let mut end = yaml_start;
    for line in lines {
        if matches!(line.trim_end(), "---" | "...") {
            let mut receiver = DraftFields::default();
            YamlParser::new_from_str(&text[yaml_start..end])
                .load(&mut receiver, true)
                .context("Invalid YAML frontmatter (locations are relative to the YAML block)")?;
            // YAML line 1 is document line 2, after the opening delimiter.
            findings.extend(
                receiver
                    .lines
                    .into_iter()
                    .map(|line| Diagnostic::draft(line + 1)),
            );
            let body_start = end + line.len();
            return Ok((&text[body_start..], body_start));
        }
        end += line.len();
    }
    bail!("Unclosed YAML frontmatter; expected a closing --- or ... delimiter")
}

#[derive(Default)]
struct DraftFields {
    depth: usize,
    root_mapping: bool,
    expecting_key: bool,
    lines: Vec<usize>,
}

impl MarkedEventReceiver for DraftFields {
    fn on_event(&mut self, event: YamlEvent, mark: Marker) {
        match event {
            YamlEvent::MappingStart(..) | YamlEvent::SequenceStart(..) => {
                if self.depth == 0 {
                    self.root_mapping = matches!(event, YamlEvent::MappingStart(..));
                    self.expecting_key = true;
                } else if self.depth == 1 && self.root_mapping {
                    self.expecting_key = !self.expecting_key;
                }
                self.depth += 1;
            }
            YamlEvent::MappingEnd | YamlEvent::SequenceEnd => self.depth -= 1,
            YamlEvent::Scalar(..) | YamlEvent::Alias(..)
                if self.depth == 1 && self.root_mapping =>
            {
                if self.expecting_key
                    && let YamlEvent::Scalar(key, ..) = event
                    && key == "draft"
                {
                    self.lines.push(mark.line());
                }
                self.expecting_key = !self.expecting_key;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(text: &str) -> Vec<(usize, &'static str)> {
        lint_idr(text)
            .unwrap()
            .into_iter()
            .map(|d| (d.line, d.code))
            .collect()
    }

    #[test]
    fn reports_all_rules_at_source_lines() {
        assert_eq!(
            codes(
                "---\ndraft: true\n---\n# Title\n<!-- guidance\nmore -->\n## Working notes (ephemeral)\n"
            ),
            vec![(2, "IDR001"), (5, "IDR002"), (7, "IDR003")]
        );
    }

    #[test]
    fn draft_presence_matters_not_value_or_yaml_spelling() {
        for field in [
            "draft: false",
            "'draft': null",
            "\"draft\": []",
            "{draft: anything}",
            "draft:\n  nested: value",
        ] {
            assert_eq!(
                codes(&format!("---\n{field}\n---\n# Title")),
                vec![(2, "IDR001")]
            );
        }
    }

    #[test]
    fn ignores_unrelated_metadata_and_nested_draft_keys() {
        assert!(codes("---\nstatus: draft\nother:\n  draft: true\nsummary: |\n  draft: true\n---\n# Title\n\ndraft: true\n").is_empty());
        assert_eq!(
            codes("---\nother: [draft, false]\ndraft: false\n---\n"),
            vec![(3, "IDR001")]
        );
    }

    #[test]
    fn ignores_code_examples_and_preserves_following_line_numbers() {
        let text = "````markdown\n---\ndraft: true\n---\n```\n<!-- example -->\n## Working notes\n````\n~~~\n<!-- example -->\n## Working notes\n~~~\n\n    <!-- indented -->\n    ## Working notes\n\n`<!-- inline -->` and `## Working notes`\n\n<!-- actual -->\n## Working notes\n";
        assert_eq!(codes(text), vec![(19, "IDR002"), (20, "IDR003")]);
    }

    #[test]
    fn ignores_fences_in_lists_and_quotes() {
        assert!(codes("> ```md\n> <!-- example -->\n> ## Working notes\n> ```\n\n- Example:\n\n  ~~~\n  <!-- example -->\n  ## Working notes\n  ~~~\n").is_empty());
    }

    #[test]
    fn comments_are_reported_once_each_and_hide_headings() {
        assert_eq!(
            codes(
                "<!--\n## Working notes\n-->\nText <!-- one --> and <!-- two -->\n<!-- unfinished\n"
            ),
            vec![(1, "IDR002"), (4, "IDR002"), (4, "IDR002"), (5, "IDR002")]
        );
    }

    #[test]
    fn recognizes_heading_syntax_without_matching_prose() {
        assert_eq!(
            codes(
                "### WORKING notes ###\n\nWorking notes (ephemeral)\n-------------------------\n\n## **Working notes**\n\nWorking notes are temporary.\n\n## Working notes on something else\n"
            ),
            vec![(1, "IDR003"), (3, "IDR003"), (6, "IDR003")]
        );
    }

    #[test]
    fn handles_bom_crlf_and_unicode() {
        assert_eq!(
            codes("\u{feff}---\r\nowner: Zoë\r\ndraft: false\r\n...\r\nCafé <!-- remove -->\r\n"),
            vec![(3, "IDR001"), (5, "IDR002")]
        );
    }

    #[test]
    fn malformed_frontmatter_is_an_error() {
        assert!(lint_idr("---\ndraft: [\n---\n").is_err());
        assert!(lint_idr("---\ndraft: true\n").is_err());
    }

    #[test]
    fn finalized_document_passes() {
        assert!(codes("---\nowner: Someone\n---\n# A decision\n\n## Overview\nWe chose this to solve a problem.\n").is_empty());
        assert!(codes("").is_empty());
    }
}
