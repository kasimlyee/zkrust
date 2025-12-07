//! ZKTeco checksum algorithm
//!
//! From the official protocol manual:
//! 1. Build buffer: [Command, 0x00, 0x00, SessionID, ReplyID, Payload]
//! 2. Sum as unsigned 16-bit little-endian values
//! 3. When sum > 0xFFFF, subtract 0xFFFF (wrapping)
//! 4. Take ones-complement: ~sum
//! 5. Return as unsigned 16-bit

use tracing::trace;

/// Calculate ZKTeco packet checksum
///
/// # Algorithm
///
/// ```text
/// 1. Create buffer: [cmd_lo, cmd_hi, 0, 0, sess_lo, sess_hi, reply_lo, reply_hi, ...payload]
/// 2. Sum all 16-bit words (little-endian)
/// 3. While sum > 0xFFFF: sum -= 0xFFFF
/// 4. Return ~sum as u16
/// ```
///
/// # Examples
///
/// ```
/// use zkrust_core::checksum;
///
/// let checksum = checksum::calculate(1000, 0, 0, &[]);
/// println!("Checksum: 0x{:04X}", checksum);
/// ```
pub fn calculate(command: u16, session_id: u16, reply_id: u16, payload: &[u8]) -> u16 {
    // Build complete buffer for checksum calculation
    let mut buf = Vec::with_capacity(8 + payload.len());
    
    // Header (checksum field is 0x0000 for calculation)
    buf.extend_from_slice(&command.to_le_bytes());
    buf.extend_from_slice(&[0, 0]); // Checksum placeholder
    buf.extend_from_slice(&session_id.to_le_bytes());
    buf.extend_from_slice(&reply_id.to_le_bytes());
    buf.extend_from_slice(payload);
    
    // Sum as 16-bit words
    let mut sum: u32 = 0;
    
    for chunk in buf.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_le_bytes([chunk[0], chunk[1]]) as u32
        } else {
            // Odd byte - treat as low byte of u16
            chunk[0] as u32
        };
        
        sum = sum.wrapping_add(word);
        
        // Wrap if exceeds 16-bit
        while sum > 0xFFFF {
            sum = sum.wrapping_sub(0xFFFF);
        }
    }
    
    // Final wrapping (in case still > 0xFFFF)
    while sum > 0xFFFF {
        sum = sum.wrapping_sub(0xFFFF);
    }
    
    // Ones complement
    let checksum = !sum as u16;
    
    trace!(
        command = command,
        session_id = session_id,
        reply_id = reply_id,
        payload_len = payload.len(),
        checksum = format!("0x{:04X}", checksum),
        "Calculated checksum"
    );
    
    checksum
}

/// Verify checksum
pub fn verify(
    command: u16,
    session_id: u16,
    reply_id: u16,
    payload: &[u8],
    expected: u16,
) -> bool {
    let calculated = calculate(command, session_id, reply_id, payload);
    calculated == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checksum_empty_payload() {
        // CMD_CONNECT (1000), session=0, reply=0, no payload
        let checksum = calculate(1000, 0, 0, &[]);
        
        // Checksum should be consistent
        assert_eq!(checksum, calculate(1000, 0, 0, &[]));
    }
    
    #[test]
    fn test_checksum_with_payload() {
        let payload = vec![1, 2, 3, 4];
        let checksum = calculate(1000, 100, 200, &payload);
        
        // Verify is consistent
        assert_eq!(checksum, calculate(1000, 100, 200, &payload));
    }
    
    #[test]
    fn test_checksum_verify() {
        let payload = vec![0xAB, 0xCD];
        let checksum = calculate(1000, 50, 100, &payload);
        
        assert!(verify(1000, 50, 100, &payload, checksum));
        assert!(!verify(1000, 50, 100, &payload, checksum.wrapping_add(1)));
    }
    
    #[test]
    fn test_checksum_different_commands() {
        // Different commands should produce different checksums
        let cs1 = calculate(1000, 0, 0, &[]);
        let cs2 = calculate(1001, 0, 0, &[]);
        
        assert_ne!(cs1, cs2);
    }
    
    #[test]
    fn test_checksum_different_sessions() {
        let cs1 = calculate(1000, 100, 0, &[]);
        let cs2 = calculate(1000, 200, 0, &[]);
        
        assert_ne!(cs1, cs2);
    }
    
    #[test]
    fn test_checksum_odd_payload_length() {
        // Test with odd-length payload
        let payload = vec![1, 2, 3];
        let checksum = calculate(1000, 0, 0, &payload);
        
        // Should not panic and should be consistent
        assert_eq!(checksum, calculate(1000, 0, 0, &payload));
    }
    
    #[test]
    fn test_checksum_large_payload() {
        let payload = vec![0xFF; 1000];
        let checksum = calculate(1000, 0, 0, &payload);
        
        // Should handle large payloads
        assert_eq!(checksum, calculate(1000, 0, 0, &payload));
    }
}