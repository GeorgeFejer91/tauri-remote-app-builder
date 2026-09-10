//! Transport-neutral remote-control authority starter.
//!
//! Authentication adapters must complete a reviewed PAKE or device-key proof
//! before calling `Authority::issue_grant`. This crate deliberately does not
//! implement cryptography, signaling, filesystem access, or Tauri commands.

mod authority;
mod protocol;
mod transfer;

pub use authority::{Authority, Grant};
pub use protocol::{AppAction, AppSnapshot, AppState, Applied, CommandRequest, Rejected};
pub use transfer::{decode_frame, encode_frame, BinaryChunkMetadata, FrameError};
