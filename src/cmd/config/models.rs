use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration with split storage approach.
///
/// This struct coordinates between two storage backends:
/// - Public configuration (endpoint) stored in a YAML file
/// - Private configuration (token) stored in the OS keyring
///
/// This separation ensures sensitive credentials are stored securely
/// in the platform's native credential manager rather than in plaintext files.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// Public configuration (endpoint URL) stored in YAML file
    pub public_config: PublicConfig,

    /// Private configuration (authentication token) stored in OS keyring
    pub private_config: PrivateConfig,
}

/// Public configuration stored in a YAML file.
///
/// Contains non-sensitive configuration that can be safely stored in plaintext.
/// Default location: `~/.config/paperless-ngx-uploader/config.yaml`
#[derive(Serialize, Deserialize, Debug)]
pub struct PublicConfig {
    /// The URL to your Paperless-ngx instance (e.g., <https://paperless.example.com>)
    pub endpoint: String,

    /// Allow insecure HTTP connections to non-localhost endpoints.
    ///
    /// When `false` (default), HTTP connections to remote hosts are rejected
    /// to prevent sending authentication tokens over unencrypted connections.
    /// Set to `true` only if you understand the security implications.
    #[serde(default)]
    pub allow_insecure: bool,

    /// The path to the config file (runtime state, not serialized)
    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
}

/// Private configuration stored in the OS keyring.
///
/// Contains sensitive credentials that should never be stored in plaintext.
/// Uses platform-native credential storage:
/// - macOS: Keychain
/// - Linux: Secret Service
/// - Windows: Credential Manager
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PrivateConfig {
    /// Authentication token for your Paperless-ngx instance
    pub token: String,
}

#[derive(Parser)]
pub enum ConfigCommand {
    Ls,
    Create,
    Set,
    Rm,
}
