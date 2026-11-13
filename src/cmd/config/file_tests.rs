//! Tests for the YAML file format handled by `PublicConfig` (`file.rs`).

use crate::cmd::config::models::PublicConfig;
use crate::cmd::config::HandleConfig;
use crate::cmd::models::CmdError;
use tempfile::TempDir;

/// Builds a `PublicConfig` with placeholder values pointing at `path`.
///
/// # Arguments
///
/// * `path` - Location of the config file to load from or save to.
///
/// # Returns
///
/// A config whose values differ from anything the tests write to disk.
fn config_at(path: std::path::PathBuf) -> PublicConfig {
    PublicConfig { endpoint: "placeholder".to_string(), allow_insecure: true, path }
}

#[test]
fn test_load_reads_file_written_by_previous_versions() {
    // Expected use: this is the exact layout `serde_yaml` (via `confy`) wrote,
    // so configs created by older releases must keep loading.
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("config.yaml");
    std::fs::write(&path, "endpoint: https://paperless.example.com\nallow_insecure: false\n")
        .unwrap();

    let mut config = config_at(path);
    config.load().unwrap();

    assert_eq!(config.endpoint, "https://paperless.example.com");
    assert!(!config.allow_insecure);
}

#[test]
fn test_load_missing_file_writes_defaults_into_new_directory() {
    // Edge case: neither the file nor its parent directory exists yet.
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("nested").join("config.yaml");

    let mut config = config_at(path.clone());
    config.load().unwrap();

    assert_eq!(config.endpoint, PublicConfig::default().endpoint);
    assert!(!config.allow_insecure);
    assert!(std::fs::read_to_string(&path).unwrap().contains("endpoint:"));
}

#[test]
fn test_load_malformed_yaml_fails() {
    // Failure case: the file exists but is not a valid config.
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("config.yaml");
    std::fs::write(&path, "endpoint: [unclosed\n").unwrap();

    let result = config_at(path).load();

    assert!(matches!(result, Err(CmdError::ConfigFileLoadFailed(_))));
}
