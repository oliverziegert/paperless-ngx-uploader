use super::*;
use crate::cmd::config::models::{Config, PrivateConfig, PublicConfig};
use crate::cmd::models::CmdError;
use log::LevelFilter;
use tempfile::TempDir;

/// Sets up the test logger to capture log output during tests.
/// Also configures the keyring to use the mock credential store.
fn setup_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // Set up mock credential store for testing
    // Use ::keyring to refer to the crate, not the local module
    ::keyring::set_default_credential_builder(::keyring::mock::default_credential_builder());
}

#[cfg(test)]
mod public_config_tests {
    use super::*;

    #[test]
    fn test_load_existing_config() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a config file with known content
        let initial_config = PublicConfig {
            endpoint: "http://test.example.com".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        initial_config.save().unwrap();

        // Load the config
        let mut loaded_config = PublicConfig {
            endpoint: String::new(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        loaded_config.load().unwrap();

        // Verify the loaded endpoint matches
        assert_eq!(loaded_config.endpoint, "http://test.example.com");
    }

    #[test]
    fn test_load_nonexistent_config_uses_defaults() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yaml");

        // Try to load a non-existent config
        let mut config = PublicConfig {
            endpoint: "default_endpoint".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };

        // This should succeed because confy creates default config
        let result = config.load();
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_creates_config_file() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create and save a config
        let config = PublicConfig {
            endpoint: "http://save-test.example.com".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        config.save().unwrap();

        // Verify the file was created
        assert!(config_path.exists());

        // Verify we can load it back
        let mut loaded_config = PublicConfig {
            endpoint: String::new(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        loaded_config.load().unwrap();
        assert_eq!(loaded_config.endpoint, "http://save-test.example.com");
    }

    #[test]
    fn test_delete_removes_config_file() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a config file
        let config = PublicConfig {
            endpoint: "http://delete-test.example.com".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        config.save().unwrap();
        assert!(config_path.exists());

        // Delete the config
        config.delete().unwrap();

        // Verify the file was deleted
        assert!(!config_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_file_returns_error() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yaml");

        let config = PublicConfig {
            endpoint: "http://test.example.com".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };

        // Attempting to delete a non-existent file should error
        let result = config.delete();
        assert!(result.is_err());

        // Verify it's the correct error type
        match result {
            Err(CmdError::ConfigFileDeletionFailed(_)) => (),
            _ => panic!("Expected ConfigFileDeletionFailed error"),
        }
    }

    #[test]
    fn test_roundtrip_preserves_data() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let original_endpoint = "http://roundtrip.example.com:8080";

        // Save config
        let config = PublicConfig {
            endpoint: original_endpoint.to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        config.save().unwrap();

        // Load it back
        let mut loaded_config = PublicConfig {
            endpoint: String::new(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        loaded_config.load().unwrap();

        // Verify data is preserved
        assert_eq!(loaded_config.endpoint, original_endpoint);
    }
}

#[cfg(test)]
mod private_config_tests {
    use super::*;

    /// Test save and load token roundtrip.
    ///
    /// Note: This test is ignored by default because the mock credential store in keyring v3.6.2
    /// doesn't persist data between different Entry::new() calls. Each Entry instance has isolated
    /// MockData. This test works correctly with real OS keyring backends.
    ///
    /// To run this test with real keyring: `cargo test test_save_and_load_token -- --ignored`
    #[test]
    #[ignore]
    fn test_save_and_load_token() {
        setup_logger();

        let test_token = "test_token_12345";

        // Save token using mock credential store
        let config = PrivateConfig {
            token: test_token.to_string(),
        };
        config.save().unwrap();

        // Load token back
        let mut loaded_config = PrivateConfig {
            token: String::new(),
        };
        loaded_config.load().unwrap();
        assert_eq!(loaded_config.token, test_token);

        // Cleanup
        loaded_config.delete().unwrap();
    }

    /// Test delete token operation.
    ///
    /// Note: This test is ignored by default because the mock credential store in keyring v3.6.2
    /// doesn't persist data between different Entry::new() calls. Each Entry instance has isolated
    /// MockData. This test works correctly with real OS keyring backends.
    ///
    /// To run this test with real keyring: `cargo test test_delete_token -- --ignored`
    #[test]
    #[ignore]
    fn test_delete_token() {
        setup_logger();

        let test_token = "delete_test_token";

        // Save token using mock credential store
        let config = PrivateConfig {
            token: test_token.to_string(),
        };
        config.save().unwrap();

        // Delete token
        config.delete().unwrap();

        // Try to load - should fail with KeyringLoadFailed
        let mut loaded_config = PrivateConfig {
            token: String::new(),
        };

        let result = loaded_config.load();
        assert!(result.is_err());

        match result {
            Err(CmdError::KeyringLoadFailed) => (),
            _ => panic!("Expected KeyringLoadFailed after deletion"),
        }
    }

    #[test]
    fn test_load_nonexistent_token_fails() {
        setup_logger();

        // Try to load a token that doesn't exist in mock store
        let mut config = PrivateConfig {
            token: String::new(),
        };

        let result = config.load();

        // Should fail with KeyringLoadFailed
        assert!(result.is_err());
        match result {
            Err(CmdError::KeyringLoadFailed) => (),
            _ => panic!("Expected KeyringLoadFailed for non-existent token"),
        }
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    /// Test loading and saving both public and private configs.
    ///
    /// Note: This test is ignored by default because the mock credential store in keyring v3.6.2
    /// doesn't persist data between different Entry::new() calls. Each Entry instance has isolated
    /// MockData. This test works correctly with real OS keyring backends.
    ///
    /// To run this test with real keyring: `cargo test test_config_load_saves_both_configs -- --ignored`
    #[test]
    #[ignore]
    fn test_config_load_saves_both_configs() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a complete config
        let config = Config {
            public_config: PublicConfig {
                endpoint: "http://integration.example.com".to_string(),
                allow_insecure: false,
                path: config_path.clone(),
            },
            private_config: PrivateConfig {
                token: "integration_test_token".to_string(),
            },
        };

        // Save the config (both file and mock keyring)
        config.save().unwrap();

        // Verify public config file was created
        assert!(config_path.exists());

        // Load both configs back
        let loaded = Config::load(Some(&config_path)).unwrap();

        assert_eq!(loaded.public_config.endpoint, "http://integration.example.com");
        assert_eq!(loaded.private_config.token, "integration_test_token");

        // Cleanup
        loaded.delete().unwrap();
    }

    /// Test config roundtrip (save then load preserves data).
    ///
    /// Note: This test is ignored by default because the mock credential store in keyring v3.6.2
    /// doesn't persist data between different Entry::new() calls. Each Entry instance has isolated
    /// MockData. This test works correctly with real OS keyring backends.
    ///
    /// To run this test with real keyring: `cargo test test_config_roundtrip -- --ignored`
    #[test]
    #[ignore]
    fn test_config_roundtrip() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let original_endpoint = "http://roundtrip-integration.example.com:8000";
        let original_token = "roundtrip_token_xyz";

        // Create and save config using mock keyring
        let config = Config {
            public_config: PublicConfig {
                endpoint: original_endpoint.to_string(),
                allow_insecure: false,
                path: config_path.clone(),
            },
            private_config: PrivateConfig {
                token: original_token.to_string(),
            },
        };

        config.save().unwrap();

        // Load it back
        let loaded = Config::load(Some(&config_path)).unwrap();

        // Verify all data is preserved
        assert_eq!(loaded.public_config.endpoint, original_endpoint);
        assert_eq!(loaded.private_config.token, original_token);

        // Cleanup
        loaded.delete().unwrap();
    }

    /// Test that delete removes both config file and keyring entry.
    ///
    /// Note: This test is ignored by default because the mock credential store in keyring v3.6.2
    /// doesn't persist data between different Entry::new() calls. Each Entry instance has isolated
    /// MockData. This test works correctly with real OS keyring backends.
    ///
    /// To run this test with real keyring: `cargo test test_config_delete_removes_both -- --ignored`
    #[test]
    #[ignore]
    fn test_config_delete_removes_both() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create and save config using mock keyring
        let config = Config {
            public_config: PublicConfig {
                endpoint: "http://delete-integration.example.com".to_string(),
                allow_insecure: false,
                path: config_path.clone(),
            },
            private_config: PrivateConfig {
                token: "delete_integration_token".to_string(),
            },
        };

        config.save().unwrap();
        assert!(config_path.exists());

        // Delete config (both file and mock keyring)
        config.delete().unwrap();

        // Verify file is deleted
        assert!(!config_path.exists());

        // Verify token is deleted from mock keyring (loading should fail)
        let mut test_private = PrivateConfig {
            token: String::new(),
        };
        let load_result = test_private.load();
        assert!(load_result.is_err());
    }
}
