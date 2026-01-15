use anyhow::{Context, Result};
use bytes::{Buf, BufMut, BytesMut};
use prost::Message;

// Include generated protobuf code
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/search.ipc.rs"));
}

pub use proto::*;

/// Encode a protobuf message with length prefix (4 bytes little-endian)
pub fn encode_message<M: Message>(msg: &M) -> Result<Vec<u8>> {
    let mut buf = BytesMut::new();

    // Encode the message to get its size
    let encoded = msg.encode_to_vec();
    let len = encoded.len() as u32;

    // Write length prefix (4 bytes, little-endian)
    buf.put_u32_le(len);

    // Write the protobuf payload
    buf.put_slice(&encoded);

    Ok(buf.to_vec())
}

/// Decode a length-prefixed protobuf message
pub fn decode_message<M: Message + Default>(data: &[u8]) -> Result<M> {
    if data.len() < 4 {
        anyhow::bail!("Data too short for length prefix");
    }

    let mut buf = &data[..];
    let len = buf.get_u32_le() as usize;

    if buf.len() < len {
        anyhow::bail!("Incomplete message: expected {} bytes, got {}", len, buf.len());
    }

    let msg = M::decode(&buf[..len])
        .context("Failed to decode protobuf message")?;

    Ok(msg)
}

/// Read length prefix from buffer
pub fn read_length_prefix(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        return None;
    }

    let mut buf = &data[..4];
    Some(buf.get_u32_le())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let req = PingReq {};
        let encoded = encode_message(&req).unwrap();

        // Check length prefix
        assert_eq!(encoded.len(), 4); // Just the length prefix for empty message

        let decoded: PingReq = decode_message(&encoded).unwrap();
        assert_eq!(req, decoded);
    }
}
