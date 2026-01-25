//! Helper functions and utilities for client operations.
//!
//! This module contains shared helper functions and utility code
//! used across the client functionality.

use std::cmp::min;

pub(crate) const MAX_TITLE_LENGTH: usize = 64;

/// Returns the title of the file based on its filename.
///
/// The title is the filename without its extension, truncated to a maximum length of
/// `MAX_TITLE_LENGTH`. This is used as the title when uploading the file to Paperless-ngx.
pub(crate) fn get_title_from_filename(file: &std::path::Path) -> String {
    let file_name = match file.file_name() {
        Some(name) => name.to_str().unwrap_or(""),
        None => "",
    };
    let extension = match file.extension() {
        Some(ext) => format!(".{}", ext.to_str().unwrap_or("")),
        None => "".to_string(),
    };
    let name = file_name.replace(extension.as_str(), "");
    // Truncate the name to a maximum length of `MAX_TITLE_LENGTH`.
    // This is a safety precaution to avoid overflowing the Paperless-ngx title field.
    let title = name.split_at(min(name.len(), MAX_TITLE_LENGTH)).0;
    title.into()
}
