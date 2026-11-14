use crate::cmd::config::models::PublicConfig;
use crate::cmd::config::HandleConfig;
use crate::cmd::endpoint::{normalize_endpoint, MIGRATION_SUNSET_VERSION};
use crate::cmd::models::CmdError;
use log::{debug, warn};

impl PublicConfig {
    /// Reduces an old-style endpoint (with API path) to the base URL and rewrites the file.
    ///
    /// Does nothing when the endpoint already is a base URL. Otherwise the reduced
    /// value is used in memory, the file is rewritten and a notice is printed to stderr.
    fn migrate_endpoint(&mut self) {
        let reduced = normalize_endpoint(&self.endpoint);
        if reduced == self.endpoint {
            return;
        }
        let old = std::mem::replace(&mut self.endpoint, reduced);
        let path = self.path.display();
        // Reason: a failed save must not fail the command. Unattended runs (cron,
        // read-only config mounts) keep working with the reduced value and repeat
        // the notice until the file is fixed by hand.
        let outcome = if self.save().is_ok() {
            format!("{path} was updated.")
        } else {
            format!("Could not update {path}; please change the endpoint manually.")
        };
        let notice = format!(
            "Notice: the configured endpoint '{old}' contains an API path.\n\
             The endpoint must be the base URL of your Paperless-ngx instance; using '{}'.\n\
             {outcome}\n\
             Automatic correction will be removed in version {MIGRATION_SUNSET_VERSION}.",
            self.endpoint
        );
        // Reason: the default log level is Error, so `warn!` alone would be invisible.
        eprintln!("{notice}");
        warn!("{notice}");
    }
}

impl HandleConfig for PublicConfig {
    /// Loads the configuration from the YAML file at `self.path`.
    ///
    /// If the file does not exist, a default configuration is written to
    /// `self.path` (creating parent directories) and used instead.
    ///
    /// Note: The `path` field is not overwritten from the file as it represents
    /// runtime state, not stored configuration.
    ///
    /// An endpoint that still contains an API path (configs written before 0.4.0)
    /// is reduced to the base URL, the file is rewritten and a notice is printed.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(CmdError::ConfigFileLoadFailed)` if the file
    /// cannot be read, parsed, or (when missing) created. A failed rewrite of an
    /// old-style endpoint is not an error.
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
        self.migrate_endpoint();
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
