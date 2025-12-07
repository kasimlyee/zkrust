//! Protocol constants

/// TCP magic header (for TCP-wrapped packets in some devices)
pub const TCP_MAGIC_1: u16 = 0x5050;
pub const TCP_MAGIC_2: u16 = 0x8272;

/// Default connection timeout (seconds)
pub const DEFAULT_TIMEOUT: u64 = 5;

/// Default read timeout (seconds)
pub const DEFAULT_READ_TIMEOUT: u64 = 5;

/// Maximum retries for network operations
pub const MAX_RETRIES: usize = 3;

/// Real-time event flags
pub mod events {
    /// Attendance log event
    pub const EF_ATTLOG: u32 = 1;
    
    /// Fingerprint pressed
    pub const EF_FINGER: u32 = 1 << 1;
    
    /// User enrolled
    pub const EF_ENROLLUSER: u32 = 1 << 2;
    
    /// Fingerprint enrolled
    pub const EF_ENROLLFINGER: u32 = 1 << 3;
    
    /// Button pressed
    pub const EF_BUTTON: u32 = 1 << 4;
    
    /// Door unlocked
    pub const EF_UNLOCK: u32 = 1 << 5;
    
    /// Verification event
    pub const EF_VERIFY: u32 = 1 << 7;
    
    /// Fingerprint minutiae captured
    pub const EF_FPFTR: u32 = 1 << 8;
    
    /// Alarm signal
    pub const EF_ALARM: u32 = 1 << 9;
}

/// Data type flags (for CMD_DB_RRQ, etc.)
pub mod data_types {
    /// Attendance log
    pub const FCT_ATTLOG: u8 = 1;
    
    /// Fingerprint template
    pub const FCT_FINGERTMP: u8 = 2;
    
    /// Operation log
    pub const FCT_OPLOG: u8 = 4;
    
    /// User record
    pub const FCT_USER: u8 = 5;
    
    /// SMS
    pub const FCT_SMS: u8 = 6;
    
    /// User data
    pub const FCT_UDATA: u8 = 7;
    
    /// Work code
    pub const FCT_WORKCODE: u8 = 8;
}

/// Verification modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VerifyMode {
    Password = 0,
    Fingerprint = 1,
    Card = 3,
    Face = 15,
}

/// Punch types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PunchType {
    CheckIn = 0,
    CheckOut = 1,
    OvertimeIn = 2,
    OvertimeOut = 3,
}