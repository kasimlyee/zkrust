//! UDP transport for ZKTeco devices
//!
//! Most ZKTeco devices use UDP protocol on port 4370.
//! The packet format is the same as TCP 

use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use bytes::BytesMut;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

use crate::{error::*, Transport};

/// UDP transport for ZKTeco devices
///
/// This is the most common transport method for ZKTeco devices.
/// Uses standard UDP datagrams on port 4370.
pub struct UdpTransport {
    addr: String,
    port: u16,
    socket: Option<UdpSocket>,
    remote_addr: Option<SocketAddr>,
    connect_timeout: Duration,
    read_timeout: Duration,
}

impl UdpTransport {
    /// Create new UDP transport
    pub fn new(addr: impl Into<String>, port: u16) -> Self {
        Self {
            addr: addr.into(),
            port,
            socket: None,
            remote_addr: None,
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
        if let Some(addr) = self.remote_addr {
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

        self.remote_addr = Some(*addr);
        Ok(*addr)
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Err(Error::AlreadyConnected);
        }

        let remote = self.resolve_addr().await?;

        debug!("Connecting to {} via UDP...", remote);

        // Bind to any available local port
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(Error::Io)?;

        // Connect to remote address (sets default send/recv target)
        socket.connect(remote).await.map_err(Error::Io)?;

        debug!("Connected to {} via UDP", remote);

        self.socket = Some(socket);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(_socket) = self.socket.take() {
            debug!("Disconnecting from {}...", self.remote_addr());
        }

        self.remote_addr = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    async fn send(&mut self, data: &[u8]) -> Result<()> {
        let socket = self.socket.as_ref().ok_or(Error::NotConnected)?;

        trace!(
            "Sending {} bytes via UDP: {:02X?}",
            data.len(),
            &data[..data.len().min(32)]
        );

        socket.send(data).await.map_err(Error::Io)?;

        Ok(())
    }

    async fn receive(&mut self, timeout_secs: u64) -> Result<BytesMut> {
        let socket = self.socket.as_ref().ok_or(Error::NotConnected)?;

        let timeout_duration = Duration::from_secs(timeout_secs);

        // Read UDP datagram
        let mut buf = BytesMut::with_capacity(2048);
        buf.resize(2048, 0);

        let n = timeout(timeout_duration, socket.recv(&mut buf))
            .await
            .map_err(|_| {
                warn!("Read timeout after {} seconds", timeout_secs);
                Error::ReadTimeout
            })?
            .map_err(|e| {
                warn!("Read error: {}", e);
                Error::Io(e)
            })?;

        if n == 0 {
            warn!("Received 0 bytes");
            return Err(Error::ConnectionClosed);
        }

        // Truncate to actual received size
        buf.truncate(n);

        trace!(
            "Received {} bytes via UDP: {:02X?}",
            n,
            &buf[..n.min(32)]
        );

        Ok(buf)
    }

    fn remote_addr(&self) -> String {
        self.remote_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| format!("{}:{}", self.addr, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_transport_create() {
        let transport = UdpTransport::new("192.168.1.201", 4370);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_udp_transport_invalid_address() {
        let mut transport = UdpTransport::new("invalid..address", 4370)
            .with_connect_timeout(Duration::from_millis(100));

        let result = transport.connect().await;
        assert!(result.is_err());
    }
}
