use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmdError {
    #[error("Failed to load configuration file from {0}")]
    ConfigFileLoadFailed(String),

    #[error("Failed to save configuration file to {0}")]
    ConfigFileSaveFailed(String),

    #[error("Failed to delete configuration file at {0}")]
    ConfigFileDeletionFailed(String),

    #[error(
        "Failed to load token from OS keyring. Please run 'init' to configure authentication."
    )]
    KeyringLoadFailed,

    #[error(
        "Failed to save token to OS keyring. The keyring may not be available on this system."
    )]
    KeyringSaveFailed,

    #[error("Failed to delete token from OS keyring")]
    KeyringDeletionFailed,

    #[error("OS keyring is not available on this platform")]
    KeyringNotAvailable,

    #[error("No authentication token configured. Please run 'init' to configure authentication.")]
    TokenNotConfigured,

    #[error("No endpoint configured. Please run 'init' to configure the Paperless-ngx endpoint.")]
    EndpointNotConfigured,

    #[error("Insecure HTTP connection to {0}. Use HTTPS for secure connections, or pass --allow-insecure to allow HTTP (not recommended for remote servers).")]
    InsecureConnection(String),

    #[error("Failed to create HTTP client")]
    ClientCreationFailed,

    #[error("Failed to create config directory. Please check directory permissions.")]
    ConfigDirCreationFailed,

    #[error(
        "Unsupported platform (no config dir found). Only Linux, MacOS and Windows are supported."
    )]
    UnsupportedPlatform,

    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
}
