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

        index.add_entry(FileEntry::new(
            "C:\\test\\document.txt".to_string(),
            "document.txt".to_string(),
        ));

        index.add_entry(FileEntry::new(
            "C:\\test\\image.png".to_string(),
            "image.png".to_string(),
        ));

        let results = search_filename(&index, "doc", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "document.txt");

        let results = search_filename(&index, "test", 10);
        assert_eq!(results.len(), 0); // "test" is in path, not filename
    }
}
