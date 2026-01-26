mod file;
pub mod keyring;
mod models;

#[cfg(test)]
mod tests;

use crate::cmd::config::models::PublicConfig;
use crate::cmd::models::CmdError;
use crate::cmd::APP_NAME;
use log::debug;
use std::path::PathBuf;

// Re-export Config publicly so it can be used from main.rs
pub use crate::cmd::config::models::Config;

const CONFIG_FILE_NAME: &str = "config.yaml";

impl Default for PublicConfig {
    fn default() -> Self {
        // Don't create directories in Default - error handling is done in Config::load()
        // This allows Default to be infallible as required by the trait
        Self {
            endpoint: "http://localhost:8000".into(),
            allow_insecure: false,
            path: PathBuf::new(), // Empty path - will be set in Config::load()
        }
    }
}

/// Trait for handling configuration operations across different storage backends.
///
/// This trait provides a common interface for loading, saving, and deleting
/// configuration data, allowing both file-based and keyring-based storage
/// to be managed uniformly.
pub trait HandleConfig {
    /// Loads configuration from the storage backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded.
    fn load(&mut self) -> Result<(), CmdError>;

    /// Saves configuration to the storage backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    fn save(&self) -> Result<(), CmdError>;

    /// Deletes configuration from the storage backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be deleted.
    fn delete(&self) -> Result<(), CmdError>;
}

pub fn get_or_create_config_dir() -> Result<PathBuf, CmdError> {
    let config_dir = dirs::config_dir().ok_or(CmdError::UnsupportedPlatform)?;

    let config_dir = config_dir.join(APP_NAME);

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|_| CmdError::ConfigDirCreationFailed)?;
    }

    Ok(config_dir)
}

impl Config {
    /// Loads configuration from both file and keyring.
    ///
    /// This method coordinates loading from both storage backends:
    /// - Public configuration (endpoint) from YAML file
    /// - Private configuration (token) from OS keyring
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to the configuration file. If None, uses the default path.
    ///
    /// # Returns
    ///
    /// A fully populated `Config` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if either the file or keyring loading fails.
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self, CmdError> {
        debug!("Config::load called");

        // Start with default config
        let mut config = Config::default();

        // Set the config file path (use provided path or default)
        if let Some(path) = config_path {
            config.public_config.path.clone_from(path);
        } else {
            // No custom path provided - use default config directory
            let config_dir = get_or_create_config_dir()?;
            config.public_config.path = config_dir.join(CONFIG_FILE_NAME);
        }

        debug!("Loading public config from: {}", config.public_config.path.display());

        // Load public config from file
        config.public_config.load()?;

        debug!("Loading private config from keyring");

        // Load private config from keyring
        config.private_config.load()?;

        debug!("Config loaded successfully");
        Ok(config)
    }

    /// Saves configuration to both file and keyring.
    ///
    /// This method coordinates saving to both storage backends:
    /// - Public configuration (endpoint) to YAML file
    /// - Private configuration (token) to OS keyring
    ///
    /// # Returns
    ///
    /// `Ok(())` if both saves succeed.
    ///
    /// # Errors
    ///
    /// Returns an error if either the file or keyring saving fails.
    pub fn save(&self) -> Result<(), CmdError> {
        debug!("Config::save called");

        // Save public config to file
        self.public_config.save()?;
        debug!("Public config saved successfully");

        // Save private config to keyring
        self.private_config.save()?;
        debug!("Private config saved successfully");

        Ok(())
    }

    /// Deletes configuration from both file and keyring.
    ///
    /// This method removes all stored configuration:
    /// - Deletes the YAML configuration file
    /// - Removes the token from the OS keyring
    ///
    /// # Returns
    ///
    /// `Ok(())` if both deletions succeed.
    ///
    /// # Errors
    ///
    /// Returns an error if either deletion fails.
    pub fn delete(&self) -> Result<(), CmdError> {
        debug!("Config::delete called");

        // Delete public config file
        self.public_config.delete()?;
        debug!("Public config deleted successfully");

        // Delete private config from keyring
        self.private_config.delete()?;
        debug!("Private config deleted successfully");

        Ok(())
    }
}
