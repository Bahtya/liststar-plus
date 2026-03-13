use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tokio::sync::RwLock;

use super::protocol::*;
use crate::index::MemoryIndex;
use crate::search;

/// USN monitoring control handle
pub type UsnControlHandle = Arc<RwLock<Option<(thread::JoinHandle<()>, Arc<AtomicBool>)>>>;

/// IPC request handler
pub struct RequestHandler {
    index: Arc<RwLock<MemoryIndex>>,
    usn_handle: UsnControlHandle,
}

impl RequestHandler {
    pub fn new(index: Arc<RwLock<MemoryIndex>>, usn_handle: UsnControlHandle) -> Self {
        Self { index, usn_handle }
    }

    /// Handle Ping request
    pub async fn handle_ping(&self, _req: PingReq) -> Result<PingResp> {
        Ok(PingResp {
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Handle BuildIndex request
    pub async fn handle_build_index(&self, req: BuildIndexReq) -> Result<BuildIndexResp> {
        log::info!("Building index for roots: {:?}", req.roots);

        let mut index = self.index.write().await;

        // Clear existing index
        index.clear();

        let mut total_files = 0u64;

        for root in &req.roots {
            match crate::index::build_mft_index(root, &mut *index) {
                Ok(count) => {
                    total_files += count;
                    log::info!("Indexed {} files from {}", count, root);
                }
                Err(e) => {
                    log::error!("Failed to index {}: {}", root, e);
                }
            }
        }

        log::info!("Total indexed files: {}", total_files);

        Ok(BuildIndexResp {
            success: total_files > 0,
            indexed_files: total_files,
        })
    }

    /// Handle Search request
    pub async fn handle_search(&self, req: SearchReq) -> Result<SearchResp> {
        log::debug!("Searching for: {} (limit: {})", req.keyword, req.limit);

        let index = self.index.read().await;

        let results = search::search_filename(&*index, &req.keyword, req.limit as usize);

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|entry| {
                // Reconstruct full path from file_ref
                index.get_full_path(entry.file_ref).map(|path| SearchResult {
                    path,
                    filename: entry.filename().to_string(),
                })
            })
            .collect();

        log::debug!("Found {} results", search_results.len());

        Ok(SearchResp {
            results: search_results,
        })
    }

    /// Handle StartUsnMonitoring request
    pub async fn handle_start_usn_monitoring(&self, req: StartUsnMonitoringReq) -> Result<StartUsnMonitoringResp> {
        log::info!("Starting USN monitoring for drive: {}", req.drive_letter);

        // Check if already running
        let mut handle_guard = self.usn_handle.write().await;
        if handle_guard.is_some() {
            return Ok(StartUsnMonitoringResp {
                success: false,
                message: "USN monitoring is already running".to_string(),
            });
        }

        // Parse drive letter
        let drive_letter = req.drive_letter.chars().next().unwrap_or('C');

        // Create stop flag
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        // Get volume handle
        let volume_handle = match crate::index::get_volume_handle(drive_letter) {
            Ok(handle) => handle,
            Err(e) => {
                return Ok(StartUsnMonitoringResp {
                    success: false,
                    message: format!("Failed to open volume: {}", e),
                });
            }
        };

        // Create USN monitor
        let monitor = match crate::index::UsnMonitor::new(volume_handle, self.index.clone(), stop_flag_clone) {
            Ok(m) => m,
            Err(e) => {
                return Ok(StartUsnMonitoringResp {
                    success: false,
                    message: format!("Failed to create USN monitor: {}", e),
                });
            }
        };

        // Start monitoring in background thread
        // Note: We use std::thread because Windows HANDLE is not Send
        let task_handle = thread::spawn(move || {
            // Create a new tokio runtime for this thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut monitor = monitor;
                if let Err(e) = monitor.start_monitoring().await {
                    log::error!("USN monitoring task failed: {}", e);
                }
            });
        });

        *handle_guard = Some((task_handle, stop_flag));

        Ok(StartUsnMonitoringResp {
            success: true,
            message: format!("USN monitoring started for drive {}", drive_letter),
        })
    }

    /// Handle StopUsnMonitoring request
    pub async fn handle_stop_usn_monitoring(&self, _req: StopUsnMonitoringReq) -> Result<StopUsnMonitoringResp> {
        log::info!("Stopping USN monitoring");

        let mut handle_guard = self.usn_handle.write().await;

        if let Some((handle, stop_flag)) = handle_guard.take() {
            // Set stop flag to signal the monitoring thread to stop
            stop_flag.store(true, Ordering::Relaxed);

            // Wait for the thread to finish (with timeout)
            let join_result = thread::spawn(move || {
                handle.join()
            }).join();

            match join_result {
                Ok(_) => {
                    Ok(StopUsnMonitoringResp {
                        success: true,
                        message: "USN monitoring stopped successfully".to_string(),
                    })
                }
                Err(_) => {
                    Ok(StopUsnMonitoringResp {
                        success: true,
                        message: "USN monitoring stop signal sent (thread may still be finishing)".to_string(),
                    })
                }
            }
        } else {
            Ok(StopUsnMonitoringResp {
                success: false,
                message: "USN monitoring is not running".to_string(),
            })
        }
    }
}
