//! High-level device interface

use std::time::Duration;

use bytes::{Bytes};
use tracing::{debug, info, trace, warn};

use zkrust_core::{make_commkey, Command, Packet, Session};
use zkrust_transport::{TcpTransport, UdpTransport, Transport};
use zkrust_types::DeviceInfo;

use crate::error::{Error, Result};

/// ZKTeco device
///
/// High-level interface for communicating with ZKTeco biometric devices.
///
/// # Examples
///
/// ```no_run
/// use zkrust::Device;
///
/// #[tokio::main]
/// async fn main() -> zkrust::Result<()> {
///     let mut device = Device::new("192.168.1.201", 4370);
///     
///     device.connect().await?;
///     println!("Connected!");
///     
///     let info = device.get_device_info().await?;
///     println!("Device: {}", info);
///     
///     device.disconnect().await?;
///     Ok(())
/// }
/// ```
pub struct Device {
    transport: Box<dyn Transport>,
    session: Session,
    timeout: Duration,
    password: u32, // CommKey password (default: 0)
}

impl Device {
    /// Create a new device instance (TCP transport)
    pub fn new(ip: impl Into<String>, port: u16) -> Self {
        Self {
            transport: Box::new(TcpTransport::new(ip, port).with_tcp_wrapper(false)),
            session: Session::new(),
            timeout: Duration::from_secs(5),
            password: 0, // Default CommKey password
        }
    }

    /// Create a new device instance using UDP transport (recommended)
    ///
    /// Most ZKTeco devices use UDP protocol. This is the recommended method.
    pub fn new_udp(ip: impl Into<String>, port: u16) -> Self {
        Self {
            transport: Box::new(UdpTransport::new(ip, port)),
            session: Session::new(),
            timeout: Duration::from_secs(5),
            password: 0, // Default CommKey password
        }
    }

    /// Set command timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set CommKey password (default: 0)
    pub fn with_password(mut self, password: u32) -> Self {
        self.password = password;
        self
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.session.is_connected() && self.transport.is_connected()
    }
    
    /// Connect to device
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Network connection fails
    /// - Device doesn't respond
    /// - Authentication required but not provided
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to {}...", self.transport.remote_addr());
        
        // Establish TCP connection
        self.transport.connect().await?;
        
        // Send CMD_CONNECT
        let packet = Packet::new(Command::Connect, 0, 0);
        self.send_packet(&packet).await?;
        
        // Receive response
        let response = self.receive_packet().await?;
        
        match response.command {
            Command::AckOk => {
                // Success - initialize session
                let session_id = response.session_id;
                self.session.initialize(session_id)?;

                info!(
                    "Connected successfully (session_id={})",
                    session_id
                );

                Ok(())
            }
            Command::AckUnauth => {
                // Device requires authentication
                info!("Device requires authentication, sending password...");

                // Use the session_id from the AckUnauth response
                let session_id = response.session_id;

                // Generate authentication key using ZKTeco's proprietary algorithm
                let auth_key = make_commkey(self.password, session_id, 50);

                debug!(
                    "Auth key: {:02X?} (password={}, session_id={})",
                    auth_key, self.password, session_id
                );

                // Send CMD_AUTH with scrambled password
                let auth_packet = Packet::with_payload(
                    Command::Auth,
                    session_id,
                    0,
                    auth_key,
                );

                self.send_packet(&auth_packet).await?;

                // Receive authentication response
                let auth_response = self.receive_packet().await?;

                match auth_response.command {
                    Command::AckOk => {
                        // Authentication successful - initialize session
                        let session_id = auth_response.session_id;
                        self.session.initialize(session_id)?;

                        info!(
                            "Authenticated successfully (session_id={})",
                            session_id
                        );

                        Ok(())
                    }
                    Command::AckError => {
                        Err(Error::InvalidResponse("Authentication failed - incorrect password".into()))
                    }
                    _ => Err(Error::InvalidResponse(format!(
                        "Unexpected auth response: {}",
                        auth_response.command
                    ))),
                }
            }
            Command::AckError => {
                Err(Error::InvalidResponse("Device returned error".into()))
            }
            _ => Err(Error::InvalidResponse(format!(
                "Unexpected response: {}",
                response.command
            ))),
        }
    }
    
    /// Disconnect from device
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected() {
            return Ok(());
        }
        
        info!("Disconnecting from {}...", self.transport.remote_addr());
        
        // Send CMD_EXIT
        let packet = self.create_packet(Command::Exit, Bytes::new());
        if let Err(e) = self.send_packet(&packet).await {
            warn!("Failed to send EXIT command: {}", e);
        }
        
        // Close transport
        self.transport.disconnect().await?;
        self.session.close();
        
        info!("Disconnected");
        Ok(())
    }
    
    /// Get device information
    ///
    /// Retrieves device serial number, firmware version, etc.
    pub async fn get_device_info(&mut self) -> Result<DeviceInfo> {
        self.ensure_connected()?;
        
        debug!("Getting device info...");
        
        // Send CMD_GET_VERSION
        let packet = self.create_packet(Command::GetVersion, Bytes::new());
        self.send_packet(&packet).await?;
        
        let response = self.receive_packet().await?;
        
        if !response.is_success() {
            return Err(Error::InvalidResponse("Failed to get version".into()));
        }
        
        // Parse firmware version from payload
        let firmware_version = String::from_utf8_lossy(&response.payload).to_string();
        
        // For now, use dummy serial number - we'll implement full parsing in Phase 2
        let serial_number = "UNKNOWN".to_string();
        
        let info = DeviceInfo::new(serial_number, firmware_version);
        
        debug!("Device info: {}", info);
        
        Ok(info)
    }
    
    /// Enable device (normal operation mode)
    pub async fn enable_device(&mut self) -> Result<()> {
        self.ensure_connected()?;
        
        debug!("Enabling device...");
        
        let packet = self.create_packet(Command::EnableDevice, Bytes::new());
        self.send_packet(&packet).await?;
        
        let response = self.receive_packet().await?;
        
        if response.is_success() {
            debug!("Device enabled");
            Ok(())
        } else {
            Err(Error::InvalidResponse("Failed to enable device".into()))
        }
    }
    
    /// Disable device (show "Working..." on LCD)
    pub async fn disable_device(&mut self) -> Result<()> {
        self.ensure_connected()?;
        
        debug!("Disabling device...");
        
        let packet = self.create_packet(Command::DisableDevice, Bytes::new());
        self.send_packet(&packet).await?;
        
        let response = self.receive_packet().await?;
        
        if response.is_success() {
            debug!("Device disabled");
            Ok(())
        } else {
            Err(Error::InvalidResponse("Failed to disable device".into()))
        }
    }
    
    /// Restart device
    pub async fn restart(&mut self) -> Result<()> {
        self.ensure_connected()?;
        
        warn!("Restarting device...");
        
        let packet = self.create_packet(Command::Restart, Bytes::new());
        self.send_packet(&packet).await?;
        
        // Device will disconnect after restart
        self.session.close();
        
        Ok(())
    }
    
    /// Power off device
    pub async fn power_off(&mut self) -> Result<()> {
        self.ensure_connected()?;
        
        warn!("Powering off device...");
        
        let packet = self.create_packet(Command::PowerOff, Bytes::new());
        self.send_packet(&packet).await?;
        
        // Device will disconnect after power off
        self.session.close();
        
        Ok(())
    }
    
    // Helper methods
    
    fn ensure_connected(&self) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::NotConnected);
        }
        Ok(())
    }
    
    fn create_packet(&self, command: Command, payload: Bytes) -> Packet {
        Packet::with_payload(
            command,
            self.session.session_id(),
            self.session.next_reply_id(),
            payload,
        )
    }
    
    async fn send_packet(&mut self, packet: &Packet) -> Result<()> {
        trace!("Sending: {:?}", packet);
        
        let data = packet.encode();
        self.transport.send(&data).await?;
        
        Ok(())
    }
    
    async fn receive_packet(&mut self) -> Result<Packet> {
        let buf = self.transport.receive(self.timeout.as_secs()).await?;
        
        let packet = Packet::decode(buf)?;
        
        trace!("Received: {:?}", packet);
        
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_create() {
        let device = Device::new("192.168.1.201", 4370);
        assert!(!device.is_connected());
    }
    
    // Integration tests require real device
    // Run with: cargo test --features integration-tests
    
    #[tokio::test]
    #[ignore] // Only run with real device
    async fn test_device_connect() {
        let mut device = Device::new("192.168.1.201", 4370);
        
        device.connect().await.unwrap();
        assert!(device.is_connected());
        
        device.disconnect().await.unwrap();
        assert!(!device.is_connected());
    }
    
    #[tokio::test]
    #[ignore] // Only run with real device
    async fn test_device_get_info() {
        let mut device = Device::new("192.168.1.201", 4370);
        device.connect().await.unwrap();
        
        let info = device.get_device_info().await.unwrap();
        println!("{:?}", info);
        
        device.disconnect().await.unwrap();
    }
}