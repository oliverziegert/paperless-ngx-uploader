use log::debug;
use crate::cmd::config::HandleConfig;
use crate::cmd::config::models::PublicConfig;
use crate::cmd::models::CmdError;

impl HandleConfig for PublicConfig {
    /// Loads a configuration from a file at the given path.
    ///
    /// The configuration is loaded from the file using `confy::load_path`.
    /// If the configuration file exists and is valid, this function returns
    /// `Ok(())`.  Otherwise, it returns `Err(CmdError)`.
    ///
    /// Note: The `path` field is not overwritten from the file as it represents
    /// runtime state, not stored configuration.
    fn load(&mut self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::load called");
        let loaded: PublicConfig = confy::load_path(&self.path)
            .map_err(|_| CmdError::ConfigFileLoadFailed(self.path.display().to_string()))?;
        self.endpoint = loaded.endpoint;
        self.allow_insecure = loaded.allow_insecure;
        // Note: path is not overwritten - it's runtime state, not stored config
        debug!("The configuration file path is: {:#?}", &self.path.display());
        debug!("The configuration is:");
        debug!("{:#?}", self);
        Ok(())
    }

    fn save(&self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::save called");
        debug!("Saving Config: {:#?}", self);
        confy::store_path(&self.path, &self)
            .map_err(|_| CmdError::ConfigFileSaveFailed(self.path.display().to_string()))
    }

    fn delete(&self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::delete called");
        // Delete the configuration file.
        std::fs::remove_file(&self.path)
            .map_err(|_| CmdError::ConfigFileDeletionFailed(self.path.display().to_string()))
    }
}
