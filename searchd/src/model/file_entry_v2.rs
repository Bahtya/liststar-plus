use std::sync::Arc;

/// Compact file entry optimized for memory and search performance
/// Based on Everything's architecture: Vec-based sequential scan with SIMD
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// MFT File Reference Number (unique identifier)
    pub file_ref: u64,

    /// Parent directory MFT Reference
    pub parent_ref: u64,

    /// File name only (not full path)
    pub name: Arc<str>,

    /// File size in bytes
    pub size: u64,

    /// File attributes (directory, hidden, system, etc.)
    pub attributes: u32,
}

impl FileEntry {
    pub fn new(file_ref: u64, parent_ref: u64, name: String, size: u64, attributes: u32) -> Self {
        Self {
            file_ref,
            parent_ref,
            name: Arc::from(name),
            size,
            attributes,
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
}

/// High-performance memory index using Vec-based architecture
/// Inspired by Everything: sequential scan with SIMD optimization potential
pub struct MemoryIndex {
    /// Main table: sequential storage for fast scanning
    entries: Vec<FileEntry>,

    /// Fast lookup: file_ref → index in entries Vec
    file_ref_map: std::collections::HashMap<u64, usize>,

    /// Parent-child relationships for path reconstruction
    /// parent_ref → Vec<child_index>
    children_map: std::collections::HashMap<u64, Vec<usize>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            file_ref_map: std::collections::HashMap::new(),
            children_map: std::collections::HashMap::new(),
        }
    }

    /// Add a file entry to the index
    pub fn add_entry(&mut self, entry: FileEntry) {
        let index = self.entries.len();
        let file_ref = entry.file_ref;
        let parent_ref = entry.parent_ref;

        // Add to main table
        self.entries.push(entry);

        // Update file_ref lookup map
        self.file_ref_map.insert(file_ref, index);

        // Update parent-child relationships
        self.children_map
            .entry(parent_ref)
            .or_insert_with(Vec::new)
            .push(index);
    }

    /// Remove a file entry by file reference
    pub fn remove_entry(&mut self, file_ref: u64) {
        if let Some(&index) = self.file_ref_map.get(&file_ref) {
            // Mark as removed (don't actually remove to keep indices stable)
            // In production, use a tombstone or compaction strategy
            self.file_ref_map.remove(&file_ref);

            // Remove from children map
            let parent_ref = self.entries[index].parent_ref;
            if let Some(children) = self.children_map.get_mut(&parent_ref) {
                children.retain(|&i| i != index);
            }
        }
    }

    /// Update a file entry (for rename operations)
    pub fn update_entry(&mut self, file_ref: u64, new_entry: FileEntry) {
        if let Some(&index) = self.file_ref_map.get(&file_ref) {
            // Update parent-child relationships if parent changed
            let old_parent = self.entries[index].parent_ref;
            let new_parent = new_entry.parent_ref;

            if old_parent != new_parent {
                // Remove from old parent
                if let Some(children) = self.children_map.get_mut(&old_parent) {
                    children.retain(|&i| i != index);
                }

                // Add to new parent
                self.children_map
                    .entry(new_parent)
                    .or_insert_with(Vec::new)
                    .push(index);
            }

            // Update entry
            self.entries[index] = new_entry;
        }
    }

    /// Search for files by keyword (case-insensitive substring match)
    /// Uses sequential scan - can be optimized with SIMD in the future
    pub fn search(&self, keyword: &str, limit: usize) -> Vec<FileEntry> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        // Sequential scan through all entries
        for entry in &self.entries {
            // Skip if already reached limit
            if results.len() >= limit {
                break;
            }

            // Case-insensitive substring match
            if entry.name_lowercase().contains(&keyword_lower) {
                results.push(entry.clone());
            }
        }

        results
    }

    /// Reconstruct full path for a file entry
    pub fn get_full_path(&self, file_ref: u64, drive_letter: char) -> Option<String> {
        let mut path_components = V();
        let mut current_ref = file_ref;

        // Traverse up the directory tree
        while let Some(&index) = self.file_ref_map.get(&current_ref) {
            let entry = &self.entries[index];
            path_components.push(entry.name.as_ref());

            // Stop at root
            if entry.parent_ref == 0 || entry.parent_ref == current_ref {
                break;
            }

            current_ref = entry.parent_ref;
        }

        // Build path from root to file
        if path_components.is_empty() {
            return None;
        }

        path_componverse();
        Some(format!("{}:\\{}", drive_letter, path_components.join("\\")))
    }

    /// Get total number of indexed files
    pub fn total_files(&self) -> usize {
        self.file_ref_map.len()
    }

    /// Clear the entire index
    pub fn clear(&mut self) {
        self.entries.clear();
        self.file_ref_map.clear();
        self.children_map.clear();
    }

    /// Get entry by file reference
    pub fn get_entry(&self, file_ref: u64) -> Option<&FileEntry> {
        self.file_ref_map
            .get(&file_ref)
            .and_then(|&index| self.entries.get(index))
    }

    /// Get children of a directory
    pub fn get_chien(&self, parent_ref: u64) -> Vec<&FileEntry> {
        self.children_map
            .get(&parent_ref)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.entries.get(i))
                    .collect()
            })
            .unwrap_or_default()
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
            1,
            0,
            "file1.txt".to_string(),
            1024,
            0,
        ));

        index.add_entry(FileEntry::new(
            2,
            0,
            "file2.txt".to_string(),
            2048,
            0,
        ));

        let results = index.search("file", 10);
        assert_eq!(results.len(), 2);

        let results = index.search("file1", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name.as_ref(), "file1.txt");
    }

    #[test]
    fn test_path_reconstruction() {
        let mut index = MemoryIndex::new();

        // Root directory
        index.add_entry(FileEntry::new(
            5,
            5,  // Root points to itself
            "".to_string(),
            0,
            0x10,  // Directory
        ));

        // Subdirectory: C:\Users
        index.add_entry(FileEntry::new(
            10,
            5,
            "Users".to_string(),
            0,
            0x10,
        ));

        // File: C:\Users\test.txt
        index.add_entry(FileEntry::new(
            20,
            10,
            "test.txt".to_string(),
            1024,
   0,
        ));

 path = index.get_full_path(20, 'C');
        assert!(path.is_some());
        // Path should be something like "C:\Users\test.txt"
    }

    #[test]
    fn test_remove_entry() {
        let mut index = MemoryIndex::new();

        index.add_entry(FileEntry::new(
            1,
            0,
            "file1.txt".to_string(),
            1024,
            0,
        ));

        assert_eq!(index.total_files(), 1);

        index.remove_entry(1);
        assert_eq!(index.total_files(), 0);
    }

    #[test]
    fn test_update_entry() {
        let mut index = MemoryIndex::new();

        index.add_entry(FileEntry::new(
            1,
            0,
            "old_name.txt".to_string(),
            1024,
            0,
        ));

        index.update_entry(
            1,
            FileEntry::new(1, 0, "new_name.txt".to_string(), 1024, 0),
        );

        let entry = index.get_entry(1).unwrap();
        assert_eq!(entry.name.as_ref(), "new_name.txt");
    }
}
