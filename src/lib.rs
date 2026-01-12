use std::fs;
use std::path::PathBuf;

use jiff::Timestamp;

mod git;
pub mod idr;

/// Create a new Implementation Decision Record
///
/// # Arguments
///
/// * `dir` - Directory where the IDR should be created
/// * `title` - Title of the IDR
/// * `no_comments` - Whether to strip HTML comments from the template
///
/// # Returns
///
/// PathBuf of the created IDR file
pub fn create_idr(dir: PathBuf, title: &str, no_comments: bool) -> anyhow::Result<PathBuf> {
    let now = Timestamp::now();
    let owner = git::get_identity().unwrap_or_else(|| "Unknown <unknown>".to_string());

    let mut rendered_output = idr::render_idr_template(title, &owner, now)?;

    if no_comments {
        rendered_output = idr::strip_html_comments(&rendered_output);
    }

    let filename = idr::get_idr_filename(title, now);
    let path = idr::get_idr_path(dir.as_path(), filename);

    fs::write(&path, rendered_output)?;

    Ok(path)
}
