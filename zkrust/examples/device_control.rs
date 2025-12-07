//! Device control example

use std::time::Duration;
use tokio::time::sleep;
use zkrust::Device;

#[tokio::main]
async fn main() -> zkrust::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let ip = std::env::var("DEVICE_IP").unwrap_or_else(|_| "192.168.1.201".to_string());
    
    let mut device = Device::new(ip, 4370);
    device.connect().await?;
    
    println!("Device connected!");
    
    // Disable device (show "Working...")
    println!("Disabling device...");
    device.disable_device().await?;
    sleep(Duration::from_secs(3)).await;
    
    // Enable device (resume normal operation)
    println!("Enabling device...");
    device.enable_device().await?;
    
    println!("Done!");
    
    device.disconnect().await?;
    
    Ok(())
}