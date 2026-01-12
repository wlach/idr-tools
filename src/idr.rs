use std::path::{Path, PathBuf};

use jiff::Timestamp;
use minijinja::{Environment, context};
use regex::Regex;
use slug::slugify;

pub fn get_idr_filename(title: &str, timestamp: Timestamp) -> String {
    format!("{}-{}.md", timestamp.strftime("%Y%m%d%H%M"), slugify(title))
}

/// Semi-intelligent getting of idr path:
/// If there's a directory called "idrs" in the present directory, use that.
/// Otherwise, just return the current path.
pub fn get_idr_path(path: &Path, filename: String) -> PathBuf {
    let idrs_path = path.join("idrs");
    if idrs_path.is_dir() {
        idrs_path.join(filename)
    } else {
        path.join(filename)
    }
}

/// Strips HTML comments out of a markdown file
pub fn strip_html_comments(text: &str) -> String {
    // Remove HTML comments (including multiline)
    let re_comments = Regex::new(r"(?s)<!--.*?-->").unwrap();
    let mut result = re_comments.replace_all(text, "").to_string();

    // Clean up any blank lines left by removing comments
    // First normalize all whitespace-only lines to actual blank lines
    let re_whitespace_lines = Regex::new(r"(?m)^\s+$").unwrap();
    result = re_whitespace_lines.replace_all(&result, "").to_string();

    // Then collapse multiple consecutive blank lines (3+) to exactly 2 newlines
    let re_multiple_blanks = Regex::new(r"\n{3,}").unwrap();
    result = re_multiple_blanks.replace_all(&result, "\n\n").to_string();

    // Finally, strip any leading/trailing whitespace from the entire document
    result.trim().to_string()
}

/// Render IDR template with given parameters
pub fn render_idr_template(
    title: &str,
    owner: &str,
    timestamp: Timestamp,
) -> anyhow::Result<String> {
    let mut env = Environment::new();
    env.add_template("idr", include_str!("./default_template.md.jinja"))?;
    let tmpl = env.get_template("idr")?;

    let now_yyyymmdd = timestamp.strftime("%Y-%m-%d").to_string();

    let rendered = tmpl.render(context!(
        date => now_yyyymmdd,
        title => title,
        owner => owner,
    ))?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idr_filename() {
        // Create a timestamp: 2026-12-25 15:45:00 UTC
        let timestamp = "2026-12-25T15:45:00Z".parse::<Timestamp>().unwrap();
        let filename = get_idr_filename("My Test IDR", timestamp);
        assert_eq!(filename, "202612251545-my-test-idr.md");
    }

    #[test]
    fn test_get_idr_filename_special_chars() {
        let timestamp = "2026-12-25T15:45:00Z".parse::<Timestamp>().unwrap();
        let filename = get_idr_filename("Hello & World!", timestamp);
        assert_eq!(filename, "202612251545-hello-world.md");
    }

    #[test]
    fn test_get_idr_path_without_idrs_dir() {
        // When idrs/ doesn't exist, should return base path + filename
        let base = Path::new("/tmp/test");
        let filename = "202601051430-test.md".to_string();
        let result = get_idr_path(base, filename.clone());
        assert_eq!(result, base.join(filename));
    }

    #[test]
    fn test_strip_html_comments_simple() {
        let input = "Hello <!-- comment --> World";
        let output = strip_html_comments(input);
        assert_eq!(output, "Hello  World");
    }

    #[test]
    fn test_strip_html_comments_multiline() {
        let input = r#"Line 1
<!-- This is a
multiline
comment -->
Line 2"#;
        let output = strip_html_comments(input);
        assert_eq!(output, "Line 1\n\nLine 2");
    }

    #[test]
    fn test_strip_html_comments_multiple() {
        let input = r#"Start
<!-- Comment 1 -->
Middle
<!-- Comment 2 -->
End"#;
        let output = strip_html_comments(input);
        assert_eq!(output, "Start\n\nMiddle\n\nEnd");
    }

    #[test]
    fn test_strip_html_comments_collapse_blank_lines() {
        let input = "Line 1\n\n\n\n\nLine 2";
        let output = strip_html_comments(input);
        assert_eq!(output, "Line 1\n\nLine 2");
    }

    #[test]
    fn test_strip_html_comments_trim_whitespace() {
        let input = "   \n\nContent\n\n   ";
        let output = strip_html_comments(input);
        assert_eq!(output, "Content");
    }

    #[test]
    fn test_strip_html_comments_no_comments() {
        let input = "Just plain text";
        let output = strip_html_comments(input);
        assert_eq!(output, "Just plain text");
    }

    #[test]
    fn test_render_idr_template() {
        let timestamp = "2026-01-12T12:00:00Z".parse::<Timestamp>().unwrap();
        let result = render_idr_template("Test Title", "Test User <test@example.com>", timestamp);

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("# 2026-01-12: Test Title"));
        assert!(content.contains("Owner: Test User <test@example.com>"));
        assert!(content.contains("## Overview"));
    }
}
