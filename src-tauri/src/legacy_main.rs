mod index;
mod ipc;
mod model;
mod search;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use index::MemoryIndex;
use ipc::{NamedPipeServer, RequestHandler, UsnControlHandle};
use ipc::protocol::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting listory-plus search daemon v{}", env!("CARGO_PKG_VERSION"));

    // Create shared memory index
    let index = Arc::new(RwLock::new(MemoryIndex::new()));

    // Create USN monitoring control handle
    let usn_handle: UsnControlHandle = Arc::new(RwLock::new(None));

    // Create request handler
    let handler = Arc::new(RequestHandler::new(index.clone(), usn_handle.clone()));

    // Start USN monitoring task (optional, can be enabled after initial indexing)
    // Uncomment to enable real-time file monitoring:
    // let index_clone = index.clone();
    // tokio::spawn(async move {
    //     if let Err(e) = start_usn_monitoring(index_clone, 'C').await {
    //         log::error!("USN monitoring failed: {}", e);
    //     }
    // });

    // Create Named Pipe server
    let server = NamedPipeServer::new()?;
    log::info!("Named Pipe server started, waiting for connections...");

    // Main server loop
    loop {
        // Wait for client connection
        if let Err(e) = server.accept() {
            log::error!("Failed to accept client: {}", e);
            continue;
        }

        // Handle client requests
        if let Err(e) = handle_client(&server, handler.clone()).await {
            log::error!("Error handling client: {}", e);
        }

        // Disconnect client
        if let Err(e) = server.disconnect() {
            log::error!("Failed to disconnect client: {}", e);
        }
    }
}

/// Handle a single client connection
async fn handle_client(server: &NamedPipeServer, handler: Arc<RequestHandler>) -> Result<()> {
    loop {
        // Read message type (1 byte)
        let mut type_buf = [0u8; 1];
        log::debug!("Waiting to read message type...");

        let bytes_read = match server.read(&mut type_buf) {
            Ok(n) => {
                log::debug!("Read {} bytes for message type", n);
                n
            }
            Err(e) => {
                log::error!("Failed to read message type: {}", e);
                return Err(e);
            }
        };

        if bytes_read == 0 {
            log::info!("Client disconnected");
            break;
        }

        if bytes_read < 1 {
            log::warn!("Incomplete message type received");
            break;
        }

        let msg_type = type_buf[0];
        log::debug!("Message type: {}", msg_type);

        // Read length prefix (4 bytes)
        let mut length_buf = [0u8; 4];
        log::debug!("Waiting to read length prefix...");

        let bytes_read = match server.read(&mut length_buf) {
            Ok(n) => {
                log::debug!("Read {} bytes for length prefix", n);
                n
            }
            Err(e) => {
                log::error!("Failed to read length prefix: {}", e);
                return Err(e);
            }
        };

        if bytes_read == 0 {
            log::info!("Client disconnected");
            break;
        }

        if bytes_read < 4 {
            log::warn!("Incomplete length prefix received: {} bytes", bytes_read);
            break;
        }

        let msg_length = u32::from_le_bytes(length_buf) as usize;
        log::debug!("Message length: {} bytes", msg_length);

        // Read message payload (if any)
        let msg_buf = if msg_length > 0 {
            let mut buf = vec![0u8; msg_length];
            let bytes_read = match server.read(&mut buf) {
                Ok(n) => {
                    log::debug!("Read {} bytes for payload", n);
                    n
                }
                Err(e) => {
                    log::error!("Failed to read payload: {}", e);
                    return Err(e);
                }
            };

            if bytes_read < msg_length {
                log::warn!("Incomplete message received: expected {}, got {}", msg_length, bytes_read);
                break;
            }
            buf
        } else {
            log::debug!("Empty message (length = 0)");
            vec![]
        };

        // Decode and handle based on message type
        match msg_type {
            0 => {
                // Ping
                log::debug!("Received Ping request");
                let req: PingReq = prost::Message::decode(&msg_buf[..])?;
                let resp = handler.handle_ping(req).await?;
                send_response(server, &resp)?;
            }
            1 => {
                // BuildIndex
                log::debug!("Received BuildIndex request");
                let req: BuildIndexReq = prost::Message::decode(&msg_buf[..])?;
                let resp = handler.handle_build_index(req).await?;
                send_response(server, &resp)?;
            }
            2 => {
                // Search
                log::debug!("Received Search request");
                let req: SearchReq = prost::Message::decode(&msg_buf[..])?;
                let resp = handler.handle_search(req).await?;
                send_response(server, &resp)?;
            }
            3 => {
                // StartUsnMonitoring
                log::debug!("Received StartUsnMonitoring request");
                let req: StartUsnMonitoringReq = prost::Message::decode(&msg_buf[..])?;
                let resp = handler.handle_start_usn_monitoring(req).await?;
                send_response(server, &resp)?;
            }
            4 => {
                // StopUsnMonitoring
                log::debug!("Received StopUsnMonitoring request");
                let req: StopUsnMonitoringReq = prost::Message::decode(&msg_buf[..])?;
                let resp = handler.handle_stop_usn_monitoring(req).await?;
                send_response(server, &resp)?;
            }
            _ => {
                log::warn!("Unknown message type: {}", msg_type);
                break;
            }
        }
    }

    Ok(())
}

/// Send a response message
fn send_response<M: prost::Message>(server: &NamedPipeServer, msg: &M) -> Result<()> {
    let encoded = ipc::protocol::encode_message(msg)?;
    server.write(&encoded)?;
    Ok(())
}

/// Start USN journal monitoring for a drive
/// This function runs in a background task and monitors file system changes
#[allow(dead_code)]
async fn start_usn_monitoring(index: Arc<RwLock<MemoryIndex>>, drive_letter: char) -> Result<()> {
    use std::sync::atomic::AtomicBool;

    log::info!("Starting USN monitoring for drive {}:", drive_letter);

    // Get volume handle
    let volume_handle = index::get_volume_handle(drive_letter)?;

    // Create stop flag (this version doesn't support stopping)
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Create USN monitor
    let mut monitor = index::UsnMonitor::new(volume_handle, index, stop_flag)?;

    // Start monitoring loop
    monitor.start_monitoring().await?;

    Ok(())
}
