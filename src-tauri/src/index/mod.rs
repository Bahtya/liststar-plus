pub mod mft;
pub mod usn;

pub use crate::model::MemoryIndex;
pub use mft::build_mft_index;
pub use usn::UsnMonitor;

use anyhow::Result;
use windows::Win32::Foundation::HANDLE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHandle(pub HANDLE);

unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

pub fn get_volume_handle(drive_letter: char) -> Result<SendHandle> {
    mft::get_volume_handle(drive_letter).map(SendHandle)
}

pub fn close_volume_handle(handle: SendHandle) {
    mft::close_volume_handle(handle.0);
}
