//! # zkrust-core
//!
//! Core protocol implementation for ZKTeco biometric devices.
//!
//! This crate provides the low-level protocol primitives:
//! - Packet structure and encoding/decoding
//! - Checksum calculation
//! - Command definitions
//! - Protocol constants

pub mod checksum;
pub mod command;
pub mod constants;
pub mod error;
pub mod packet;
pub mod session;

pub use command::Command;
pub use error::{Error, Result};
pub use packet::Packet;
pub use session::Session;

/// Protocol version information
pub const PROTOCOL_VERSION: &str = "1.0";

/// Default device port
pub const DEFAULT_PORT: u16 = 4370;

/// Maximum packet size (64KB)
pub const MAX_PACKET_SIZE: usize = 65535;

/// Packet header size
pub const HEADER_SIZE: usize = 8;