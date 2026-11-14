//! Reduction of endpoint values to the base URL of the Paperless-ngx instance.

use url::Url;

/// Version in which the automatic rewrite of old-style endpoints on config load is removed.
pub const MIGRATION_SUNSET_VERSION: &str = "0.6.0";

/// API path suffixes that are cut off an endpoint, longest first.
const API_SUFFIXES: [&str; 3] = ["/api/documents/post_document", "/api/documents", "/api"];

/// Reduces an endpoint to the base URL by cutting off a trailing API path.
///
/// Endpoints whose URL path ends in `/api/documents/post_document`, `/api/documents`
/// or `/api` (with or without trailing slash) lose that suffix. Everything else,
/// including values that do not parse as a URL or carry a query or fragment,
/// is returned unchanged; rejecting malformed values is the validator's job.
///
/// # Arguments
///
/// * `endpoint` - The endpoint as typed by the user or read from the config file.
///
/// # Returns
///
/// The base URL, or the input unchanged when no API suffix was found.
pub fn normalize_endpoint(endpoint: &str) -> String {
    let Ok(url) = Url::parse(endpoint) else {
        return endpoint.to_string();
    };
    if url.query().is_some() || url.fragment().is_some() {
        return endpoint.to_string();
    }
    // Reason: the suffix is matched against the parsed URL path, not the raw string,
    // so a host named like the suffix (`https://api`) is never cut.
    let path = url.path().trim_end_matches('/');
    let trimmed = endpoint.trim_end_matches('/');
    for suffix in API_SUFFIXES {
        if path.ends_with(suffix) && trimmed.ends_with(suffix) {
            return trimmed[..trimmed.len() - suffix.len()].to_string();
        }
    }
    endpoint.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_endpoint;

    #[test]
    fn test_normalize_endpoint_cuts_api_suffixes() {
        // Expected use: all three suffixes, with and without trailing slash.
        for suffix in ["/api/documents/post_document", "/api/documents", "/api"] {
            assert_eq!(normalize_endpoint(&format!("https://host{suffix}")), "https://host");
            assert_eq!(normalize_endpoint(&format!("https://host{suffix}/")), "https://host");
        }
    }

    #[test]
    fn test_normalize_endpoint_keeps_sub_path() {
        // Edge case: only the API suffix is cut, the sub-path stays.
        assert_eq!(
            normalize_endpoint("https://host/paperless/api/documents/"),
            "https://host/paperless"
        );
        assert_eq!(normalize_endpoint("https://host/paperless"), "https://host/paperless");
    }

    #[test]
    fn test_normalize_endpoint_is_single_pass() {
        // Edge case: only one suffix is removed (design D1).
        assert_eq!(normalize_endpoint("https://host/api/api/"), "https://host/api");
    }

    #[test]
    fn test_normalize_endpoint_leaves_non_suffix_values_unchanged() {
        // Edge cases: partial segment, host named like the suffix, query string, base URL.
        for endpoint in [
            "https://host/myapi",
            "https://api",
            "https://api/",
            "https://host/api?x=1",
            "https://host/api#frag",
            "https://host",
            "https://host/",
        ] {
            assert_eq!(normalize_endpoint(endpoint), endpoint);
        }
    }

    #[test]
    fn test_normalize_endpoint_returns_unparsable_input_unchanged() {
        // Failure case: empty and unparsable values are left for the validator.
        assert_eq!(normalize_endpoint(""), "");
        assert_eq!(normalize_endpoint("host/api/"), "host/api/");
        assert_eq!(normalize_endpoint("not a url"), "not a url");
    }
}
