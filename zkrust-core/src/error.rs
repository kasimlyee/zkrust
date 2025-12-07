//! Error types for zkrust-core



/// Result type alias for zkrust operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core protocol errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Packet is too short to be valid
    #[error("Packet too short: expected at least {expected} bytes, got {actual} bytes")]
    PacketTooShort {
        expected: usize,
        actual: usize,
    },
    
    /// Checksum verification failed
    #[error("Checksum mismatch: expected 0x{expected:04X}, received 0x{received:04X}")]
    ChecksumMismatch {
        expected: u16,
        received: u16,
    },
    
    /// Unknown command code
    #[error("Unknown command code: {0}")]
    UnknownCommand(u16),
    
    /// Invalid session state
    #[error("Invalid session state: {0}")]
    InvalidSessionState(String),
    
    /// Session not initialized
    #[error("Session not initialized - connect to device first")]
    SessionNotInitialized,
    
    /// Device returned error response
    #[error("Device returned error: {command}")]
    DeviceError {
        command: crate::command::Command,
    },
    
    /// Authentication required
    #[error("Authentication required - device has CommKey set")]
    AuthenticationRequired,
    
    /// Authentication failed
    #[error("Authentication failed - invalid password")]
    AuthenticationFailed,
    
    /// Timeout waiting for response
    #[error("Timeout waiting for response after {seconds}s")]
    Timeout {
        seconds: u64,
    },
    
    /// Payload too large
    #[error("Payload too large: {size} bytes (max: {max} bytes)")]
    PayloadTooLarge {
        size: usize,
        max: usize,
    },
    
    /// Invalid reply ID
    #[error("Invalid reply ID: expected {expected}, got {actual}")]
    InvalidReplyId {
        expected: u16,
        actual: u16,
    },
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Check if error is recoverable (retry might succeed)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. } 
                | Self::Io(_)
                | Self::DeviceError { .. }
        )
    }
    
    /// Check if error requires reconnection
    pub fn requires_reconnect(&self) -> bool {
        matches!(
            self,
            Self::SessionNotInitialized
                | Self::InvalidSessionState(_)
                | Self::Io(_)
        )
    }
}