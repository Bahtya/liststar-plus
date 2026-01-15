mod index;
mod ipc;
mod model;
mod search;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use index::MemoryIndex;
use ipc::{NamedPipeServer, RequestHandler};
use ipc::protocol::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting listory-plus search daemon v{}", env!("CARGO_PKG_VERSION"));

    // Create shared memory index
    let index = Arc::new(RwLock::new(MemoryIndex::new()));

    // Create request handler
    let handler = Arc::new(RequestHandler::new(index.clone()));

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
        // Read length prefix (4 bytes)
        let mut length_buf = [0u8; 4];
        let bytes_read = server.read(&mut length_buf)?;

        if bytes_read == 0 {
            log::info!("Client disconnected");
            break;
        }

        if bytes_read < 4 {
            log::warn!("Incomplete length prefix received");
            break;
        }

        let msg_length = u32::from_le_bytes(length_buf) as usize;

        // Read message payload
        let mut msg_buf = vec![0u8; msg_length];
        let bytes_read = server.read(&mut msg_buf)?;

        if bytes_read < msg_length {
            log::warn!("Incomplete message received");
            break;
        }

        // Try to decode as different request types
        // In a real implementation, we'd have a message type field
        // For MVP, we'll try each type in order

        // Try Ping
        if let Ok(req) = prost::Message::decode(&msg_buf[..]) {
            let req: PingReq = req;
            log::debug!("Received Ping request");
            let resp = handler.handle_ping(req).await?;
            send_response(server, &resp)?;
            continue;
        }

        // Try BuildIndex
        if let Ok(req) = prost::Message::decode(&msg_buf[..]) {
            let req: BuildIndexReq = req;
            log::debug!("Received BuildIndex request");
            let resp = handler.handle_build_index(req).await?;
            send_response(server, &resp)?;
            continue;
        }

        // Try Search
        if let Ok(req) = prost::Message::decode(&msg_buf[..]) {
            let req: SearchReq = req;
            log::debug!("Received Search request");
            let resp = handler.handle_search(req).await?;
            send_response(server, &resp)?;
            continue;
        }

        log::warn!("Unknown message type received");
    }

    Ok(())
}

/// Send a response message
fn send_response<M: prost::Message>(server: &NamedPipeServer, msg: &M) -> Result<()> {
    let encoded = ipc::protocol::encode_message(msg)?;
    server.write(&encoded)?;
    Ok(())
}
