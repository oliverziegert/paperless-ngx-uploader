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
    let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    // Truncate the name to a maximum length of `MAX_TITLE_LENGTH`.
    // This is a safety precaution to avoid overflowing the Paperless-ngx title field.
    name.split_at(min(name.len(), MAX_TITLE_LENGTH)).0.to_string()
}

/// API path of the Paperless-ngx API root, relative to the base URL.
pub(crate) const API_STATUS_PATH: &str = "api/";

/// API path of the Paperless-ngx document upload, relative to the base URL.
pub(crate) const API_UPLOAD_PATH: &str = "api/documents/post_document/";

/// Composes a request URL from the stored base URL and an API path.
///
/// # Arguments
///
/// * `endpoint` - Base URL of the Paperless-ngx instance, with or without trailing slash.
/// * `path` - API path without leading slash (e.g. `API_UPLOAD_PATH`).
///
/// # Returns
///
/// The base URL and the path joined by exactly one slash.
pub(crate) fn api_url(endpoint: &str, path: &str) -> String {
    // Reason: `Url::join` would drop the last segment of a sub-path base without
    // trailing slash (`https://host/paperless` + `api/` gives `https://host/api/`).
    format!("{}/{}", endpoint.trim_end_matches('/'), path)
}

#[cfg(test)]
mod tests {
    use super::{api_url, API_STATUS_PATH, API_UPLOAD_PATH};

    #[test]
    fn test_api_url_without_trailing_slash() {
        // Expected use: base URL at the host root.
        assert_eq!(
            api_url("https://host", API_UPLOAD_PATH),
            "https://host/api/documents/post_document/"
        );
        assert_eq!(api_url("https://host", API_STATUS_PATH), "https://host/api/");
    }

    #[test]
    fn test_api_url_with_trailing_slash() {
        // Edge case: a trailing slash must not produce a double slash.
        assert_eq!(
            api_url("https://host/", API_UPLOAD_PATH),
            "https://host/api/documents/post_document/"
        );
    }

    #[test]
    fn test_api_url_keeps_sub_path() {
        // Edge case: sub-path installs keep their last path segment.
        assert_eq!(
            api_url("https://host/paperless/", API_STATUS_PATH),
            "https://host/paperless/api/"
        );
        assert_eq!(
            api_url("https://host/paperless", API_UPLOAD_PATH),
            "https://host/paperless/api/documents/post_document/"
        );
    }

    #[test]
    fn test_api_url_empty_endpoint_stays_relative() {
        // Failure case: an empty endpoint yields no usable URL; the request fails later.
        assert_eq!(api_url("", API_STATUS_PATH), "/api/");
    }
}
