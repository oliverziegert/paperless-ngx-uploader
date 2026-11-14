//! Tests for the endpoint handling of `init` (`main.rs`).
//!
//! `init` itself prompts for the token on the terminal, so the successful
//! paths are tested through `set_endpoint`, the part of `init` before the prompt.

use super::{init, set_endpoint};
use crate::cmd::config::Config;
use tempfile::TempDir;

#[test]
fn test_init_reduces_endpoint_with_api_path() {
    // Expected use: the full upload URL is stored as base URL and reported as reduced.
    let mut cfg = Config::default();

    let reduced =
        set_endpoint("https://host/api/documents/post_document/", false, &mut cfg).unwrap();

    assert!(reduced);
    assert_eq!(cfg.public_config.endpoint, "https://host");
}

#[test]
fn test_init_keeps_base_url_unchanged() {
    // Edge case: a base URL with sub-path is stored as given, no reduction message.
    let mut cfg = Config::default();

    let reduced = set_endpoint("https://host/paperless", false, &mut cfg).unwrap();

    assert!(!reduced);
    assert_eq!(cfg.public_config.endpoint, "https://host/paperless");
}

#[test]
fn test_init_rejects_invalid_endpoint_and_saves_nothing() {
    // Failure case: `init` fails before the token prompt and writes no config file.
    let temp_dir = TempDir::new().unwrap();
    let mut cfg = Config::default();
    cfg.public_config.path = temp_dir.path().join("config.yaml");

    let result = init(Some("host/api/".to_string()), false, &mut cfg);

    assert!(result.is_err());
    assert!(!cfg.public_config.path.exists());
}
