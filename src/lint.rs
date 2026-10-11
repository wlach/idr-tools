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
    let mut html_comments = HtmlComments::default();
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
                html_comments.scan(&html, |start| {
                    findings.push(Diagnostic::comment(line_at(offset + start)));
                });
            }
            // Code blocks emit text, not HTML or heading events, so examples
            // (including nested fences and indented code) cannot trigger rules.
            _ => {}
        }
    }
    findings.sort_by_key(|finding| (finding.line, finding.code));
    Ok(findings)
}

/// Markdown emits whole inline tags but may split block HTML across lines.
/// Track tags and quotes across chunks so attribute values aren't comments.
#[derive(Default)]
struct HtmlComments {
    in_comment: bool,
    in_tag: bool,
    quote: Option<u8>,
}

impl HtmlComments {
    fn scan(&mut self, html: &str, mut report: impl FnMut(usize)) {
        let bytes = html.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if self.in_comment {
                if bytes[i..].starts_with(b"-->") {
                    self.in_comment = false;
                    i += 3;
                    continue;
                }
            } else if self.in_tag {
                match (self.quote, bytes[i]) {
                    (Some(quote), byte) if quote == byte => self.quote = None,
                    (None, b'\'' | b'"') => self.quote = Some(bytes[i]),
                    (None, b'>') => self.in_tag = false,
                    _ => {}
                }
            } else if bytes[i..].starts_with(b"<!--") {
                report(i);
                i += 4;
                // HTML also accepts these abruptly closed empty comments.
                if bytes[i..].starts_with(b">") {
                    i += 1;
                } else if bytes[i..].starts_with(b"->") {
                    i += 2;
                } else {
                    self.in_comment = true;
                }
                continue;
            } else if bytes[i] == b'<'
                && bytes
                    .get(i + 1)
                    .is_some_and(|b| b.is_ascii_alphabetic() || b"/!?".contains(b))
            {
                self.in_tag = true;
            }
            i += 1;
        }
    }
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

    #[test]
    fn lint_cases() {
        // Each row names the behavior and gives the exact source lines/codes.
        type Case = (&'static str, &'static str, &'static [(usize, &'static str)]);
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("all rules", "---\ndraft: true\n---\n# Title\n<!-- guidance\nmore -->\n## Working notes (ephemeral)\n",
                &[(2, "IDR001"), (5, "IDR002"), (7, "IDR003")]),
            ("unrelated metadata", "---\nstatus: draft\nother:\n  draft: true\nsummary: |\n  draft: true\n---\n# Title\n\ndraft: true\n", &[]),
            ("draft after nested value", "---\nother: [draft, false]\ndraft: false\n---\n", &[(3, "IDR001")]),
            ("nested backtick fence", "````md\n---\ndraft: true\n---\n```\n<!-- example -->\n## Working notes\n````\n<!-- real -->\n", &[(9, "IDR002")]),
            ("tilde fence", "~~~\n<!-- example -->\n## Working notes\n~~~\n## Working notes\n", &[(5, "IDR003")]),
            ("indented code", "    <!-- example -->\n    ## Working notes\n", &[]),
            ("inline code", "`<!-- example -->` and `## Working notes`\n", &[]),
            ("quoted fence", "> ```md\n> <!-- example -->\n> ## Working notes\n> ```\n", &[]),
            ("list fence", "- Example:\n\n  ~~~\n  <!-- example -->\n  ## Working notes\n  ~~~\n", &[]),
            ("multiline and inline comments", "<!--\n## Working notes\n-->\nText <!-- one --> and <!-- two -->\n<!-- unfinished\n",
                &[(1, "IDR002"), (4, "IDR002"), (4, "IDR002"), (5, "IDR002")]),
            ("heading variants", "### WORKING notes ###\n\nWorking notes (ephemeral)\n-------------------------\n\n## **Working notes**\n",
                &[(1, "IDR003"), (3, "IDR003"), (6, "IDR003")]),
            ("prose and different heading", "Working notes are temporary.\n\n## Working notes on something else\n", &[]),
            ("BOM, CRLF, Unicode", "\u{feff}---\r\nowner: Zoë\r\ndraft: false\r\n...\r\nCafé <!-- remove -->\r\n",
                &[(3, "IDR001"), (5, "IDR002")]),
            ("final record", "---\nowner: Someone\n---\n# A decision\n\n## Overview\nWe chose this to solve a problem.\n", &[]),
            ("empty document", "", &[]),
            ("double-quoted attribute", "<span title=\"<!-- example -->\">Example</span>\n", &[]),
            ("single-quoted attribute and real comment", "<span title='<!-- example -->'>Example</span><!-- real -->\n", &[(1, "IDR002")]),
            ("multiline block attribute", "<div title=\"\n<!-- example -->\n\">\n<!-- real -->\n</div>\n", &[(4, "IDR002")]),
            ("attribute with greater-than sign", "<div title=\"> <!-- example -->\">\n<!-- real -->\n</div>\n", &[(2, "IDR002")]),
            ("quotes in text outside tags", "<div>\nIt's <!-- real -->\n</div>\n", &[(2, "IDR002")]),
            ("nested opener inside comment", "<!-- outer <!-- inner -->\n<!-- real -->\n", &[(1, "IDR002"), (2, "IDR002")]),
            ("abruptly closed empty comments", "<!-->\n\n<!--->\n\n<!-- real -->\n", &[(1, "IDR002"), (3, "IDR002"), (5, "IDR002")]),
        ];
        for &(name, input, expected) in cases {
            let findings = lint_idr(input).unwrap_or_else(|error| panic!("{name}: {error}"));
            let actual: Vec<_> = findings.iter().map(|d| (d.line, d.code)).collect();
            assert_eq!(actual, expected, "{name}");
        }
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
                lint_idr(&format!("---\n{field}\n---\n# Title")).unwrap(),
                vec![Diagnostic::draft(2)],
                "{field}"
            );
        }
    }

    #[test]
    fn malformed_frontmatter_is_an_error() {
        for input in ["---\ndraft: [\n---\n", "---\ndraft: true\n"] {
            assert!(lint_idr(input).is_err(), "{input:?}");
        }
    }
}
