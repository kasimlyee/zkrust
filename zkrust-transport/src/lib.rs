//! Transport layer for ZKTeco protocol
//!
//! Provides TCP/UDP communication with devices.

pub mod tcp;
pub mod udp;
pub mod error;

pub use error::{Error, Result};
pub use tcp::TcpTransport;
pub use udp::UdpTransport;

use async_trait::async_trait;
use bytes::BytesMut;

/// Transport trait for different communication methods
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to device
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from device
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
    
    /// Send raw bytes
    async fn send(&mut self, data: &[u8]) -> Result<()>;
    
    /// Receive raw bytes (with timeout)
    async fn receive(&mut self, timeout_secs: u64) -> Result<BytesMut>;
    
    /// Get remote address
    fn remote_addr(&self) -> String;
}