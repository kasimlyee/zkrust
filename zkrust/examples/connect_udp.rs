//! UDP connection example (recommended for most devices)

use tracing_subscriber;
use zkrust::Device;

#[tokio::main]
async fn main() -> zkrust::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Change to your device IP
    let ip = std::env::var("DEVICE_IP").unwrap_or_else(|_| "192.168.1.201".to_string());

    println!("Connecting to {} via UDP...", ip);

    // Use UDP transport (recommended for most ZKTeco devices)
    let mut device = Device::new_udp(ip, 4370);

    // Connect
    device.connect().await?;
    println!("✓ Connected!");

    // Get device info
    let info = device.get_device_info().await?;
    println!("✓ Device: {}", info);

    // Disconnect
    device.disconnect().await?;
    println!("✓ Disconnected");

    Ok(())
}
