use crate::model::FileEntry;
use std::collections::HashMap;
use std::sync::Arc;

/// High-performance memory index using Vec-based architecture
/// Inspired by Everything: sequential scan with SIMD optimization potential
pub struct MemoryIndex {
    /// Main table: sequential storage for fast scanning
    entries: Vec<FileEntry>,

    /// Fast lookup: (drive_letter, file_ref) → index in entries Vec
    file_ref_map: HashMap<(char, u64), usize>,

    /// Parent-child relationships for path reconstruction
    /// (drive_letter, parent_ref) → Vec<child_index>
    children_map: HashMap<(char, u64), Vec<usize>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            file_ref_map: HashMap::new(),
            children_map: HashMap::new(),
        }
    }

    /// Add a file entry to the index
    pub fn add_entry(&mut self, entry: FileEntry) {
        let index = self.entries.len();
        let drive_letter = entry.drive_letter;
        let file_ref = entry.file_ref;
        let parent_ref = entry.parent_ref;

        // Add to main table
        self.entries.push(entry);

        // Update file_ref lookup map
        self.file_ref_map.insert((drive_letter, file_ref), index);

        // Update parent-child relationships
        self.children_map
            .entry((drive_letter, parent_ref))
            .or_insert_with(Vec::new)
            .push(index);
    }

    /// Remove a file entry by drive letter and file reference
    pub fn remove_entry(&mut self, drive_letter: char, file_ref: u64) {
        if let Some(&index) = self.file_ref_map.get(&(drive_letter, file_ref)) {
            // Remove from lookup map
            self.file_ref_map.remove(&(drive_letter, file_ref));

            // Remove from children map
            let parent_ref = self.entries[index].parent_ref;
            if let Some(children) = self.children_map.get_mut(&(drive_letter, parent_ref)) {
                children.retain(|&i| i != index);
            }

            // Note: We don't actually remove from entries Vec to keep indices stable
        }
    }

    /// Update a file entry (for rename operations)
    pub fn update_entry(&mut self, drive_letter: char, file_ref: u64, new_entry: FileEntry) {
        if let Some(&index) = self.file_ref_map.get(&(drive_letter, file_ref)) {
            // Update parent-child relationships if parent changed
            let old_parent = self.entries[index].parent_ref;
            let new_parent = new_entry.parent_ref;

            if old_parent != new_parent {
                // Remove from old parent
                if let Some(children) = self.children_map.get_mut(&(drive_letter, old_parent)) {
                    children.retain(|&i| i != index);
                }

                // Add to new parent
                self.children_map
                    .entry((drive_letter, new_parent))
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
            // Skip removed entries (check if still in map)
            if !self
                .file_ref_map
                .contains_key(&(entry.drive_letter, entry.file_ref))
            {
                continue;
            }

            // Skip if already reached limit
            if results.len() >= limit {
                break;
            }

            // Case-insensitive substring match
            if entry.name.to_lowercase().contains(&keyword_lower) {
                results.push(entry.clone());
            }
        }

        results
    }

    /// Reconstruct full path for a file entry
    pub fn get_full_path(&self, drive_letter: char, file_ref: u64) -> Option<String> {
        let mut path_components = Vec::new();
        let mut current_ref = file_ref;
        let mut visited = std::collections::HashSet::new();

        // Traverse up the directory tree
        while let Some(&index) = self.file_ref_map.get(&(drive_letter, current_ref)) {
            // Prevent infinite loops
            if !visited.insert(current_ref) {
                break;
            }

            let entry = &self.entries[index];

            // Don't add empty names (root)
            if !entry.name.is_empty() {
                path_components.push(entry.name.as_ref());
            }

            // Stop at root (parent points to itself or is 0)
            if entry.parent_ref == 0 || entry.parent_ref == current_ref {
                break;
            }

            current_ref = entry.parent_ref;
        }

        // Build path from root to file
        if path_components.is_empty() {
            return None;
        }

        path_components.reverse();
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

    /// Get entry by drive letter and file reference
    pub fn get_entry(&self, drive_letter: char, file_ref: u64) -> Option<&FileEntry> {
        self.file_ref_map
            .get(&(drive_letter, file_ref))
            .and_then(|&index| self.entries.get(index))
    }

    /// Get children of a directory
    pub fn get_children(&self, drive_letter: char, parent_ref: u64) -> Vec<&FileEntry> {
        self.children_map
            .get(&(drive_letter, parent_ref))
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
