//! TCP transport implementation

use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

use crate::{error::*, Transport};

/// TCP transport for ZKTeco devices
///
/// Many ZKTeco devices require TCP packets to be wrapped with a header:
/// [0x5050][0x8272][length: 4 bytes LE] + [ZK packet]
pub struct TcpTransport {
    addr: String,
    port: u16,
    socket_addr: Option<SocketAddr>,
    stream: Option<TcpStream>,
    connect_timeout: Duration,
    read_timeout: Duration,
    use_tcp_wrapper: bool, // Enable TCP wrapper for F18 and similar devices
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
            use_tcp_wrapper: true, // Default: enabled (most devices need it)
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
    
    /// Enable/disable TCP wrapper
    pub fn with_tcp_wrapper(mut self, enabled: bool) -> Self {
        self.use_tcp_wrapper = enabled;
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
    
    /// Wrap data with TCP header
    fn wrap_tcp_packet(&self, data: &[u8]) -> BytesMut {
        let mut buf = BytesMut::with_capacity(8 + data.len());
        
        // Magic bytes
        buf.put_u16_le(0x5050);
        buf.put_u16_le(0x8272);
        
        // Payload length (4 bytes, little-endian)
        buf.put_u32_le(data.len() as u32);
        
        // Payload
        buf.put_slice(data);
        
        trace!(
            "Wrapped packet: {} bytes payload -> {} bytes total",
            data.len(),
            buf.len()
        );
        
        buf
    }
    
    /// Unwrap TCP header from received data
    fn unwrap_tcp_packet(&self, mut data: BytesMut) -> Result<BytesMut> {
        if data.len() < 8 {
            return Ok(data); // Not wrapped or incomplete
        }
        
        // Check for TCP wrapper magic
        let magic1 = u16::from_le_bytes([data[0], data[1]]);
        let magic2 = u16::from_le_bytes([data[2], data[3]]);
        
        if magic1 == 0x5050 && magic2 == 0x8272 {
            // Has TCP wrapper - skip 8-byte header
            let _length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            
            trace!("Unwrapped TCP packet: {} bytes header removed", 8);
            
            // Return data without header
            data.advance(8);
        }
        
        Ok(data)
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
        
        debug!(
            "Connected to {} (TCP wrapper: {})",
            addr,
            if self.use_tcp_wrapper { "enabled" } else { "disabled" }
        );
        
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
        // Wrap packet if needed (before getting mutable borrow of stream)
        let send_data = if self.use_tcp_wrapper {
            self.wrap_tcp_packet(data)
        } else {
            BytesMut::from(data)
        };

        trace!(
            "Sending {} bytes: {:02X?}",
            send_data.len(),
            &send_data[..send_data.len().min(32)]
        );

        // Get stream and send
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        stream.write_all(&send_data).await?;
        stream.flush().await?;

        Ok(())
    }
    
    async fn receive(&mut self, timeout_secs: u64) -> Result<BytesMut> {
        let timeout_duration = Duration::from_secs(timeout_secs);

        // Read data with timeout
        let mut buf = BytesMut::with_capacity(2048);

        // Limit scope of mutable borrow
        let n = {
            let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;

            // Read with timeout
            timeout(timeout_duration, stream.read_buf(&mut buf))
                .await
                .map_err(|_| {
                    warn!("Read timeout after {} seconds", timeout_secs);
                    Error::ReadTimeout
                })?
                .map_err(|e| {
                    warn!("Read error: {}", e);
                    Error::Io(e)
                })?
        };

        if n == 0 {
            warn!("Connection closed by remote (read 0 bytes)");
            return Err(Error::ConnectionClosed);
        }

        trace!(
            "Received {} bytes: {:02X?}",
            n,
            &buf[..n.min(32)]
        );

        // Unwrap TCP header if present
        if self.use_tcp_wrapper {
            self.unwrap_tcp_packet(buf)
        } else {
            Ok(buf)
        }
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
            // Don't warn in drop - normal if error occurred
            let _ = self.stream.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrap_tcp_packet() {
        let transport = TcpTransport::new("127.0.0.1", 4370);
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let wrapped = transport.wrap_tcp_packet(&data);
        
        // Check magic
        assert_eq!(wrapped[0], 0x50);
        assert_eq!(wrapped[1], 0x50);
        assert_eq!(wrapped[2], 0x72);
        assert_eq!(wrapped[3], 0x82);
        
        // Check length
        assert_eq!(u32::from_le_bytes([wrapped[4], wrapped[5], wrapped[6], wrapped[7]]), 4);
        
        // Check payload
        assert_eq!(&wrapped[8..], &data[..]);
    }
    
    #[test]
    fn test_unwrap_tcp_packet() {
        let transport = TcpTransport::new("127.0.0.1", 4370);
        
        // Create wrapped packet
        let mut data = BytesMut::new();
        data.put_u16_le(0x5050);
        data.put_u16_le(0x8272);
        data.put_u32_le(4);
        data.put_slice(&[0x01, 0x02, 0x03, 0x04]);
        
        let unwrapped = transport.unwrap_tcp_packet(data).unwrap();
        
        assert_eq!(unwrapped.as_ref(), &[0x01, 0x02, 0x03, 0x04]);
    }
    
    #[tokio::test]
    async fn test_tcp_transport_create() {
        let transport = TcpTransport::new("192.168.1.201", 4370);
        assert!(!transport.is_connected());
        assert!(transport.use_tcp_wrapper);
    }
    
    #[tokio::test]
    async fn test_tcp_transport_invalid_address() {
        let mut transport = TcpTransport::new("invalid..address", 4370)
            .with_connect_timeout(Duration::from_millis(100));
        
        let result = transport.connect().await;
        assert!(result.is_err());
    }
}