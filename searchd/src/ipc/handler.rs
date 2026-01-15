use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::protocol::*;
use crate::index::MemoryIndex;
use crate::search;

/// IPC request handler
pub struct RequestHandler {
    index: Arc<RwLock<MemoryIndex>>,
}

impl RequestHandler {
    pub fn new(index: Arc<RwLock<MemoryIndex>>) -> Self {
        Self { index }
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
            .map(|entry| SearchResult {
                path: entry.path.clone(),
                filename: entry.filename.clone(),
            })
            .collect();

        log::debug!("Found {} results", search_results.len());

        Ok(SearchResp {
            results: search_results,
        })
    }
}
