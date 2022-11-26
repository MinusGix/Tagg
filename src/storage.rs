//! Storage in tagg is pretty simple. We just store the declarations as toml, and stuff the files in a single folder.   
//! This is enough for most use-cases. A databse would be overkill and also be more complicated.  
//!
//! Though, it aims to be replaceable if we later want to switch to a more robust and efficient storage method.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "kebab-case")]
pub struct Storage {
    pub files: Vec<FileInfo>,
}
impl Storage {
    // TODO: I think these lifetimes are iffy
    /// Given some prefix (or exact version) of the id, get the file info structure
    pub fn find_file_from_prefix<'a, 'b: 'a>(
        &'a self,
        prefix: &'b str,
    ) -> impl Iterator<Item = &'a FileInfo> + 'a {
        self.files
            .iter()
            .filter(move |x| x.filename.starts_with(prefix))
    }

    pub(crate) fn find_single_file_mut_from_prefix<'a>(
        &'a mut self,
        prefix: &str,
    ) -> Option<&'a mut FileInfo> {
        self.files
            .iter_mut()
            .find(move |x| x.filename.starts_with(prefix))
    }

    // TODO: I think these lifetimes are iffy
    /// Given some prefix (or exact version) of the id, get the file info structure
    pub fn find_file_mut_from_prefix<'a, 'b: 'a>(
        &'a mut self,
        prefix: &'b str,
    ) -> impl Iterator<Item = &'a mut FileInfo> + '_ {
        self.files
            .iter_mut()
            .filter(move |x| x.filename.starts_with(prefix))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FileInfo {
    // TODO: Use strings that are references into some loaded toml file
    // this would avoid lots of individual allocations.
    /// The name of the file in the storage folder. Note that this is not the original filename.  
    /// Tagg replaces the filename with a randomly generated id. This avoids issues of name collision,
    /// because Tagg stores them in a flat structure (aka a single folder).  
    /// This includes the file extension.  
    pub filename: String,

    /// The filename that it originally had.  
    /// Unlike some tagging software, this is kept around because it can be useful to know.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,

    // TODO: Should we make this a `HashSet`?
    /// The tag list for the file
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Various comment information about the file.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub comments: HashMap<String, String>,
}
