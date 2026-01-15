use anyhow::{Context, Result};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, FSCTL_READ_FILE_USN_DATA};
use windows::Win32::System::IO::DeviceIoControl;

use crate::model::FileEntry;
use super::MemoryIndex;

/// Build initial index from NTFS MFT
/// This is a simplified implementation that walks the filesystem
/// A full MFT reader would require parsing the $MFT file directly
pub fn build_mft_index(root: &str, index: &mut MemoryIndex) -> Result<u64> {
    let mut count = 0u64;

    // For MVP, we'll use a simple filesystem walk
    // A production version would read the MFT directly for better performance
    walk_directory(root, index, &mut count)?;

    Ok(count)
}

/// Recursively walk directory and add files to index
fn walk_directory(path: &str, index: &mut MemoryIndex, count: &mut u64) -> Result<()> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read directory {}: {}", path, e);
            return Ok(()); // Skip inaccessible directories
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to get metadata for {}: {}", path_str, e);
                continue;
            }
        };

        if metadata.is_file() {
            // Add file to index
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy().to_string();
                let file_entry = FileEntry::new(path_str.clone(), filename_str);
                index.add_entry(file_entry);
                *count += 1;

                if *count % 10000 == 0 {
                    log::info!("Indexed {} files...", count);
                }
            }
        } else if metadata.is_dir() {
            // Recursively walk subdirectories
            walk_directory(&path_str, index, count)?;
        }
    }

    Ok(())
}

/// Get volume handle for USN operations
pub fn get_volume_handle(drive_letter: char) -> Result<HANDLE> {
    let volume_path = format!("\\\\.\\{}:", drive_letter);
    let wide_path: Vec<u16> = volume_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let handle = CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
)
        .context("Failed to open volume")?;

        if handle == INVALID_HANDLE_VALUE {
            anyhow::bail!("Invalid volume handle");
        }

        Ok(handle)
    }
}

/// Close volume handle
pub fn close_volume_handle(handle: HANDLE) {
    unsafe {
        let _ = CloseHandle(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index() {
        let mut index = MemoryIndex::new();

        // Test with a small directory
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.to_string_lossy().to_string();

        // This might fail if temp dir is inaccessible, so we just check it doesn't panic
        let _ = build_mft_index(&temp_path, &mut index);
    }
}
