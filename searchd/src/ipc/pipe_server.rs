use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE,
    OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe,
    PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

const PIPE_NAME: &str = r"\\.\pipe\listory_plus_search";
const BUFFER_SIZE: u32 = 65536;

pub struct NamedPipeServer {
    pipe_handle: HANDLE,
}

impl NamedPipeServer {
    /// Create a new Named Pipe server
    pub fn new() -> Result<Self> {
        let pipe_name = to_wide_string(PIPE_NAME);

        unsafe {
            let pipe_handle = CreateNamedPipeW(
                PCWSTR(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                BUFFER_SIZE,
                BUFFER_SIZE,
                0,
                Some(ptr::null_mut() as *mut SECURITY_ATTRIBUTES),
            );

            if pipe_handle.is_invalid() {
                anyhow::bail!("Failed to create named pipe");
            }

            log::info!("Named pipe server created: {}", PIPE_NAME);

            Ok(Self { pipe_handle })
        }
    }

    /// Wait for a client to connect
    pub fn accept(&self) -> Result<()> {
        unsafe {
            ConnectNamedPipe(self.pipe_handle, None)
                .context("Failed to connect to client")?;
        }
        log::info!("Client connected");
        Ok(())
    }

    /// Read data from the pipe
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut bytes_read = 0u32;

        unsafe {
            ReadFile(
                self.pipe_handle,
                Some(buffer),
                Some(&mut bytes_read),
                None,
            )
            .context("Failed to read from pipe")?;
        }

        Ok(bytes_read as usize)
    }

    /// Write data to the pipe
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(data),
                Some(&mut bytes_written),
                None,
            )
            .context("Failed to write to pipe")?;
        }

        Ok(bytes_written as usize)
    }

    /// Disconnect the current client
    pub fn disconnect(&self) -> Result<()> {
        unsafe {
            DisconnectNamedPipe(self.pipe_handle)
                .context("Failed to disconnect client")?;
        }
        log::info!("Client disconnected");
        Ok(())
    }
}

impl Drop for NamedPipeServer {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.pipe_handle);
        }
    }
}

/// Named Pipe client for testing
pub struct NamedPipeClient {
    pipe_handle: HANDLE,
}

impl NamedPipeClient {
    /// Connect to the Named Pipe server
    pub fn connect() -> Result<Self> {
        let pipe_name = to_wide_string(PIPE_NAME);
        unsafe {
            let pipe_handle = CreateFileW(
                PCWSTR(pipe_name.as_ptr()),
                (windows::Win32::Storage::FileSystem::FILE_GENERIC_READ.0
                    | windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE.0),
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
            .context("Failed to connect to named pipe")?;

            if pipe_handle == INVALID_HANDLE_VALUE {
                anyhow::bail!("Invalid pipe handle");
            }

            Ok(Self { pipe_handle })
        }
    }

    /// Read data from the pipe
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut bytes_read = 0u32;

        unsafe {
            ReadFile(
                self.pipe_handle,
                Some(buffer),
                Some(&mut bytes_read),
                None,
            )
            .context("Failed to read from pipe")?;
        }

        Ok(bytes_read as usize)
    }

    /// Write data to the pipe
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut bytes_written = 0u32;

        unsafe {
            WriteFile(
                self.pipe_handle,
                Some(data),
                Some(&mut bytes_written),
                None,
            )
            .context("Failed to write to pipe")?;
        }

        Ok(bytes_written as usize)
    }
}

impl Drop for NamedPipeClient {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.pipe_handle);
        }
    }
}

/// Convert a Rust string to a wide (UTF-16) null-terminated string
fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
