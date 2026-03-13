use anyhow::{Context, Result};
use std::process::Command;
use crate::model::FileEntry;

/// Search file content using ripgrep
pub fn search_content(keyword: &str, roots: &[String], limit: usize) -> Result<Vec<FileEntry>> {
    let mut results = Vec::new();

    for root in roots {
        // Run ripgrep to search for content
        let output = Command::new("rg")
            .arg("--files-with-matches") // Only return filenames
            .arg("--max-count=1") // Stop after first match per file
            .arg("--ignore-case") // Case-insensitive search
            .arg("--no-messages") // Suppress error messages
            .arg(keyword)
            .arg(root)
            .output()
            .context("Failed to execute ripgrep. Make sure 'rg' is installed and in PATH")?;

        if !output.status.success() {
            log::warn!("ripgrep search failed for root: {}", root);
            continue;
        }

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let path = line.trim();
            if path.is_empty() {
                continue;
            }

            // Extract filename from path
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();

            // Use backward compatibility method for content search
            results.push(FileEntry::from_path_filename(path.to_string(), filename));

            if results.len() >= limit {
                return Ok(results);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_content() {
        // This test requires ripgrep to be installed
        // We'll just verify the function doesn't panic
        let roots = vec![std::env::temp_dir().to_string_lossy().to_string()];
        let _ = search_content("test", &roots, 10);
    }
}
