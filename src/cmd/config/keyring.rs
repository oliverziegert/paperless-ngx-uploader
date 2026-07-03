use crate::cmd::config::models::PrivateConfig;
use crate::cmd::config::HandleConfig;
use crate::cmd::models::CmdError;
use crate::cmd::APP_NAME;
use keyring_core::Entry;
use log::debug;

/// The credential name used to identify the authentication token in the OS keyring.
///
/// This constant is combined with [`APP_NAME`] to create a unique keyring entry
/// for storing the Paperless-ngx authentication token. The keyring entry is accessed
/// using the format `(APP_NAME, KEYRING_TOKEN_NAME)` through the platform's native
/// credential storage system.
const KEYRING_TOKEN_NAME: &str = "token";

/// Installs the platform-native credential store as the keyring default.
///
/// Since keyring v4, the platform backends live in separate crates and the
/// application must install a credential store before creating entries. This
/// function selects the appropriate store for the current platform:
///
/// - **Linux/BSD**: Secret Service API (typically GNOME Keyring or `KWallet`)
/// - **macOS/iOS**: Keychain
/// - **Windows**: Credential Manager
///
/// # Reason: If a default store is already installed (e.g. the mock store in
/// unit tests), it is left untouched so tests never hit the real keyring.
///
/// # Errors
///
/// Returns `CmdError::KeyringNotAvailable` if the platform store cannot be
/// initialized.
fn ensure_default_store() -> Result<(), CmdError> {
    if keyring_core::get_default_store().is_some() {
        return Ok(());
    }

    debug!("Installing platform-native credential store");

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let store = apple_native_keyring_store::keychain::Store::new()
        .map_err(|_| CmdError::KeyringNotAvailable)?;

    #[cfg(target_os = "windows")]
    let store =
        windows_native_keyring_store::Store::new().map_err(|_| CmdError::KeyringNotAvailable)?;

    #[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
    let store = zbus_secret_service_keyring_store::Store::new()
        .map_err(|_| CmdError::KeyringNotAvailable)?;

    keyring_core::set_default_store(store);
    Ok(())
}

/// Creates and initializes a keyring entry for secure token storage.
///
/// This function creates a keyring entry using the platform's native credential
/// storage system. The entry is identified by the application name and a fixed
/// credential name for the authentication token.
///
/// # Platform Behavior
///
/// - **Linux**: Uses the Secret Service API (typically GNOME Keyring or `KWallet`)
/// - **macOS**: Uses the Keychain
/// - **Windows**: Uses the Credential Manager
///
/// # Returns
///
/// A keyring `Entry` configured for token storage.
///
/// # Errors
///
/// Returns `CmdError::KeyringNotAvailable` if the keyring cannot be initialized
/// on the current platform.
fn setup_keyring() -> Result<Entry, CmdError> {
    debug!("Config::PrivateConfig::setup_keyring called");
    ensure_default_store()?;
    Entry::new(APP_NAME, KEYRING_TOKEN_NAME).map_err(|_| CmdError::KeyringNotAvailable)
}

impl HandleConfig for PrivateConfig {
    fn load(&mut self) -> Result<(), CmdError> {
        debug!("Config::PrivateConfig::load called");
        let entry = setup_keyring()?;
        debug!("Loading token from keyring");
        self.token = entry.get_password().map_err(|_| CmdError::KeyringLoadFailed)?;
        Ok(())
    }

    fn save(&self) -> Result<(), CmdError> {
        debug!("Config::PrivateConfig::save called");
        let entry = setup_keyring()?;
        debug!("Saving token to keyring");
        entry
            .set_password(self.token.as_str())
            .map_err(|_| CmdError::KeyringSaveFailed)?;
        Ok(())
    }

    fn delete(&self) -> Result<(), CmdError> {
        debug!("Config::PrivateConfig::delete called");
        let entry = setup_keyring()?;
        debug!("Deleting token from keyring");
        entry.delete_credential().map_err(|_| CmdError::KeyringDeletionFailed)?;
        Ok(())
    }
}

#[cfg(test)]
mod real_keyring_smoke {
    /// Smoke test for the real platform credential store.
    ///
    /// Verifies that `ensure_default_store` installs a working platform store
    /// and that a set/get/delete roundtrip succeeds against the real OS
    /// keyring. Uses a throwaway service name so the application's real token
    /// is never touched.
    ///
    /// Ignored by default because it requires a real OS keyring (unavailable
    /// in most CI environments) and must not run alongside tests that install
    /// the mock store. Run it alone with:
    /// `cargo test smoke_real_store_roundtrip -- --ignored`
    #[test]
    #[ignore]
    fn smoke_real_store_roundtrip() {
        super::ensure_default_store().unwrap();
        let entry = keyring_core::Entry::new("paperless-ngx-uploader-smoke-test", "token").unwrap();
        entry.set_password("smoke").unwrap();
        assert_eq!(entry.get_password().unwrap(), "smoke");
        entry.delete_credential().unwrap();
    }
}
