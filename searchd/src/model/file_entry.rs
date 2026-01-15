use serde::{Deserialize, Serialize};

/// Represents a file entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full path to the file
    pub path: String,
    /// Filename only (for quick lookup)
    pub filename: String,
}

impl FileEntry {
    pub fn new(path: String, filename: String) -> Self {
        Self { path, filename }
    }
}
