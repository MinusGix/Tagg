//! Part of the general design of `tagg` is that it tries to make it easy to do common actions.  
//! However, it also tries to encourage you to tag your files appropriately. What's the use of a
//! tagging system if you just add a bunch of files and never tag them?  
//!
//! That's why the default way for tagg to operate is by a somewhat-Git-like:  
//! - add files  
//! - modify file tags / comments / etc  
//! - register files  
//!   
//! This means it will take more commands to add a file, but it also lets you more easily add many at a time.  
//! And, it encourages you to set your tags before you even register them.  
//!   
//! However, it certainly can be useful to add files quickly without bothering to tag them. So, there's a short
//! `addq` command to simply immediately register the files. This takes a few more parameters so that you can still specify
//! tags if you want.

use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml::value::Datetime;

use crate::storage::Storage;

/// The currently active state.  
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct State {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registration_area: Vec<AddedFile>,

    /// The time that the registration was last modified.  
    /// If this was more than [`Config::registration_delay_limit`] then we dump the registration
    /// state. This is to avoid accidentally leaving a file in the registration without adding it,
    /// and thus avoids accidentally including it the next time you add things.  
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_registration: Option<Datetime>,

    pub storage: Storage,
}
impl State {
    /// Load the state from the state file, creating it if it doesn't already exist.  
    pub fn load_from(state_path: &Path) -> eyre::Result<State> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(state_path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let state = toml::from_str(&buf)?;
        Ok(state)
    }

    pub fn save_to(&self, path: &Path) -> eyre::Result<()> {
        let data = toml::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

/// The name of the main 'comment' field
pub const COMMENT_MAIN: &str = "comment";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This is data for a file that has yet to be registered.
pub struct AddedFile {
    /// The absolute path to the file location
    pub path: PathBuf,

    /// The hash of the file when it was added.  
    /// We aren't strict about this, but it does let us alert the user that they seem to
    /// be adding a file that has changed.  
    /// This can be `None` if it is disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<u64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// A comment about the file. These can be arbitrarily named, to allow
    /// custom information.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub comment: HashMap<String, String>,
}
impl AddedFile {
    /// Check if the file still exists
    pub fn exists(&self) -> eyre::Result<bool> {
        todo!()
    }

    /// Check if the file still exists (returns `Some(_)`) and
    /// that the hash is correct. If the hash is not stored, then it assumes
    /// that it is fine.  
    ///  
    /// `Some(true)` if file exists and (hash correct || hash is none)
    /// `Some(false)` if file exists and hash is incorrect
    /// `None` if file does not exist  
    pub fn exists_hash_correct(&self) -> eyre::Result<Option<bool>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        state::{AddedFile, State},
        storage::Storage,
    };

    // This test ensures that we can properly serialize the structures.
    // For some forsaken reason, the `toml` crate seems to care about the ordering of your structure's fields!
    // And will error if the ordering is ''bad''
    #[test]
    fn test_serialize() {
        let storage = Storage { files: Vec::new() };

        let storage_text = toml::to_string(&storage).unwrap();
        println!("Storage: {}", storage_text);

        let file = AddedFile {
            path: PathBuf::from("toaster.txt"),
            hash: None,
            comment: HashMap::new(),
            tags: vec![],
        };
        let file_text = toml::to_string(&file).unwrap();
        println!("File: {}", file_text);

        let state = State {
            registration_area: vec![file],
            last_registration: None,
            storage,
        };

        let state = toml::to_string(&state).unwrap();
        println!("State: {}", state);
    }
}
