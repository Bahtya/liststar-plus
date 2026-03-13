mod index;
mod model;
mod search;

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{State, Emitter, Window};
use crate::model::{MemoryIndex, SearchResult};

struct AppState {
    index: Arc<RwLock<MemoryIndex>>,
}

#[tauri::command]
async fn search(state: State<'_, AppState>, keyword: String, limit: usize) -> Result<Vec<SearchResult>, String> {
    let index = state.index.read().await;
    let results = index.search(&keyword, limit);
    
    let mut search_results = Vec::new();
    for entry in results {
        if let Some(path) = index.get_full_path(entry.drive_letter, entry.file_ref) {
            search_results.push(SearchResult {
                filename: entry.name.to_string(),
                path,
                size: entry.size,
                attributes: entry.attributes,
            });
        }
    }
    
    Ok(search_results)
}

#[tauri::command]
async fn build_index(window: Window, state: State<'_, AppState>, roots: Vec<String>) -> Result<u64, String> {
    let index = state.index.clone();
    let mut handles = Vec::new();
    
    let _ = window.emit("indexing-progress", "Starting...");
    
    for root in roots {
        let index_clone = index.clone();
        let root_clone = root.clone();
        let window_clone = window.clone();
        let handle = tokio::spawn(async move {
            let _ = window_clone.emit("indexing-progress", format!("Indexing {}...", root_clone));
            index::mft::build_mft_index(&root_clone, index_clone).await
        });
        handles.push(handle);
    }
    
    let mut total_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(count)) => total_count += count,
            Ok(Err(e)) => return Err(e.to_string()),
            Err(e) => return Err(e.to_string()),
        }
    }
    
    let _ = window.emit("indexing-progress", Option::<String>::None);
    let _ = window.emit("file-count", total_count);
    
    Ok(total_count)
}

#[tauri::command]
async fn start_usn_monitoring(window: Window, state: State<'_, AppState>, drive_letter: char) -> Result<(), String> {
    let index = state.index.clone();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let window_clone = window.clone();
    
    tokio::spawn(async move {
        let _ = window_clone.emit("usn-status", format!("Monitoring {}", drive_letter));
        match index::get_volume_handle(drive_letter) {
            Ok(handle) => {
                match index::usn::UsnMonitor::new(drive_letter, handle, index, stop_flag) {
                    Ok(mut monitor) => {
                        if let Err(e) = monitor.start_monitoring().await {
                            log::error!("USN monitor error for drive {}: {}", drive_letter, e);
                            let _ = window_clone.emit("usn-status", format!("Error: {}", e));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create USN monitor for drive {}: {}", drive_letter, e);
                        let _ = window_clone.emit("usn-status", format!("Failed: {}", e));
                    }
                }
                index::close_volume_handle(handle);
            }
            Err(e) => {
                log::error!("Failed to get volume handle for drive {}: {}", drive_letter, e);
                let _ = window_clone.emit("usn-status", format!("No Handle: {}", e));
            }
        }
    });
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let index = Arc::new(RwLock::new(MemoryIndex::new()));
    
    tauri::Builder::default()
        .manage(AppState { index })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search, 
            build_index, 
            start_usn_monitoring
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
