//! ZKTeco protocol packet structure and encoding/decoding

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt;

use crate::{
    checksum,
    command::Command,
    error::{Error, Result},
};

/// ZKTeco protocol packet
///
/// # Packet Structure
///
/// ```text
/// ┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
/// │   Command   │  Checksum   │  SessionID  │  ReplyID    │   Payload   │
/// │   2 bytes   │   2 bytes   │   2 bytes   │   2 bytes   │   N bytes   │
/// │ (LE u16)    │  (LE u16)   │  (LE u16)   │  (LE u16)   │   (bytes)   │
/// └─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
/// ```
///
/// All multi-byte values are in little-endian format.
///
/// # Examples
///
/// ```
/// use zkrust_core::{Packet, Command};
/// use bytes::BytesMut;
///
/// // Create a connection packet
/// let packet = Packet::new(Command::Connect, 0, 0);
/// let encoded = packet.encode();
///
/// // Decode it back
/// let decoded = Packet::decode(encoded).unwrap();
/// assert_eq!(packet.command, decoded.command);
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Packet {
    /// Command code
    pub command: Command,
    
    /// Session identifier (assigned by device on connect)
    pub session_id: u16,
    
    /// Reply number (increments per command in session)
    pub reply_id: u16,
    
    /// Packet payload (command-specific data)
    pub payload: Bytes,
}

impl Packet {
    /// Packet header size in bytes
    pub const HEADER_SIZE: usize = 8;
    
    /// Maximum payload size
    pub const MAX_PAYLOAD_SIZE: usize = 65535 - Self::HEADER_SIZE;
    
    /// Create a new packet with empty payload
    ///
    /// # Examples
    ///
    /// ```
    /// use zkrust_core::{Packet, Command};
    ///
    /// let packet = Packet::new(Command::Connect, 0, 0);
    /// assert_eq!(packet.payload.len(), 0);
    /// ```
    pub fn new(command: Command, session_id: u16, reply_id: u16) -> Self {
        Self {
            command,
            session_id,
            reply_id,
            payload: Bytes::new(),
        }
    }
    
    /// Create a packet with payload
    ///
    /// # Examples
    ///
    /// ```
    /// use zkrust_core::{Packet, Command};
    ///
    /// let payload = vec![1, 2, 3, 4];
    /// let packet = Packet::with_payload(Command::Auth, 1234, 65534, payload);
    /// assert_eq!(packet.payload.len(), 4);
    /// ```
    pub fn with_payload(
        command: Command,
        session_id: u16,
        reply_id: u16,
        payload: impl Into<Bytes>,
    ) -> Self {
        Self {
            command,
            session_id,
            reply_id,
            payload: payload.into(),
        }
    }
    
    /// Calculate checksum for this packet
    ///
    /// Uses the ZKTeco checksum algorithm (ones-complement sum).
    pub fn checksum(&self) -> u16 {
        checksum::calculate(
            self.command.into(),
            self.session_id,
            self.reply_id,
            &self.payload,
        )
    }
    
    /// Encode packet to bytes
    ///
    /// # Returns
    ///
    /// A `BytesMut` containing the complete encoded packet.
    ///
    /// # Examples
    ///
    /// ```
    /// use zkrust_core::{Packet, Command};
    ///
    /// let packet = Packet::new(Command::Connect, 0, 0);
    /// let bytes = packet.encode();
    /// assert_eq!(bytes.len(), 8); // Header only
    /// ```
    pub fn encode(&self) -> BytesMut {
        let total_size = Self::HEADER_SIZE + self.payload.len();
        let mut buf = BytesMut::with_capacity(total_size);
        
        // Encode header (little-endian)
        buf.put_u16_le(self.command.into());
        buf.put_u16_le(self.checksum());
        buf.put_u16_le(self.session_id);
        buf.put_u16_le(self.reply_id);
        
        // Append payload
        buf.put_slice(&self.payload);
        
        buf
    }
    
    /// Decode packet from bytes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Buffer is too short (< 8 bytes)
    /// - Checksum verification fails
    /// - Command code is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use zkrust_core::{Packet, Command};
    ///
    /// let original = Packet::new(Command::Connect, 0, 0);
    /// let encoded = original.encode();
    /// let decoded = Packet::decode(encoded).unwrap();
    ///
    /// assert_eq!(original.command, decoded.command);
    /// ```
    pub fn decode(mut buf: BytesMut) -> Result<Self> {
        // Check minimum size
        if buf.len() < Self::HEADER_SIZE {
            return Err(Error::PacketTooShort {
                expected: Self::HEADER_SIZE,
                actual: buf.len(),
            });
        }
        
        // Decode header
        let command_raw = buf.get_u16_le();
        let checksum_received = buf.get_u16_le();
        let session_id = buf.get_u16_le();
        let reply_id = buf.get_u16_le();
        
        // Parse command
        let command = Command::try_from(command_raw)?;
        
        // Remaining bytes are payload
        let payload = buf.freeze();
        
        // Construct packet
        let packet = Self {
            command,
            session_id,
            reply_id,
            payload,
        };
        
        // Verify checksum
        let checksum_calculated = packet.checksum();
        if checksum_calculated != checksum_received {
            return Err(Error::ChecksumMismatch {
                expected: checksum_calculated,
                received: checksum_received,
            });
        }
        
        Ok(packet)
    }
    
    /// Check if this is a response packet (ACK)
    pub fn is_response(&self) -> bool {
        matches!(
            self.command,
            Command::AckOk
                | Command::AckError
                | Command::AckData
                | Command::AckRetry
                | Command::AckRepeat
                | Command::AckUnauth
                | Command::AckUnknown
                | Command::AckErrorCmd
                | Command::AckErrorInit
                | Command::AckErrorData
        )
    }
    
    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        matches!(self.command, Command::AckOk | Command::AckData)
    }
    
    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(
            self.command,
            Command::AckError
                | Command::AckErrorCmd
                | Command::AckErrorInit
                | Command::AckErrorData
        )
    }
    
    /// Get total packet size
    pub fn size(&self) -> usize {
        Self::HEADER_SIZE + self.payload.len()
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("command", &self.command)
            .field("session_id", &format!("0x{:04X}", self.session_id))
            .field("reply_id", &format!("0x{:04X}", self.reply_id))
            .field("checksum", &format!("0x{:04X}", self.checksum()))
            .field("payload_len", &self.payload.len())
            .finish()
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Packet[{}](session={}, reply={}, len={})",
            self.command,
            self.session_id,
            self.reply_id,
            self.payload.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn test_packet_new() {
        let packet = Packet::new(Command::Connect, 0, 0);
        assert_eq!(packet.command, Command::Connect);
        assert_eq!(packet.session_id, 0);
        assert_eq!(packet.reply_id, 0);
        assert_eq!(packet.payload.len(), 0);
    }
    
    #[test]
    fn test_packet_with_payload() {
        let payload = vec![1, 2, 3, 4];
        let packet = Packet::with_payload(Command::Auth, 1234, 65534, payload.clone());
        
        assert_eq!(packet.command, Command::Auth);
        assert_eq!(packet.payload.as_ref(), payload.as_slice());
    }
    
    #[test]
    fn test_packet_encode_decode() {
        let original = Packet::with_payload(
            Command::Connect,
            0,
            0,
            vec![1, 2, 3, 4],
        );
        
        let encoded = original.encode();
        let decoded = Packet::decode(encoded).unwrap();
        
        assert_eq!(original.command, decoded.command);
        assert_eq!(original.session_id, decoded.session_id);
        assert_eq!(original.reply_id, decoded.reply_id);
        assert_eq!(original.payload, decoded.payload);
    }
    
    #[test]
    fn test_packet_checksum_verification() {
        let packet = Packet::new(Command::Connect, 0, 65534);
        let mut encoded = packet.encode();
        
        // Corrupt checksum (bytes 2-3)
        encoded[2] ^= 0xFF;
        encoded[3] ^= 0xFF;
        
        let result = Packet::decode(encoded);
        assert!(result.is_err());
        
        if let Err(Error::ChecksumMismatch { expected, received }) = result {
            assert_ne!(expected, received);
        } else {
            panic!("Expected ChecksumMismatch error");
        }
    }
    
    #[test]
    fn test_packet_too_short() {
        let buf = BytesMut::from(&[1, 2, 3][..]);
        let result = Packet::decode(buf);
        
        assert!(matches!(result, Err(Error::PacketTooShort { .. })));
    }
    
    #[test]
    fn test_packet_empty() {
        let packet = Packet::new(Command::Connect, 0, 0);
        let encoded = packet.encode();
        
        assert_eq!(encoded.len(), Packet::HEADER_SIZE);
        
        let decoded = Packet::decode(encoded).unwrap();
        assert_eq!(decoded.payload.len(), 0);
    }
    
    #[test]
    fn test_packet_large_payload() {
        let payload = vec![0xAB; 1000];
        let packet = Packet::with_payload(Command::Auth, 100, 200, payload.clone());
        
        let encoded = packet.encode();
        let decoded = Packet::decode(encoded).unwrap();
        
        assert_eq!(decoded.payload.as_ref(), payload.as_slice());
    }
    
    #[test]
    fn test_is_response() {
        assert!(Packet::new(Command::AckOk, 0, 0).is_response());
        assert!(Packet::new(Command::AckError, 0, 0).is_response());
        assert!(!Packet::new(Command::Connect, 0, 0).is_response());
    }
    
    #[test]
    fn test_is_success() {
        assert!(Packet::new(Command::AckOk, 0, 0).is_success());
        assert!(Packet::new(Command::AckData, 0, 0).is_success());
        assert!(!Packet::new(Command::AckError, 0, 0).is_success());
    }
}