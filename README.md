# zkrust

Rust Implementation of the ZKTeco Attendance device communication protocol.

## Features

- Type-safe protocol implementation
- Async/await API using Tokio
- Comprehensive error handling
- TCP transport 
- Full protocol support (50+ commands)
- Zero unsafe code


## Installation
```toml
[dependencies]
zkrust = "0.1"
```

## Quick Start
```rust
use zkrust::Device;

#[tokio::main]
async fn main() -> zkrust::Result<()> {
    // Connect to device
    let mut device = Device::new("192.168.1.201", 4370);
    device.connect().await?;
    
    // Get device info
    let info = device.get_device_info().await?;
    println!("Device: {}", info);
    
    // Disconnect
    device.disconnect().await?;
    
    Ok(())
}
```


## Testing with Real Device
```bash
# Set your device IP
export DEVICE_IP="192.168.1.201"

# Run tests
./test.sh
```

## License

MIT 