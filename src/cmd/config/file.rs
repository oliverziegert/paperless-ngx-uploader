use crate::cmd::config::models::PublicConfig;
use crate::cmd::config::HandleConfig;
use crate::cmd::models::CmdError;
use log::debug;

impl HandleConfig for PublicConfig {
    /// Loads the configuration from the YAML file at `self.path`.
    ///
    /// If the file does not exist, a default configuration is written to
    /// `self.path` (creating parent directories) and used instead.
    ///
    /// Note: The `path` field is not overwritten from the file as it represents
    /// runtime state, not stored configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(CmdError::ConfigFileLoadFailed)` if the file
    /// cannot be read, parsed, or (when missing) created.
    fn load(&mut self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::load called");
        let load_failed = || CmdError::ConfigFileLoadFailed(self.path.display().to_string());
        let loaded: PublicConfig = match std::fs::read_to_string(&self.path) {
            Ok(yaml) => serde_saphyr::from_str(&yaml).map_err(|_| load_failed())?,
            // Reason: a missing file is not an error (first run);
            // the defaults are persisted so the user has a file to edit.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let defaults = PublicConfig { path: self.path.clone(), ..PublicConfig::default() };
                defaults.save().map_err(|_| load_failed())?;
                defaults
            }
            Err(_) => return Err(load_failed()),
        };
        self.endpoint = loaded.endpoint;
        self.allow_insecure = loaded.allow_insecure;
        // Note: path is not overwritten - it's runtime state, not stored config
        debug!("The configuration file path is: {:#?}", self.path.display());
        debug!("The configuration is:");
        debug!("{self:#?}");
        Ok(())
    }

    /// Saves the configuration as YAML to `self.path`.
    ///
    /// Parent directories are created if they do not exist.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(CmdError::ConfigFileSaveFailed)` otherwise.
    fn save(&self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::save called");
        debug!("Saving Config: {self:#?}");
        let save_failed = || CmdError::ConfigFileSaveFailed(self.path.display().to_string());
        let yaml = serde_saphyr::to_string(self).map_err(|_| save_failed())?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| save_failed())?;
        }
        std::fs::write(&self.path, yaml).map_err(|_| save_failed())
    }

    fn delete(&self) -> Result<(), CmdError> {
        debug!("Config::PublicConfig::delete called");
        // Delete the configuration file.
        std::fs::remove_file(&self.path)
            .map_err(|_| CmdError::ConfigFileDeletionFailed(self.path.display().to_string()))
    }
}
