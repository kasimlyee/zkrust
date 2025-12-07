//! ZKTeco authentication algorithm
//!
//! The CommKey authentication algorithm scrambles the password with the session_id
//! to create an authentication key. This algorithm was reverse-engineered from
//! ZKTeco's commpro.c - MakeKey function.

use bytes::Bytes;

/// Create authentication key from password and session_id
///
/// This function takes a password and session_id and scrambles them
/// using ZKTeco's proprietary algorithm.
///
/// # Algorithm
///
/// 1. Reverse bits of the password
/// 2. Add session_id to reversed password
/// 3. XOR with 'Z', 'K', 'S', 'O' bytes
/// 4. Swap the two 16-bit halves
/// 5. XOR with ticks value
///
/// # Arguments
///
/// * `password` - The CommKey password (usually 0 for default)
/// * `session_id` - The session ID from CMD_ACK_UNAUTH response
/// * `ticks` - Ticks value (default: 50)
///
/// # Returns
///
/// 4-byte authentication key to send in CMD_AUTH payload
///
/// # Examples
///
/// ```
/// use zkrust_core::auth;
///
/// let auth_key = auth::make_commkey(0, 32031, 50);
/// assert_eq!(auth_key.len(), 4);
/// ```
pub fn make_commkey(password: u32, session_id: u16, ticks: u8) -> Bytes {
    // Reverse bits of password
    let mut k: u32 = 0;
    for i in 0..32 {
        if (password & (1 << i)) != 0 {
            k = (k << 1) | 1;
        } else {
            k = k << 1;
        }
    }

    // Add session_id
    k = k.wrapping_add(session_id as u32);

    // Convert to bytes and XOR with 'Z', 'K', 'S', 'O'
    let bytes = k.to_le_bytes();
    let xored = [
        bytes[0] ^ b'Z',
        bytes[1] ^ b'K',
        bytes[2] ^ b'S',
        bytes[3] ^ b'O',
    ];

    // Swap the two 16-bit halves
    // Convert to two u16 values
    let low = u16::from_le_bytes([xored[0], xored[1]]);
    let high = u16::from_le_bytes([xored[2], xored[3]]);

    // Swap them
    let swapped = [high, low];

    // Convert back to bytes
    let mut result = [0u8; 4];
    result[0..2].copy_from_slice(&swapped[0].to_le_bytes());
    result[2..4].copy_from_slice(&swapped[1].to_le_bytes());

    // XOR with ticks
    let b = ticks;
    result[0] ^= b;
    result[1] ^= b;
    result[2] = b;
    result[3] ^= b;

    Bytes::copy_from_slice(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_commkey_basic() {
        // Test with password=0, session_id=0, ticks=50
        let key = make_commkey(0, 0, 50);
        assert_eq!(key.len(), 4);
    }

    #[test]
    fn test_make_commkey_with_session() {
        // Test with password=0, session_id=32031, ticks=50
        let key = make_commkey(0, 32031, 50);
        assert_eq!(key.len(), 4);

        // The result should be deterministic
        let key2 = make_commkey(0, 32031, 50);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_make_commkey_different_passwords() {
        let key1 = make_commkey(0, 100, 50);
        let key2 = make_commkey(12345, 100, 50);

        // Different passwords should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_make_commkey_different_sessions() {
        let key1 = make_commkey(0, 100, 50);
        let key2 = make_commkey(0, 200, 50);

        // Different session IDs should produce different keys
        assert_ne!(key1, key2);
    }
}
