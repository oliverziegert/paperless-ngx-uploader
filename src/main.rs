mod cmd;

use cmd::config::Config;
use cmd::input::{get_endpoint_by_prompt, get_token_by_prompt};
use cmd::logger::setup_logging;
use cmd::url_validator::{validate_endpoint_format, validate_endpoint_security};

use clap::{Parser, Subcommand};
use log::{debug, error, info, warn};
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Sets a custom config file path (defaults to platform-specific config directory)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the Config file
    Init {
        /// The url to your Paperless-ngx instance
        #[arg(short, long, value_name = "ENDPOINT")]
        endpoint: Option<String>,

        /// Allow insecure HTTP connections (not recommended for production)
        #[arg(long)]
        allow_insecure: bool,
    },

    /// Uploads a file or a folder to your Paperless-ngx instance
    Upload {
        /// File to upload
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Folder to upload
        #[arg(long, value_name = "FOLDER")]
        folder: Option<PathBuf>,

        /// Recursively scan subfolders
        #[arg(long)]
        recursive: bool,

        /// Filename filter to upload
        #[arg(long, value_name = "FILTER", default_value = ".*")]
        filter: String,

        /// Archive the uploaded files
        #[arg(long)]
        archive: bool,

        /// Period to archive the uploaded files in days
        #[arg(long, default_value_t = 31)]
        period: usize,

        /// Delete the uploaded files
        #[arg(long)]
        delete: bool,

        /// Simulate upload without actually sending files
        #[arg(long)]
        dry_run: bool,

        /// Allow insecure HTTP connections (not recommended for production)
        #[arg(long)]
        allow_insecure: bool,
    },

    /// Checks connectivity and authentication with Paperless-ngx
    Status {
        /// Allow insecure HTTP connections (not recommended for production)
        #[arg(long)]
        allow_insecure: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Set up logging
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Init { endpoint, allow_insecure } => {
            // For init, start with a fresh default config (don't try to load existing)
            let mut cfg = Config::default();

            // If a custom config path was provided, set it
            if let Some(path) = cli.config {
                cfg.public_config.path = path;
            } else {
                // No custom path provided - use default config directory
                let config_dir = match cmd::config::get_or_create_config_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        error!("Error creating config directory: {e}");
                        return Err(e.into());
                    }
                };
                cfg.public_config.path = config_dir.join("config.yaml");
            }

            init(endpoint, allow_insecure, &mut cfg)
        }
        Commands::Upload {
            file,
            folder,
            recursive,
            filter,
            archive,
            period,
            delete,
            dry_run,
            allow_insecure,
        } => {
            // For upload, load existing config (must exist)
            let cfg = match Config::load(cli.config.as_ref()) {
                Ok(config) => config,
                Err(e) => {
                    error!("Error loading config: {e}");
                    return Err(e.into());
                }
            };

            upload(
                file,
                folder,
                recursive,
                filter,
                archive,
                period,
                delete,
                dry_run,
                allow_insecure,
                cfg,
            )
            .await
        }
        Commands::Status { allow_insecure } => {
            // For status, load existing config (must exist)
            let cfg = match Config::load(cli.config.as_ref()) {
                Ok(config) => config,
                Err(e) => {
                    error!("Error loading config: {e}");
                    return Err(e.into());
                }
            };

            status(allow_insecure, cfg).await
        }
    }
}

/// Initializes the configuration by prompting for endpoint and token.
///
/// # Errors
///
/// Returns an error if:
/// - Endpoint prompt fails
/// - Endpoint format validation fails
/// - Endpoint security validation fails (unless --allow-insecure is specified)
/// - Token prompt fails
/// - Configuration save fails
fn init(
    endpoint: Option<String>,
    allow_insecure: bool,
    cfg: &mut Config,
) -> Result<(), Box<dyn Error>> {
    debug!("init called: endpoint: {endpoint:#?}, allow_insecure: {allow_insecure}");
    if let Some(endpoint) = endpoint {
        cfg.public_config.endpoint = endpoint;
    } else {
        match get_endpoint_by_prompt() {
            Ok(endpoint) => {
                cfg.public_config.endpoint = endpoint;
            }
            Err(e) => {
                error!("Error getting endpoint: {e}");
                return Err(e);
            }
        }
    }

    // Validate endpoint format
    if let Err(e) = validate_endpoint_format(&cfg.public_config.endpoint) {
        error!("{e}");
        return Err(e.into());
    }

    // Validate endpoint security
    if let Err(e) = validate_endpoint_security(&cfg.public_config.endpoint) {
        if allow_insecure {
            warn!("⚠️  Security Warning: {e}");
            warn!("Proceeding with insecure connection as --allow-insecure was specified.");
            cfg.public_config.allow_insecure = true;
        } else {
            error!("{e}");
            return Err(e.into());
        }
    }

    match get_token_by_prompt() {
        Ok(token) => {
            cfg.private_config.token = token;
        }
        Err(e) => {
            error!("Error getting token: {e}");
            return Err(e);
        }
    }

    if let Err(e) = cfg.save() {
        error!("Error saving config: {e}");
        Err("Error saving config".into())
    } else {
        info!("Config saved successfully");
        Ok(())
    }
}

/// Uploads files to Paperless-ngx with optional archival and cleanup.
///
/// # Errors
///
/// Returns an error if:
/// - Endpoint security validation fails and insecure connections are not allowed
/// - HTTP client creation fails
/// - File upload fails
#[allow(clippy::too_many_arguments)]
#[allow(clippy::fn_params_excessive_bools)]
pub async fn upload(
    file: Option<PathBuf>,
    folder: Option<PathBuf>,
    recursive: bool,
    filter: String,
    archive: bool,
    period: usize,
    delete: bool,
    dry_run: bool,
    allow_insecure: bool,
    cfg: Config,
) -> Result<(), Box<dyn Error>> {
    debug!("Called: upload");

    // Validate endpoint security
    // CLI --allow-insecure overrides config.allow_insecure
    if let Err(e) = validate_endpoint_security(&cfg.public_config.endpoint) {
        if allow_insecure || cfg.public_config.allow_insecure {
            warn!("⚠️  Security Warning: {e}");
            warn!("Proceeding with insecure connection as allowed by config or --allow-insecure flag.");
        } else {
            error!("{e}");
            return Err(e.into());
        }
    }

    let client = match cmd::client::Client::new(cfg) {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating client: {e}");
            return Err(e.into());
        }
    };

    client
        .upload(cmd::client::UploadOptions {
            file,
            folder,
            recursive,
            filter,
            archive,
            period,
            delete,
            dry_run,
        })
        .await
}

/// Checks connectivity and authentication with Paperless-ngx.
///
/// # Errors
///
/// Returns an error if:
/// - Endpoint security validation fails and insecure connections are not allowed
/// - HTTP client creation fails
/// - Status check fails (network issues, authentication failure, etc.)
pub async fn status(allow_insecure: bool, cfg: Config) -> Result<(), Box<dyn Error>> {
    debug!("Called: status");

    // Validate endpoint security
    // CLI --allow-insecure overrides config.allow_insecure
    if let Err(e) = validate_endpoint_security(&cfg.public_config.endpoint) {
        if allow_insecure || cfg.public_config.allow_insecure {
            warn!("⚠️  Security Warning: {e}");
            warn!("Proceeding with insecure connection as allowed by config or --allow-insecure flag.");
        } else {
            error!("{e}");
            return Err(e.into());
        }
    }

    let client = match cmd::client::Client::new(cfg) {
        Ok(client) => client,
        Err(e) => {
            error!("Error creating client: {e}");
            return Err(e.into());
        }
    };

    client.check_status().await
}
