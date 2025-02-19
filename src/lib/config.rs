use log::debug;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The url to your Paperless-ngx instance
    pub endpoint: String,

    /// The token for your Paperless-ngx instance
    pub token: String,

    /// The path to your config file
    #[serde(skip_serializing, skip_deserializing)]
    path: PathBuf,
}

pub trait Load {
    fn load(path: &PathBuf) -> Result<Config, confy::ConfyError>;
}

pub trait Save {
    fn save(config: &Config) -> Result<(), confy::ConfyError>;
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8000".into(),
            token: "".into(),
            path: PathBuf::new(),
        }
    }
}

impl Load for Config {
    /// Loads a configuration from a file at the given path.
    ///
    /// The configuration is loaded from the file using `confy::load_path`.
    /// If the configuration file exists and is valid, this function returns
    /// `Ok(Config)`.  Otherwise, it returns `Err(confy::ConfyError)`.
    ///
    /// After loading the configuration, the `path` field of the `Config` is set
    /// to the given path.
    fn load(path: &PathBuf) -> Result<Config, confy::ConfyError> {
        debug!("Config::load called");
        let mut cfg: Config = confy::load_path(path)?;
        cfg.path = path.clone();
        debug!("The configuration file path is: {:#?}", path.display());
        debug!("The configuration is:");
        debug!("{:#?}", cfg);
        Ok(cfg)
    }
}
impl Save for Config {
    fn save(config: &Config) -> Result<(), confy::ConfyError> {
        debug!("Config::save called");
        debug!("Saving Config: {:#?}", config);
        confy::store_path(&config.path, config)
    }
}