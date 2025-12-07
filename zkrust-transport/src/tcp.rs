//! TCP transport 

use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

use crate::{error::*, Transport};

/// TCP transport for ZKTeco devices
pub struct TcpTransport {
    addr: String,
    port: u16,
    socket_addr: Option<SocketAddr>,
    stream: Option<TcpStream>,
    connect_timeout: Duration,
    read_timeout: Duration,
}

impl TcpTransport {
    /// Create new TCP transport
    pub fn new(addr: impl Into<String>, port: u16) -> Self {
        Self {
            addr: addr.into(),
            port,
            socket_addr: None,
            stream: None,
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(5),
        }
    }
    
    /// Set connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
    
    /// Set read timeout
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }
    
    /// Resolve address to SocketAddr
    async fn resolve_addr(&mut self) -> Result<SocketAddr> {
        if let Some(addr) = self.socket_addr {
            return Ok(addr);
        }
        
        let addr_str = format!("{}:{}", self.addr, self.port);
        
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
            .await
            .map_err(|e| Error::InvalidAddress(format!("{}: {}", addr_str, e)))?
            .collect();
        
        let addr = addrs
            .first()
            .ok_or_else(|| Error::InvalidAddress(format!("No addresses found for {}", addr_str)))?;
        
        self.socket_addr = Some(*addr);
        Ok(*addr)
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Err(Error::AlreadyConnected);
        }
        
        let addr = self.resolve_addr().await?;
        
        debug!("Connecting to {}...", addr);
        
        let stream = timeout(self.connect_timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| Error::ConnectionTimeout)?
            .map_err(Error::Io)?;
        
        // Disable Nagle's algorithm for low latency
        stream.set_nodelay(true)?;
        
        debug!("Connected to {}", addr);
        
        self.stream = Some(stream);
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            debug!("Disconnecting from {}...", self.remote_addr());
            
            // Graceful shutdown
            let _ = stream.shutdown().await;
        }
        
        self.socket_addr = None;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    
    async fn send(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        
        trace!("Sending {} bytes: {:02X?}", data.len(), &data[..data.len().min(16)]);
        
        stream.write_all(data).await?;
        stream.flush().await?;
        
        Ok(())
    }
    
    async fn receive(&mut self, timeout_secs: u64) -> Result<BytesMut> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        
        let timeout_duration = Duration::from_secs(timeout_secs);
        
        // Read at least header (8 bytes)
        let mut buf = BytesMut::with_capacity(1024);
        
        // Read with timeout
        let n = timeout(timeout_duration, stream.read_buf(&mut buf))
            .await
            .map_err(|_| Error::ReadTimeout)?
            .map_err(Error::Io)?;
        
        if n == 0 {
            return Err(Error::ConnectionClosed);
        }
        
        trace!("Received {} bytes: {:02X?}", n, &buf[..n.min(16)]);
        
        Ok(buf)
    }
    
    fn remote_addr(&self) -> String {
        self.socket_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| format!("{}:{}", self.addr, self.port))
    }
}

impl Drop for TcpTransport {
    fn drop(&mut self) {
        if self.is_connected() {
            warn!("TCP transport dropped while still connected");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tcp_transport_create() {
        let transport = TcpTransport::new("192.168.1.201", 4370);
        assert!(!transport.is_connected());
    }
    
    #[tokio::test]
    async fn test_tcp_transport_invalid_address() {
        let mut transport = TcpTransport::new("invalid..address", 4370)
            .with_connect_timeout(Duration::from_millis(100));
        
        let result = transport.connect().await;
        assert!(result.is_err());
    }
    
    // Note: This test requires a real device at this IP
    // #[tokio::test]
    // async fn test_tcp_transport_connect() {
    //     let mut transport = TcpTransport::new("192.168.1.201", 4370);
    //     transport.connect().await.unwrap();
    //     assert!(transport.is_connected());
    //     transport.disconnect().await.unwrap();
    //     assert!(!transport.is_connected());
    // }
}