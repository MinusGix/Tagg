use std::path::{Path, PathBuf};

use crate::{config::Config, state::State, storage::FileInfo};

pub struct Tagg {
    pub config_path: PathBuf,
    pub state_path: PathBuf,

    pub config: Config,
    pub state: State,

    pub verbose: bool,
}
impl Tagg {
    pub fn save_state(&self) -> eyre::Result<()> {
        if self.verbose {
            eprintln!("INFO: Saving state file");
        }
        self.state.save_to(&self.state_path)
    }

    pub fn choose_filename(&self, ext: &str) -> String {
        let id = uuid::Uuid::new_v4();
        if ext.is_empty() {
            id.to_string()
        } else {
            format!("{}.{}", id, ext)
        }
    }

    /// Get the path where the file would be if it is in storage
    pub fn get_storage_path(&self, name: impl AsRef<Path>) -> eyre::Result<PathBuf> {
        // TODO: Cache this path
        let mut storage_path = self.config.storage_path(&self.config_path)?;
        storage_path.push(name);
        Ok(storage_path)
    }

    // TODO: I think these lifetimes are iffy
    /// Given some prefix (or exact version) of the id, get the file info structure
    pub fn find_file_from_prefix<'a, 'b: 'a>(
        &'a self,
        prefix: &'b str,
    ) -> impl Iterator<Item = &'a FileInfo> + 'a {
        self.state
            .storage
            .files
            .iter()
            .filter(move |x| x.filename.starts_with(prefix))
    }

    // TODO: I think these lifetimes are iffy
    /// Given some prefix (or exact version) of the id, get the file info structure
    pub fn find_file_mut_from_prefix<'a, 'b: 'a>(
        &'a mut self,
        prefix: &'b str,
    ) -> impl Iterator<Item = &'a mut FileInfo> + 'a {
        self.state
            .storage
            .files
            .iter_mut()
            .filter(move |x| x.filename.starts_with(prefix))
    }
}
