use super::*;
use crate::cmd::config::models::{Config, PrivateConfig, PublicConfig};
use crate::cmd::models::CmdError;
use log::LevelFilter;
use tempfile::TempDir;

/// Serializes tests that touch the keyring token entry.
///
/// All keyring tests operate on the same `(APP_NAME, "token")` entry in the
/// shared mock credential store, so running them in parallel would let one
/// test observe (or delete) another test's token. Every test that saves,
/// loads, or deletes the token must hold this lock for its whole duration.
static KEYRING_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Acquires [`KEYRING_LOCK`], recovering from poisoning.
///
/// # Reason: A failed assertion in one keyring test poisons the mutex; the
/// remaining tests should still run rather than all panicking on the lock.
fn lock_keyring() -> std::sync::MutexGuard<'static, ()> {
    KEYRING_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Sets up the test logger to capture log output during tests.
/// Also configures the keyring to use the mock credential store.
fn setup_logger() {
    let _ = env_logger::builder().filter_level(LevelFilter::Debug).is_test(true).try_init();

    // Set up mock credential store for testing (keyring-core ships a built-in
    // mock store since keyring v4). Installed exactly once so parallel tests
    // share one store and production code never replaces it (see
    // ensure_default_store in keyring.rs).
    static INIT_MOCK_STORE: std::sync::Once = std::sync::Once::new();
    INIT_MOCK_STORE.call_once(|| {
        // Reason: Store::new() on the mock store is infallible in practice;
        // if it ever fails the tests cannot run meaningfully anyway.
        #[allow(clippy::expect_used)]
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("failed to create mock credential store"),
        );
    });
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

    #[test]
    fn test_allow_insecure_true_roundtrip() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create and save config with allow_insecure set to true
        let config = PublicConfig {
            endpoint: "http://insecure-test.example.com".to_string(),
            allow_insecure: true,
            path: config_path.clone(),
        };
        config.save().unwrap();

        // Verify the file was created
        assert!(config_path.exists());

        // Load it back
        let mut loaded_config = PublicConfig {
            endpoint: String::new(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        loaded_config.load().unwrap();

        // Verify allow_insecure is preserved as true
        assert_eq!(loaded_config.endpoint, "http://insecure-test.example.com");
        assert!(loaded_config.allow_insecure, "allow_insecure should be true after loading");
    }

    #[test]
    fn test_allow_insecure_false_roundtrip() {
        setup_logger();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create and save config with allow_insecure set to false
        let config = PublicConfig {
            endpoint: "https://secure-test.example.com".to_string(),
            allow_insecure: false,
            path: config_path.clone(),
        };
        config.save().unwrap();

        // Verify the file was created
        assert!(config_path.exists());

        // Load it back with different initial value to ensure it's overwritten
        let mut loaded_config = PublicConfig {
            endpoint: String::new(),
            allow_insecure: true,
            path: config_path.clone(),
        };
        loaded_config.load().unwrap();

        // Verify allow_insecure is preserved as false
        assert_eq!(loaded_config.endpoint, "https://secure-test.example.com");
        assert!(!loaded_config.allow_insecure, "allow_insecure should be false after loading");
    }
}

#[cfg(test)]
mod private_config_tests {
    use super::*;

    /// Test save and load token roundtrip.
    #[test]
    fn test_save_and_load_token() {
        setup_logger();
        let _guard = lock_keyring();

        let test_token = "test_token_12345";

        // Save token using mock credential store
        let config = PrivateConfig { token: test_token.to_string() };
        config.save().unwrap();

        // Load token back
        let mut loaded_config = PrivateConfig { token: String::new() };
        loaded_config.load().unwrap();
        assert_eq!(loaded_config.token, test_token);

        // Cleanup
        loaded_config.delete().unwrap();
    }

    /// Test delete token operation.
    #[test]
    fn test_delete_token() {
        setup_logger();
        let _guard = lock_keyring();

        let test_token = "delete_test_token";

        // Save token using mock credential store
        let config = PrivateConfig { token: test_token.to_string() };
        config.save().unwrap();

        // Delete token
        config.delete().unwrap();

        // Try to load - should fail with KeyringLoadFailed
        let mut loaded_config = PrivateConfig { token: String::new() };

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
        let _guard = lock_keyring();

        // Try to load a token that doesn't exist in mock store
        let mut config = PrivateConfig { token: String::new() };

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
    #[test]
    fn test_config_load_saves_both_configs() {
        setup_logger();
        let _guard = lock_keyring();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a complete config
        let config = Config {
            public_config: PublicConfig {
                endpoint: "http://integration.example.com".to_string(),
                allow_insecure: false,
                path: config_path.clone(),
            },
            private_config: PrivateConfig { token: "integration_test_token".to_string() },
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
    #[test]
    fn test_config_roundtrip() {
        setup_logger();
        let _guard = lock_keyring();

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
            private_config: PrivateConfig { token: original_token.to_string() },
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
    #[test]
    fn test_config_delete_removes_both() {
        setup_logger();
        let _guard = lock_keyring();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create and save config using mock keyring
        let config = Config {
            public_config: PublicConfig {
                endpoint: "http://delete-integration.example.com".to_string(),
                allow_insecure: false,
                path: config_path.clone(),
            },
            private_config: PrivateConfig { token: "delete_integration_token".to_string() },
        };

        config.save().unwrap();
        assert!(config_path.exists());

        // Delete config (both file and mock keyring)
        config.delete().unwrap();

        // Verify file is deleted
        assert!(!config_path.exists());

        // Verify token is deleted from mock keyring (loading should fail)
        let mut test_private = PrivateConfig { token: String::new() };
        let load_result = test_private.load();
        assert!(load_result.is_err());
    }

    /// Test config with allow_insecure field set to true.
    #[test]
    fn test_config_with_allow_insecure() {
        setup_logger();
        let _guard = lock_keyring();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a config with allow_insecure set to true
        let config = Config {
            public_config: PublicConfig {
                endpoint: "http://insecure-integration.example.com".to_string(),
                allow_insecure: true,
                path: config_path.clone(),
            },
            private_config: PrivateConfig { token: "insecure_integration_token".to_string() },
        };

        // Save the config (both file and mock keyring)
        config.save().unwrap();

        // Verify public config file was created
        assert!(config_path.exists());

        // Load config back
        let loaded = Config::load(Some(&config_path)).unwrap();

        // Verify all fields including allow_insecure
        assert_eq!(loaded.public_config.endpoint, "http://insecure-integration.example.com");
        assert!(
            loaded.public_config.allow_insecure,
            "allow_insecure should be true after loading"
        );
        assert_eq!(loaded.private_config.token, "insecure_integration_token");

        // Cleanup
        loaded.delete().unwrap();
    }
}
