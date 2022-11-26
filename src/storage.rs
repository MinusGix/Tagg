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

    /// Various comment information about the file.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub comments: HashMap<String, String>,

    // TODO: Should we make this a `HashSet`?
    /// The tag list for the file
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}
