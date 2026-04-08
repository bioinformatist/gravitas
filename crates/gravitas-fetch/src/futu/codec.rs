//! Futu OpenD binary protocol header codec.
//!
//! 44-byte fixed header, all multi-byte integers little-endian.
//!
//! | Field         | Offset | Size | Type   |
//! |---------------|--------|------|--------|
//! | Magic "FT"    | 0      | 2    | ASCII  |
//! | proto_id      | 2      | 4    | u32 LE |
//! | proto_fmt     | 6      | 1    | u8     |
//! | proto_ver     | 7      | 1    | u8     |
//! | serial_no     | 8      | 4    | u32 LE |
//! | body_len      | 12     | 4    | u32 LE |
//! | body_sha1     | 16     | 20   | bytes  |
//! | reserved      | 36     | 8    | zeros  |

use sha1::{Digest, Sha1};

pub const HEADER_SIZE: usize = 44;
const MAGIC: &[u8; 2] = b"FT";
const PROTO_FMT_PROTOBUF: u8 = 0;
const PROTO_VER: u8 = 0;

#[derive(Debug, Clone)]
pub struct FutuHeader {
    pub proto_id: u32,
    pub serial_no: u32,
    pub body_len: u32,
}

/// Encode a full packet: 44-byte header + body.
pub fn encode_packet(proto_id: u32, serial_no: u32, body: &[u8]) -> Vec<u8> {
    let body_len = body.len() as u32;
    let sha1_hash = Sha1::digest(body);

    let mut buf = vec![0u8; HEADER_SIZE + body.len()];

    // Magic
    buf[0..2].copy_from_slice(MAGIC);
    // proto_id
    buf[2..6].copy_from_slice(&proto_id.to_le_bytes());
    // proto_fmt
    buf[6] = PROTO_FMT_PROTOBUF;
    // proto_ver
    buf[7] = PROTO_VER;
    // serial_no
    buf[8..12].copy_from_slice(&serial_no.to_le_bytes());
    // body_len
    buf[12..16].copy_from_slice(&body_len.to_le_bytes());
    // sha1
    buf[16..36].copy_from_slice(&sha1_hash);
    // reserved (already zeros)
    // body
    buf[HEADER_SIZE..].copy_from_slice(body);

    buf
}

/// Decode a 44-byte header from a buffer.
pub fn decode_header(buf: &[u8; HEADER_SIZE]) -> Result<FutuHeader, CodecError> {
    // Verify magic
    if &buf[0..2] != MAGIC {
        return Err(CodecError::InvalidMagic);
    }

    let proto_id = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
    let serial_no = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    let body_len = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);

    Ok(FutuHeader {
        proto_id,
        serial_no,
        body_len,
    })
}

/// Verify SHA1 hash of a received body against the header.
pub fn verify_sha1(header_buf: &[u8; HEADER_SIZE], body: &[u8]) -> bool {
    let expected = &header_buf[16..36];
    let actual = Sha1::digest(body);
    expected == actual.as_slice()
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("invalid header magic (expected 'FT')")]
    InvalidMagic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let body = b"hello futu";
        let packet = encode_packet(1001, 42, body);

        assert_eq!(packet.len(), HEADER_SIZE + body.len());
        assert_eq!(&packet[0..2], b"FT");

        let mut header_buf = [0u8; HEADER_SIZE];
        header_buf.copy_from_slice(&packet[..HEADER_SIZE]);
        let header = decode_header(&header_buf).unwrap();

        assert_eq!(header.proto_id, 1001);
        assert_eq!(header.serial_no, 42);
        assert_eq!(header.body_len, body.len() as u32);

        assert!(verify_sha1(&header_buf, &packet[HEADER_SIZE..]));
    }
}
