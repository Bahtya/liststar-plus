pub mod file_entry;
pub mod memory_index;

pub use file_entry::FileEntry;
pub use memory_index::MemoryIndex;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub attributes: u32,
}
