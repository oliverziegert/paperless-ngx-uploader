use crate::cmd::config::models::PrivateConfig;
use crate::cmd::config::HandleConfig;
use crate::cmd::models::CmdError;
use crate::cmd::APP_NAME;
use keyring::Entry;
use log::debug;

/// The credential name used to identify the authentication token in the OS keyring.
///
/// This constant is combined with [`APP_NAME`] to create a unique keyring entry
/// for storing the Paperless-ngx authentication token. The keyring entry is accessed
/// using the format `(APP_NAME, KEYRING_TOKEN_NAME)` through the platform's native
/// credential storage system.
const KEYRING_TOKEN_NAME: &str = "token";

fn setup_keyring() -> Result<Entry, CmdError> {
    debug!("Config::PrivateConfig::setup_keyring called");
    Entry::new(APP_NAME, KEYRING_TOKEN_NAME)
        .map_err(|_| CmdError::KeyringNotAvailable)
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
        entry.set_password(self.token.as_str()).map_err(|_| CmdError::KeyringSaveFailed)?;
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
