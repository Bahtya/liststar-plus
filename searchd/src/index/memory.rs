use std::collections::HashMap;
use crate::model::FileEntry;

/// In-memory file index using HashMap
/// Key: lowercase filename, Value: list of file entries
pub struct MemoryIndex {
    index: HashMap<String, Vec<FileEntry>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Add a file entry to the index
    pub fn add_entry(&mut self, entry: FileEntry) {
        let key = entry.filename.to_lowercase();
        self.index.entry(key).or_insert_with(Vec::new).push(entry);
    }

    /// Remove a file entry by path
    pub fn remove_entry(&mut self, path: &str) {
        // Find and remove the entry from all keys
        for entries in self.index.values_mut() {
            entries.retain(|e| e.path != path);
        }

        // Clean up empty entries
        self.index.retain(|_, v| !v.is_empty());
    }

    /// Search for files by keyword (case-insensitive substring match)
    pub fn search(&self, keyword: &str, limit: usize) -> Vec<FileEntry> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        for (filename, entries) in &self.index {
            if filename.contains(&keyword_lower) {
                for entry in entries {
                    results.push(entry.clone());
                    if results.len() >= limit {
                        return results;
                    }
                }
            }
        }

        results
    }

    /// Get total number of indexed files
    pub fn total_files(&self) -> usize {
        self.index.values().map(|v| v.len()).sum()
    }

    /// Clear the entire index
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// Update a file entry (for rename operations)
    pub fn update_entry(&mut self, old_path: &str, new_entry: FileEntry) {
        self.remove_entry(old_path);
        self.add_entry(new_entry);
    }
}

impl Default for MemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_search() {
        let mut index = MemoryIndex::new();

        index.add_entry(FileEntry::new(
            "C:\\test\\file1.txt".to_string(),
            "file1.txt".to_string(),
        ));

        index.add_entry(FileEntry::new(
            "C:\\test\\file2.txt".to_string(),
            "file2.txt".to_string(),
        ));

        let results = index.search("file", 10);
        assert_eq!(results.len(), 2);

        let results = index.search("file1", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "file1.txt");
    }

    #[test]
    fn test_remove_entry() {
        let mut index = MemoryIndex::new();

        index.add_entry(FileEntry::new(
            "C:\\test\\file1.txt".to_string(),
            "file1.txt".to_string(),
        ));

        assert_eq!(index.total_files(), 1);

        index.remove_entry("C:\\test\\file1.txt");
        assert_eq!(index.total_files(), 0);
    }
}
