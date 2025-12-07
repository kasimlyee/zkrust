//! Device information structures

use std::fmt;

/// Device information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Device serial number
    pub serial_number: String,
    
    /// Firmware version
    pub firmware_version: String,
    
    /// Device model
    pub model: Option<String>,
    
    /// Platform name
    pub platform: Option<String>,
    
    /// Device name (user-assigned)
    pub device_name: Option<String>,
    
    /// MAC address
    pub mac_address: Option<String>,
}

impl DeviceInfo {
    pub fn new(serial_number: String, firmware_version: String) -> Self {
        Self {
            serial_number,
            firmware_version,
            model: None,
            platform: None,
            device_name: None,
            mac_address: None,
        }
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device[SN: {}, FW: {}]",
            self.serial_number, self.firmware_version
        )
    }
}