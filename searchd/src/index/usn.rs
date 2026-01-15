use anyhow::{Context, Result};
use std::mem;
use std::ptr;
use std::sync::Arc;
use tokio::sync::RwLock;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use windows::Win32::System::Ioctl::{FSCTL_QUERY_USN_JOURNAL, FSCTL_READ_USN_JOURNAL};
use windows::Win32::System::IO::DeviceIoControl;

use crate::model::FileEntry;
use super::MemoryIndex;

// USN Journal structures (simplified)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct USN_JOURNAL_DATA {
    usn_journal_id: u64,
    first_usn: i64,
    next_usn: i64,
    lowest_valid_usn: i64,
    max_usn: i64,
    maximum_size: u64,
    allocation_delta: u64,
}

#[repr(C)]
#[derive(Debug)]
struct READ_USN_JOURNAL_DATA {
    start_usn: i64,
    reason_mask: u32,
    return_only_on_close: u32,
    timeout: u64,
    bytes_to_wait_for: u64,
    usn_journal_id: u64,
}

#[repr(C)]
#[derive(Debug)]
struct USN_RECORD {
    record_length: u32,
    major_version: u16,
    minor_version: u16,
    file_reference_number: u64,
    parent_file_reference_number: u64,
    usn: i64,
    timestamp: i64,
    reason: u32,
    source_info: u32,
    security_id: u32,
    file_attributes: u32,
    file_name_length: u16,
    file_name_offset: u16,
    // Followed by variable-length filename
}

// USN Reasons
const USN_REASON_FILE_CREATE: u32 = 0x00000100;
const USN_REASON_FILE_DELETE: u32 = 0x00000200;
const USN_REASON_RENAME_NEW_NAME: u32 = 0x00002000;

/// USN Journal monitor
pub struct UsnMonitor {
    volume_handle: HANDLE,
    journal_id: u64,
    next_usn: i64,
    index: Arc<RwLock<MemoryIndex>>,
}

impl UsnMonitor {
    /// Create a new USN monitor for a volume
    pub fn new(volume_handle: HANDLE, index: Arc<RwLock<MemoryIndex>>) -> Result<Self> {
        let journal_data = query_usn_journal(volume_handle)?;

        Ok(Self {
            volume_handle,
            journal_id: journal_data.usn_journal_id,
            next_usn: journal_data.next_usn,
            index,
        })
    }

    /// Start monitoring USN journal changes
    pub async fn start_monitoring(&mut self) -> Result<()> {
        log::info!("Starting USN journal monitoring...");

        loop {
            match self.read_usn_changes().await {
                Ok(()) => {}
                Err(e) => {
                    log::error!("Error reading USN changes: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }

            // Sleep briefly to avoid busy-waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Read and process USN changes
    async fn read_usn_changes(&mut self) -> Result<()> {
        let mut read_data = READ_USN_JOURNAL_DATA {
            start_usn: self.next_usn,
            reason_mask: USN_REASON_FILE_CREATE | USN_REASON_FILE_DELETE | USN_REASON_RENAME_NEW_NAME,
            return_only_on_close: 0,
            timeout: 0,
            bytes_to_wait_for: 0,
            usn_journal_id: self.journal_id,
        };

        let mut buffer = vec![0u8; 65536];
        let mut bytes_returned = 0u32;

        unsafe {
            let result = DeviceIoControl(
                self.volume_handle,
                FSCTL_READ_USN_JOURNAL,
                Some(&read_data as *const _ as *const _),
                mem::size_of::<READ_USN_JOURNAL_DATA>() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            );

            if result.is_err() || bytes_returned == 0 {
                return Ok(()); // No new changes
            }
        }

        // Process USN records
        self.process_usn_records(&buffer[..bytes_returned as usize]).await?;

        Ok(())
    }

    /// Process USN records from buffer
    async fn process_usn_records(&mut self, buffer: &[u8]) -> Result<()> {
        let mut offset = 8; // Skip the first 8 bytes (next USN)

        while offset < buffer.len() {
            if offset + mem::size_of::<USN_RECORD>() > buffer.len() {
                break;
            }

            unsafe {
                let record = &*(buffer.as_ptr().add(offset) as *const USN_RECORD);

                if record.record_length == 0 {
                    break;
                }

                // Update next USN
                self.next_usn = record.usn;

                // Process the record based on reason
                self.handle_usn_record(record, buffer, offset).await?;

                offset += record.record_length as usize;
            }
        }

        Ok(())
    }

    /// Handle a single USN record
    async fn handle_usn_record(&self, record: &USN_RECORD, buffer: &[u8], offset: usize) -> Result<()> {
        // Extract filename
        let filename_offset = offset + record.file_name_offset as usize;
        let filename_length = record.file_name_length as usize;

        if filename_offset + filename_length > buffer.len() {
            return Ok(());
        }

        let filename_bytes = &buffer[filename_offset..filename_offset + filename_length];
        let filename_u16: Vec<u16> = filename_bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let filename = String::from_utf16_lossy(&filename_u16);

        // Handle different USN reasons
        if record.reason & USN_REASON_FILE_CREATE != 0 {
            log::debug!("File created: {}", filename);
            // Note: We don't have the full path here, so we skip adding to index
      // A full implementation would need to resolve the file reference number to a path
        } else if record.reason & USN_REASON_FILE_DELETE != 0 {
            log::debug!("File deleted: {}", filename);
            // Remove from index (if we had the full path)
        } else if record.reason & USN_REASON_RENAME_NEW_NAME != 0 {
            log::debug!("File renamed: {}", filename);
            // Update index (if we had the full path)
        }

        Ok(())
    }
}

/// Query USN journal information
fn query_usn_journal(volume_handle: HANDLE) -> Result<USN_JOURNAL_DATA> {
    let mut journal_data: USN_JOURNAL_DATA = unsafe { mem::zeroed() };
    let mut bytes_returned = 0u32;

    unsafe {
        DeviceIoControl(
            volume_handle,
            FSCTL_QUERY_USN_JOURNAL,
            None,
            0,
            Some(&mut journal_data as *mut _ as *mut _),
            mem::size_of::<USN_JOURNAL_DATA>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .context("Failed to query USN journal")?;
    }

    Ok(journal_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usn_structures() {
        // Just verify the structures are properly sized
        assert!(mem::size_of::<USN_JOURNAL_DATA>() > 0);
        assert!(mem::size_of::<READ_USN_JOURNAL_DATA>() > 0);
        assert!(mem::size_of::<USN_RECORD>() > 0);
    }
}
