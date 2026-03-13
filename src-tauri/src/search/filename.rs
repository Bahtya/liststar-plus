use crate::index::MemoryIndex;
use crate::model::FileEntry;

/// Search for files by filename (case-insensitive substring match)
pub fn search_filename(index: &MemoryIndex, keyword: &str, limit: usize) -> Vec<FileEntry> {
    index.search(keyword, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::MemoryIndex;

    #[test]
    fn test_search_filename() {
        let mut index = MemoryIndex::new();

        // Add entries with new signature: (drive_letter, file_ref, parent_ref, name, size, attributes)
        index.add_entry(FileEntry::new(
            'C',
            1,
            0,
            "document.txt".to_string(),
            1024,
            0,
        ));

        index.add_entry(FileEntry::new('C', 2, 0, "image.png".to_string(), 2048, 0));

        let results = search_filename(&index, "doc", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "document.txt");

        let results = search_filename(&index, "test", 10);
        assert_eq!(results.len(), 0); // "test" is not in any filename
    }
}
