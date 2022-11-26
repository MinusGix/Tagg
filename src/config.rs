use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::util::expand_path;

// We don't allow modifying the storage location with an env var, since you could cause issues by having
// the state have files that the storage doesn't have.
// Just create a custom config file with a storage path

/// The name of the environment variable that can be used to override where we store the config
pub const CONFIG_ENV_VAR: &str = "TAGG_CONFIG";
/// The name of the environment variable that can be used to override where we store the state
pub const STATE_ENV_VAR: &str = "TAGG_STATE";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The location where the program should store the files.  
    /// This can be overwritten with the `TAGG_STORAGE` environment variable.  
    /// (So, if you want separate tagging 'repositories' then you could
    /// create bash aliases that just swap the `TAGG_STORAGE` variable)
    ///
    /// This should be a folder.
    pub storage_path: String,

    /// The location where the program should store the state.  
    /// This can be overwritten with the `TAGG_STATE` environment variable.  
    #[serde(default)]
    pub state_path: Option<String>,

    /// Whether or not adding a file should hash it to alert you if it gets changed.  
    /// This can help avoid some accidental problems where you have an old `tagg add` around.  
    pub hash_added_files: bool,

    /// How long the registration of files should be limited by. After this amount of time
    /// after the last file added to the registration-area, the area is cleared (the files being
    /// unchanged) to avoid accidentally including them if you forget.  
    pub registration_delay_limit: u64,
}
impl Config {
    pub fn config_path() -> PathBuf {
        if let Ok(config_path) = std::env::var(CONFIG_ENV_VAR) {
            expand_path(config_path)
        } else {
            // TODO: Get the path based on XDG and also for wherever windows and mac should store it
            todo!("Declare config env var")
        }
    }

    pub fn load_from(path: &Path) -> eyre::Result<Config> {
        let file = std::fs::read_to_string(path)?;
        let config = toml::from_str(&file)?;

        Ok(config)
    }

    pub fn state_path(&self, config_path: &Path) -> eyre::Result<PathBuf> {
        if let Ok(state_path) = std::env::var(STATE_ENV_VAR) {
            Ok(expand_path(state_path))
        } else if let Some(state_path) = &self.state_path {
            let state_path = expand_path(state_path);
            if let Some(config_parent) = config_path.parent() {
                // Add state_path onto the end. If state path is absolute
                // then it will just replace, otherwise it will be appended onto it.
                let mut path = config_parent.to_path_buf();
                path.push(state_path);
                Ok(path)
            } else if state_path.is_absolute() {
                // We couldn't get the config parent, which is fine if the state path is absolute
                Ok(state_path)
            } else {
                Err(eyre::eyre!(
                    "Invalid config-path parent-folder when state-path is relative"
                ))
            }
        } else {
            // TODO: Get the path based on spec
            todo!("Declare state env var")
        }
    }

    pub fn storage_path(&self, config_path: &Path) -> eyre::Result<PathBuf> {
        let storage_path = &self.storage_path;
        let storage_path = expand_path(storage_path);
        if let Some(config_parent) = config_path.parent() {
            let mut path = config_parent.to_path_buf();
            path.push(storage_path);
            Ok(path)
        } else if storage_path.is_absolute() {
            Ok(storage_path)
        } else {
            Err(eyre::eyre!(
                "Invaldi config-path parent-folder when storage-path is relative"
            ))
        }
    }
}
