//! # zkrust
//!
//! Rust implementation of the ZKTeco Attendance device communication protocol.
//!
//! ## Features
//!
//! - Type-safe protocol implementation
//! - Async/await API using Tokio
//! - Comprehensive error handling
//! - Full protocol support (50+ commands)
//!
//! ## Quick Start
//!
//! ```no_run
//! use zkrust::Device;
//!
//! #[tokio::main]
//! async fn main() -> zkrust::Result<()> {
//!     // Connect to device
//!     let mut device = Device::new("192.168.1.201", 4370);
//!     device.connect().await?;
//!     
//!     // Get device info
//!     let info = device.get_device_info().await?;
//!     println!("{}", info);
//!     
//!     // Disconnect
//!     device.disconnect().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod device;
pub mod error;

// Re-exports
pub use device::Device;
pub use error::{Error, Result};

// Re-export types
pub use zkrust_core::{Command, Packet, Session};
pub use zkrust_types::DeviceInfo;