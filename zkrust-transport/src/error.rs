//! Transport errors

use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not connected")]
    NotConnected,
    
    #[error("Already connected")]
    AlreadyConnected,
    
    #[error("Connection timeout")]
    ConnectionTimeout,
    
    #[error("Read timeout")]
    ReadTimeout,
    
    #[error("Connection closed by remote")]
    ConnectionClosed,
    
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}