use anyhow::{Context, Result};
use std::mem;
use std::sync::Arc;
use tokio::sync::RwLock;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL};
use windows::Win32::System::IO::DeviceIoControl;

use super::MemoryIndex;
use crate::model::FileEntry;

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

use super::SendHandle;

/// Build initial index from NTFS MFT using USN Journal enumeration
/// This is much faster than filesystem walking
pub async fn build_mft_index(root: &str, index: Arc<RwLock<MemoryIndex>>) -> Result<u64> {
    log::info!("Building MFT index for: {}", root);

    // Check if this is a root drive (e.g., C:\, D:\)
    if is_root_drive_path(root) {
        let drive_letter = extract_drive_letter(root)?;
        log::debug!(
            "Root drive detected, using MFT enumeration for drive {}",
            drive_letter
        );

        // Try MFT-based indexing first
        match build_mft_index_fast(drive_letter, index.clone()).await {
            Ok(count) => {
                log::info!("MFT-based indexing completed: {} files", count);
                Ok(count)
            }
            Err(e) => {
                log::warn!(
                    "MFT-based indexing failed: {}, falling back to filesystem walk",
                    e
                );
                let mut count = 0u64;
                walk_directory(root, index, &mut count).await?;
                Ok(count)
            }
        }
    } else {
        log::info!("Subdirectory detected, using filesystem walk");
        let mut count = 0u64;
        walk_directory(root, index, &mut count).await?;
        Ok(count)
    }
}

/// Fast MFT-based indexing using USN Journal enumeration
async fn build_mft_index_fast(drive_letter: char, index: Arc<RwLock<MemoryIndex>>) -> Result<u64> {
    let volume_handle = SendHandle(get_volume_handle(drive_letter)?);

    let result = enumerate_mft_records(volume_handle, drive_letter, index).await;

    close_volume_handle(volume_handle.0);

    result
}

/// Enumerate all MFT records using FSCTL_ENUM_USN_DATA
async fn enumerate_mft_records(
    volume_handle: SendHandle,
    drive_letter: char,
    index: Arc<RwLock<MemoryIndex>>,
) -> Result<u64> {
    // Query USN Journal to get valid USN range
    let journal_data = query_usn_journal(volume_handle.0)?;
    log::debug!(
        "USN Journal: first_usn={}, next_usn={}",
        journal_data.first_usn,
        journal_data.next_usn
    );

    let mut enum_data = MFT_ENUM_DATA_V0 {
        start_file_reference_number: 0,
        low_usn: 0,
        high_usn: journal_data.next_usn,
    };

    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    let mut count = 0u64;

    log::info!("Starting MFT enumeration for drive {}...", drive_letter);

    loop {
        let mut bytes_returned = 0u32;

        let result = unsafe {
            DeviceIoControl(
                volume_handle.0,
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
        let mut batch = Vec::new();

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
                    let file_entry = FileEntry::new(
                        drive_letter,
                        record.file_reference_number,
                        record.parent_file_reference_number,
                        filename,
                        0, // size - not available from USN records
                        record.file_attributes,
                    );
                    batch.push(file_entry);
                    count += 1;
                }
            }

            offset += record.record_length as usize;
            enum_data.start_file_reference_number = record.file_reference_number;
        }

        // Add batch to index
        if !batch.is_empty() {
            let mut idx = index.write().await;
            for entry in batch {
                idx.add_entry(entry);
            }
        }

        if count % 100000 == 0 && count > 0 {
            log::info!("Drive {}: Indexed {} files...", drive_letter, count);
        }
    }

    log::info!(
        "MFT enumeration complete for drive {}: {} files indexed",
        drive_letter,
        count
    );
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
async fn walk_directory(
    path: &str,
    index: Arc<RwLock<MemoryIndex>>,
    count: &mut u64,
) -> Result<()> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read directory {}: {}", path, e);
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path_buf = entry.path();
        let path_str = path_buf.to_string_lossy().to_string();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_file() {
            if let Some(filename) = path_buf.file_name() {
                let filename_str = filename.to_string_lossy().to_string();
                let file_entry = FileEntry::from_path_filename(path_str.clone(), filename_str);

                let mut idx = index.write().await;
                idx.add_entry(file_entry);
                *count += 1;

                if *count % 10000 == 0 {
                    log::info!("Indexed {} files...", *count);
                }
            }
        } else if metadata.is_dir() {
            // Box the recursion to avoid infinite stack size
            Box::pin(walk_directory(&path_str, index.clone(), count)).await?;
        }
    }

    Ok(())
}

/// Get volume handle for USN operations
pub fn get_volume_handle(drive_letter: char) -> Result<HANDLE> {
    let volume_path = format!("\\\\.\\{}:", drive_letter);
    let wide_path: Vec<u16> = volume_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

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

/// Check if path is a root drive (e.g., C:\, D:\)
fn is_root_drive_path(path: &str) -> bool {
    let path_chars: Vec<char> = path.chars().collect();

    if path_chars.len() == 3
        && path_chars[1] == ':'
        && (path_chars[2] == '\\' || path_chars[2] == '/')
    {
        return path_chars[0].is_ascii_alphabetic();
    }

    if path_chars.len() == 2 && path_chars[1] == ':' {
        return path_chars[0].is_ascii_alphabetic();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MemoryIndex;

    #[tokio::test]
    async fn test_is_root_drive_path() {
        assert!(is_root_drive_path("C:\\"));
        assert!(is_root_drive_path("D:\\"));
        assert!(is_root_drive_path("C:"));
        assert!(is_root_drive_path("D:"));
        assert!(!is_root_drive_path("C:\\Windows"));
    }

    #[tokio::test]
    async fn test_extract_drive_letter() {
        assert_eq!(extract_drive_letter("C:\\test").unwrap(), 'C');
        assert_eq!(extract_drive_letter("D:\\folder\\file.txt").unwrap(), 'D');
    }
}
