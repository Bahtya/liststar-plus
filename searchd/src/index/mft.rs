use anyhow::{Context, Result};
use std::mem;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL};
use windows::Win32::System::IO::DeviceIoControl;

use crate::model::FileEntry;
use super::MemoryIndex;

// USN Journal structures
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct USN_JOURNAL_DATA_V0 {
    usn_journal_id: u64,
    first_usn: i64,
    next_usn: i64,
    lowest_valid_usn: i64,
    max_usn: i64,
    maximum_size: u64,
    allocation_delta: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MFT_ENUM_DATA_V0 {
    start_file_reference_number: u64,
    low_usn: i64,
    high_usn: i64,
}

#[repr(C)]
#[derive(Debug)]
struct USN_RECORD_V2 {
    record_length: u32,
    major_version: u16,
    minor_version: u16,
    file_reference_number: u64,
    parent_file_reference_number: u64,
    usn: i64,
    time_stamp: i64,
    reason: u32,
    source_info: u32,
    security_id: u32,
    file_attributes: u32,
    file_name_length: u16,
    file_name_offset: u16,
    // Followed by filename (UTF-16)
}

const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;

/// Build initial index from NTFS MFT using USN Journal enumeration
/// This is much faster than filesystem walking
pub fn build_mft_index(root: &str, index: &mut MemoryIndex) -> Result<u64> {
    log::info!("Building MFT index for: {}", root);

    // Check if this is a root drive (e.g., C:\, D:\)
    // MFT enumeration indexes the entire drive, so only use it for root paths
    if is_root_drive_path(root) {
        // Extract drive letter from path
        let drive_letter = extract_drive_letter(root)?;
        log::debug!("Root drive detected, using MFT enumeration for drive {}", drive_letter);

        // Try MFT-based indexing first, fall back to filesystem walk if it fails
        match build_mft_index_fast(drive_letter, index) {
            Ok(count) => {
                log::info!("MFT-based indexing completed: {} files", count);
                Ok(count)
            }
            Err(e) => {
                log::warn!("MFT-based indexing failed: {}, falling back to filesystem walk", e);
                let mut count = 0u64;
                walk_directory(root, index, &mut count)?;
                Ok(count)
            }
        }
    } else {
        // For subdirectories, use filesystem walk
        log::info!("Subdirectory detected, using filesystem walk");
        let mut count = 0u64;
        walk_directory(root, index, &mut count)?;
        Ok(count)
    }
}

/// Check if path is a root drive (e.g., C:\, D:\)
fn is_root_drive_path(path: &str) -> bool {
    let path_chars: Vec<char> = path.chars().collect();

    // Check for patterns like "C:\", "D:\", etc.
    if path_chars.len() == 3 && path_chars[1] == ':' && (path_chars[2] == '\\' || path_chars[2] == '/') {
        return path_chars[0].is_ascii_alphabetic();
    }

    // Also accept "C:", "D:" without trailing slash
    if path_chars.len() == 2 && path_chars[1] == ':' {
        return path_chars[0].is_ascii_alphabetic();
    }

    false
}

/// Fast MFT-based indexing using USN Journal enumeration
fn build_mft_index_fast(drive_letter: char, index: &mut MemoryIndex) -> Result<u64> {
    let volume_handle = get_volume_handle(drive_letter)?;

    let result = enumerate_mft_records(volume_handle, drive_letter, index);

    unsafe {
        let _ = CloseHandle(volume_handle);
    }

    result
}

/// Enumerate all MFT records using FSCTL_ENUM_USN_DATA
fn enumerate_mft_records(volume_handle: HANDLE, drive_letter: char, index: &mut MemoryIndex) -> Result<u64> {
    // Query USN Journal to get valid USN range
    let journal_data = query_usn_journal(volume_handle)?;
    log::debug!("USN Journal: first_usn={}, next_usn={}", journal_data.first_usn, journal_data.next_usn);

    let mut enum_data = MFT_ENUM_DATA_V0 {
        start_file_reference_number: 0,
        low_usn: 0,
        high_usn: journal_data.next_usn,
    };

    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    let mut count = 0u64;

    log::info!("Starting MFT enumeration...");

    loop {
        let mut bytes_returned = 0u32;

        let result = unsafe {
            DeviceIoControl(
                volume_handle,
                FSCTL_ENUM_USN_DATA,
                Some(&enum_data as *const _ as *const std::ffi::c_void),
                mem::size_of::<MFT_ENUM_DATA_V0>() as u32,
                Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if result.is_err() || bytes_returned == 0 {
            break;
        }

        // Parse USN records
        let mut offset = 8usize; // Skip first 8 bytes (next USN)

        while offset + mem::size_of::<USN_RECORD_V2>() <= bytes_returned as usize {
            let record_ptr = unsafe { buffer.as_ptr().add(offset) as *const USN_RECORD_V2 };
            let record = unsafe { &*record_ptr };

            if record.record_length == 0 {
                break;
            }

            // Extract filename
            let filename_offset = offset + record.file_name_offset as usize;
            let filename_len = record.file_name_length as usize / 2; // UTF-16 chars

            if filename_offset + record.file_name_length as usize <= bytes_returned as usize {
                let filename_slice = unsafe {
                    std::slice::from_raw_parts(
                        buffer.as_ptr().add(filename_offset) as *const u16,
                        filename_len,
                    )
                };

                let filename = String::from_utf16_lossy(filename_slice);

                // Skip directories and system files
                let is_directory = (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                if !is_directory && !filename.starts_with('$') {
                    // Use MFT references for proper indexing
                    let file_entry = FileEntry::new(
                        record.file_reference_number,
                        record.parent_file_reference_number,
                        filename,
                        0,  // size - not available from USN records
                        record.file_attributes,
                    );
                    index.add_entry(file_entry);
                    count += 1;

                    if count % 10000 == 0 {
                        log::info!("Indexed {} files...", count);
                    }
                }
            }

            offset += record.record_length as usize;

            // Update start position for next iteration
            enum_data.start_file_reference_number = record.file_reference_number;
        }
    }

    log::info!("MFT enumeration complete: {} files indexed", count);
    Ok(count)
}

/// Query USN Journal information
fn query_usn_journal(volume_handle: HANDLE) -> Result<USN_JOURNAL_DATA_V0> {
    let mut journal_data = USN_JOURNAL_DATA_V0 {
        usn_journal_id: 0,
        first_usn: 0,
        next_usn: 0,
        lowest_valid_usn: 0,
        max_usn: 0,
        maximum_size: 0,
        allocation_delta: 0,
    };

    let mut bytes_returned = 0u32;

    unsafe {
        DeviceIoControl(
            volume_handle,
            FSCTL_QUERY_USN_JOURNAL,
            None,
            0,
            Some(&mut journal_data as *mut _ as *mut std::ffi::c_void),
            mem::size_of::<USN_JOURNAL_DATA_V0>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .context("Failed to query USN journal")?;
    }

    Ok(journal_data)
}

/// Extract drive letter from path
fn extract_drive_letter(path: &str) -> Result<char> {
    let path_chars: Vec<char> = path.chars().collect();

    if path_chars.len() >= 2 && path_chars[1] == ':' {
        let drive = path_chars[0].to_ascii_uppercase();
        if drive.is_ascii_alphabetic() {
            return Ok(drive);
        }
    }

    anyhow::bail!("Invalid path format, expected drive letter (e.g., C:\\)")
}

/// Recursively walk directory and add files to index (fallback method)
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
                // Use backward compatibility method for filesystem walk
                let file_entry = FileEntry::from_path_filename(path_str.clone(), filename_str);
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
        );

        if handle.is_err() || handle.as_ref().unwrap().is_invalid() {
            anyhow::bail!("Failed to open volume handle for drive {}", drive_letter);
        }

        Ok(handle?)
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
    fn test_is_root_drive_path() {
        // Valid root drives
        assert!(is_root_drive_path("C:\\"));
        assert!(is_root_drive_path("D:\\"));
        assert!(is_root_drive_path("C:"));
        assert!(is_root_drive_path("D:"));

        // Invalid - subdirectories
        assert!(!is_root_drive_path("C:\\Windows"));
        assert!(!is_root_drive_path("D:\\Project\\test"));
        assert!(!is_root_drive_path("C:\\Users\\test"));

        // Invalid - other formats
        assert!(!is_root_drive_path("invalid"));
        assert!(!is_root_drive_path("\\\\network\\share"));
    }

    #[test]
    fn test_extract_drive_letter() {
        assert_eq!(extract_drive_letter("C:\\test").unwrap(), 'C');
        assert_eq!(extract_drive_letter("D:\\folder\\file.txt").unwrap(), 'D');
        assert_eq!(extract_drive_letter("C:\\").unwrap(), 'C');
        assert!(extract_drive_letter("invalid").is_err());
    }

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
