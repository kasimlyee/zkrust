//! Session management for ZKTeco protocol
//!
//! A session represents a connection to a device and tracks:
//! - Session ID (assigned by device)
//! - Reply counter (increments per command)
//! - Authentication state

use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::error::{Error, Result};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Not connected
    Disconnected,
    
    /// Connected but not authenticated
    Connected,
    
    /// Authenticated and ready for commands
    Authenticated,
}

/// Session manager
///
/// Manages session state and reply ID generation.
/// Thread-safe and can be cloned cheaply (Arc internally).
#[derive(Debug, Clone)]
pub struct Session {
    inner: Arc<SessionInner>,
}

#[derive(Debug)]
struct SessionInner {
    /// Session ID assigned by device (0 when not connected)
    session_id: AtomicU16,
    
    /// Reply counter (starts at USHRT_MAX - 1 = 65534)
    reply_counter: AtomicU16,
    
    /// Current session state
    state: parking_lot::RwLock<SessionState>,
}

impl Session {
    /// Initial reply ID (from protocol manual: USHRT_MAX - 1)
    pub const INITIAL_REPLY_ID: u16 = 65534;
    
    /// Create a new disconnected session
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SessionInner {
                session_id: AtomicU16::new(0),
                reply_counter: AtomicU16::new(Self::INITIAL_REPLY_ID),
                state: parking_lot::RwLock::new(SessionState::Disconnected),
            }),
        }
    }
    
    /// Get current session ID
    pub fn session_id(&self) -> u16 {
        self.inner.session_id.load(Ordering::Acquire)
    }
    
    /// Get current state
    pub fn state(&self) -> SessionState {
        *self.inner.state.read()
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        !matches!(self.state(), SessionState::Disconnected)
    }
    
    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state(), SessionState::Authenticated)
    }
    
    /// Initialize session with device-assigned session ID
    pub fn initialize(&self, session_id: u16) -> Result<()> {
        let mut state = self.inner.state.write();
        
        if *state != SessionState::Disconnected {
            return Err(Error::InvalidSessionState(
                format!("Cannot initialize from state: {:?}", *state)
            ));
        }
        
        self.inner.session_id.store(session_id, Ordering::Release);
        self.inner.reply_counter.store(Self::INITIAL_REPLY_ID, Ordering::Release);
        *state = SessionState::Connected;
        
        Ok(())
    }
    
    /// Mark session as authenticated
    pub fn authenticate(&self) -> Result<()> {
        let mut state = self.inner.state.write();
        
        if *state != SessionState::Connected {
            return Err(Error::InvalidSessionState(
                format!("Cannot authenticate from state: {:?}", *state)
            ));
        }
        
        *state = SessionState::Authenticated;
        Ok(())
    }
    
    /// Close session
    pub fn close(&self) {
        self.inner.session_id.store(0, Ordering::Release);
        self.inner.reply_counter.store(Self::INITIAL_REPLY_ID, Ordering::Release);
        *self.inner.state.write() = SessionState::Disconnected;
    }
    
    /// Get next reply ID
    ///
    /// Reply ID starts at 65534 and increments per command.
    /// Wraps around after reaching 65535.
    pub fn next_reply_id(&self) -> u16 {
        let current = self.inner.reply_counter.fetch_add(1, Ordering::AcqRel);
        
        // Wrap around if we hit max
        if current >= 65535 {
            self.inner.reply_counter.store(0, Ordering::Release);
        }
        
        current
    }
    
    /// Reset reply counter (used in testing)
    #[cfg(test)]
    pub fn reset_reply_counter(&self) {
        self.inner.reply_counter.store(Self::INITIAL_REPLY_ID, Ordering::Release);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_new() {
        let session = Session::new();
        assert_eq!(session.session_id(), 0);
        assert_eq!(session.state(), SessionState::Disconnected);
        assert!(!session.is_connected());
        assert!(!session.is_authenticated());
    }
    
    #[test]
    fn test_session_initialize() {
        let session = Session::new();
        session.initialize(1234).unwrap();
        
        assert_eq!(session.session_id(), 1234);
        assert_eq!(session.state(), SessionState::Connected);
        assert!(session.is_connected());
        assert!(!session.is_authenticated());
    }
    
    #[test]
    fn test_session_authenticate() {
        let session = Session::new();
        session.initialize(1234).unwrap();
        session.authenticate().unwrap();
        
        assert_eq!(session.state(), SessionState::Authenticated);
        assert!(session.is_authenticated());
    }
    
    #[test]
    fn test_session_close() {
        let session = Session::new();
        session.initialize(1234).unwrap();
        session.authenticate().unwrap();
        
        session.close();
        
        assert_eq!(session.session_id(), 0);
        assert_eq!(session.state(), SessionState::Disconnected);
    }
    
    #[test]
    fn test_reply_id_generation() {
        let session = Session::new();
        session.initialize(100).unwrap();
        
        let id1 = session.next_reply_id();
        let id2 = session.next_reply_id();
        let id3 = session.next_reply_id();
        
        assert_eq!(id1, 65534);
        assert_eq!(id2, 65535);
        assert_eq!(id3, 0); // Wrapped
    }
    
    #[test]
    fn test_reply_id_wrap() {
        let session = Session::new();
        session.initialize(100).unwrap();
        
        // Generate many IDs to test wrapping
        for _ in 0..70000 {
            session.next_reply_id();
        }
        
        // Should have wrapped multiple times
        let id = session.next_reply_id();
        assert!(id < 10000); // Wrapped back to low values
    }
    
    #[test]
    fn test_invalid_state_transitions() {
        let session = Session::new();
        
        // Cannot authenticate without connecting
        assert!(session.authenticate().is_err());
        
        // Cannot initialize twice
        session.initialize(100).unwrap();
        assert!(session.initialize(200).is_err());
    }
    
    #[test]
    fn test_session_clone() {
        let session1 = Session::new();
        session1.initialize(1234).unwrap();
        
        let session2 = session1.clone();
        
        // Both share same state
        assert_eq!(session2.session_id(), 1234);
        
        session1.authenticate().unwrap();
        assert!(session2.is_authenticated());
    }
}