use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 4] = b"RBK1";
const HEADER_BYTES: usize = 8;
const MAX_METADATA_BYTES: usize = 4096;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinaryChunkMetadata {
    #[serde(rename = "type")]
    pub message_type: String,
    pub authority_generation: String,
    pub grant_id: String,
    pub principal_id: String,
    pub sequence: u64,
    pub transfer_id: String,
    pub file_id: String,
    pub offset: u64,
    pub payload_length: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    InvalidSize,
    InvalidMagic,
    InvalidMetadata,
    InvalidPayload,
    ExceedsNegotiatedMaximum,
}

pub fn encode_frame(
    mut metadata: BinaryChunkMetadata,
    payload: &[u8],
    negotiated_maximum: usize,
) -> Result<Vec<u8>, FrameError> {
    if payload.is_empty() || payload.len() > MAX_PAYLOAD_BYTES {
        return Err(FrameError::InvalidPayload);
    }
    metadata.payload_length =
        u32::try_from(payload.len()).map_err(|_| FrameError::InvalidPayload)?;
    validate_metadata(&metadata)?;
    let encoded = serde_json::to_vec(&metadata).map_err(|_| FrameError::InvalidMetadata)?;
    if encoded.len() < 2 || encoded.len() > MAX_METADATA_BYTES {
        return Err(FrameError::InvalidMetadata);
    }
    let total = HEADER_BYTES
        .checked_add(encoded.len())
        .and_then(|value| value.checked_add(payload.len()))
        .ok_or(FrameError::InvalidSize)?;
    if total > negotiated_maximum {
        return Err(FrameError::ExceedsNegotiatedMaximum);
    }
    let mut frame = Vec::with_capacity(total);
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
    frame.extend_from_slice(&encoded);
    frame.extend_from_slice(payload);
    Ok(frame)
}

pub fn decode_frame(
    frame: &[u8],
    negotiated_maximum: usize,
) -> Result<(BinaryChunkMetadata, &[u8]), FrameError> {
    if frame.len() < HEADER_BYTES + 3 || frame.len() > negotiated_maximum {
        return Err(if frame.len() > negotiated_maximum {
            FrameError::ExceedsNegotiatedMaximum
        } else {
            FrameError::InvalidSize
        });
    }
    if &frame[..4] != MAGIC {
        return Err(FrameError::InvalidMagic);
    }
    let metadata_len = u32::from_be_bytes(
        frame[4..8]
            .try_into()
            .map_err(|_| FrameError::InvalidSize)?,
    ) as usize;
    if !(2..=MAX_METADATA_BYTES).contains(&metadata_len) {
        return Err(FrameError::InvalidMetadata);
    }
    let payload_start = HEADER_BYTES
        .checked_add(metadata_len)
        .ok_or(FrameError::InvalidSize)?;
    if payload_start >= frame.len() {
        return Err(FrameError::InvalidPayload);
    }
    let metadata: BinaryChunkMetadata = serde_json::from_slice(&frame[HEADER_BYTES..payload_start])
        .map_err(|_| FrameError::InvalidMetadata)?;
    validate_metadata(&metadata)?;
    let payload = &frame[payload_start..];
    if payload.len() > MAX_PAYLOAD_BYTES || metadata.payload_length as usize != payload.len() {
        return Err(FrameError::InvalidPayload);
    }
    Ok((metadata, payload))
}

fn validate_metadata(metadata: &BinaryChunkMetadata) -> Result<(), FrameError> {
    if metadata.message_type != "example.bulk.chunk.v1"
        || metadata.payload_length == 0
        || metadata.payload_length as usize > MAX_PAYLOAD_BYTES
        || [
            &metadata.authority_generation,
            &metadata.grant_id,
            &metadata.principal_id,
            &metadata.transfer_id,
            &metadata.file_id,
        ]
        .into_iter()
        .any(|value| !valid_id(value))
    {
        return Err(FrameError::InvalidMetadata);
    }
    Ok(())
}

fn valid_id(value: &str) -> bool {
    (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> BinaryChunkMetadata {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../../contracts/command-fixtures.json"))
                .expect("fixture JSON");
        serde_json::from_value(fixtures["validBinaryMetadata"].clone()).expect("metadata fixture")
    }

    #[test]
    fn round_trips_a_bounded_binary_frame() {
        let frame = encode_frame(metadata(), b"test", 65_536).expect("encode");
        let (decoded, payload) = decode_frame(&frame, 65_536).expect("decode");
        assert_eq!(decoded, metadata());
        assert_eq!(payload, b"test");
    }

    #[test]
    fn rejects_tampered_lengths_and_negotiated_overflow() {
        let mut frame = encode_frame(metadata(), b"test", 65_536).expect("encode");
        *frame.last_mut().expect("payload") = b'!';
        assert_eq!(
            decode_frame(&frame, frame.len() - 1),
            Err(FrameError::ExceedsNegotiatedMaximum)
        );

        let mut bad_length = frame;
        let end = bad_length.len();
        bad_length.truncate(end - 1);
        assert_eq!(
            decode_frame(&bad_length, 65_536),
            Err(FrameError::InvalidPayload)
        );
    }

    #[test]
    fn rejects_unknown_metadata_fields() {
        let mut value = serde_json::to_value(metadata()).expect("metadata value");
        value["nativePath"] = serde_json::Value::String("/forbidden".into());
        let encoded = serde_json::to_vec(&value).expect("metadata JSON");
        let mut frame = Vec::new();
        frame.extend_from_slice(MAGIC);
        frame.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
        frame.extend_from_slice(&encoded);
        frame.extend_from_slice(b"test");
        assert_eq!(
            decode_frame(&frame, 65_536),
            Err(FrameError::InvalidMetadata)
        );
    }
}
