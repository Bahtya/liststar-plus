use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Compact file entry optimized for memory and search performance
/// Based on Everything's architecture: Vec-based sequential scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Drive letter (e.g., 'C')
    pub drive_letter: char,

    /// MFT File Reference Number (unique identifier within volume)
    pub file_ref: u64,

    /// Parent directory MFT Reference
    pub parent_ref: u64,

    /// File name only (not full path)
    #[serde(with = "arc_str_serde")]
    pub name: Arc<str>,

    /// File size in bytes
    pub size: u64,

    /// File attributes (directory, hidden, system, etc.)
    pub attributes: u32,
}

// Custom serde for Arc<str>
mod arc_str_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(arc: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arc.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Arc::from(s))
    }
}

impl FileEntry {
    pub fn new(
        drive_letter: char,
        file_ref: u64,
        parent_ref: u64,
        name: String,
        size: u64,
        attributes: u32,
    ) -> Self {
        Self {
            drive_letter,
            file_ref,
            parent_ref,
            name: Arc::from(name),
            size,
            attributes,
        }
    }

    /// Create from old-style path + filename (for backward compatibility)
    pub fn from_path_filename(path: String, filename: String) -> Self {
        // Generate a simple hash as file_ref for compatibility
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let file_ref = hasher.finish();

        // Extract drive letter if possible
        let drive_letter = path.chars().next().unwrap_or('C').to_ascii_uppercase();

        Self {
            drive_letter,
            file_ref,
            parent_ref: 0,
            name: Arc::from(filename),
            size: 0,
            attributes: 0,
        }
    }

    /// Check if this entry is a directory
    pub fn is_directory(&self) -> bool {
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
        (self.attributes & FILE_ATTRIBUTE_DIRECTORY) != 0
    }

    /// Get lowercase name for case-insensitive search
    pub fn name_lowercase(&self) -> String {
        self.name.to_lowercase()
    }

    /// Get the filename as a string (for compatibility)
    pub fn filename(&self) -> &str {
        &self.name
    }
}
