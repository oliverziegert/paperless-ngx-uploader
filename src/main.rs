mod cmd;

use cmd::logger::setup_logging;
use cmd::config::*;
use cmd::input::*;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::error::Error;
use log::{debug, error, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Sets a custom Config file
    #[arg(short, long, value_name = "CONFIG FILE", default_value = "./paperless-ngx-uploader.yaml")]
    config: PathBuf,

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

        /// The token for your Paperless-ngx instance
        #[arg(short, long, value_name = "TOKEN")]
        token: Option<String>,
    },

    /// Uploads a file or a folder to your Paperless-ngx instance
    Upload {
        /// File to upload
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Folder to upload
        #[arg(long, value_name = "FOLDER")]
        folder: Option<PathBuf>,

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
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Set up logging
    setup_logging(cli.verbose);

    // Load Config file
    let mut cfg= match Config::load(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Error loading config: {}", e);
            return Err(e.into());
        },
    };

    match cli.command {
        Commands::Init {endpoint, token} => init(endpoint, token, &mut cfg),
        Commands::Upload {
            file,
            folder,
            filter,
            archive,
            period,
            delete,
        } => upload(file, folder, filter, archive, period, delete, cfg),
    }
}

fn init(endpoint: Option<String>, token: Option<String>, cfg: &mut Config) -> Result<(), Box<dyn Error>> {
    debug!("init called: endpoint: {:#?} token: {:#?}", endpoint, token);
    if let Some(endpoint) = endpoint {
        cfg.endpoint = endpoint;
    } else {
        match get_endpoint_by_prompt() {
            Ok(endpoint) => {
                cfg.endpoint = endpoint;
            }
            Err(e) => {
                error!("Error getting endpoint: {}", e);
                return Err(e.into());
            }
        }
    }

    if let Some(token) = token {
        cfg.token = token;
    } else {
        match get_token_by_prompt() {
            Ok(token) => {
                cfg.token = token;
            }
            Err(e) => {
                error!("Error getting token: {}", e);
                return Err(e.into());
            }
        }
    }

    match Config::save(cfg) {
        Err(e) => {
            error!("Error saving config: {}", e);
            Err("Error saving config".into())
        }
        Ok(_) => {
            info!("Config saved successfully");
            Ok(())
        }
    }
}

pub fn upload(file: Option<PathBuf>,
              folder: Option<PathBuf>,
              filter: String,
              archive: bool,
              period: usize,
              delete: bool,
              cfg: Config) -> Result<(), Box<dyn Error>> {
    debug!("Called: upload");
    let client = cmd::client::Client::new(cfg);
    client.upload(file, folder, filter, archive, period, delete)
}

