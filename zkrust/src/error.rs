//! High-level error types

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Core protocol error: {0}")]
    Core(#[from] zkrust_core::Error),
    
    #[error("Transport error: {0}")]
    Transport(#[from] zkrust_transport::Error),
    
    #[error("Type error: {0}")]
    Types(#[from] zkrust_types::Error),
    
    #[error("Device not connected")]
    NotConnected,
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Invalid response from device: {0}")]
    InvalidResponse(String),
}