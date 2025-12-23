use local_ip_address::list_afinet_netifas;
use tokio::net::UdpSocket;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Show all local IP addresses for user visibility
        if let Ok(netifas) = list_afinet_netifas() {
            println!("Available local IP addresses:");
            for (_ifname, ip) in netifas {
                println!("  {}", ip);
            }
        }
    // 1. Identify your 'Parallel Lab' neighborhood
    let router_ip = &std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set"); // from .env

    // 2. Open 3 lanes for your 3 bands (2.4GHz, 5GHz-1, 5GHz-2)
    let sock_24 = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let sock_5g1 = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let sock_5g2 = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

    println!("🚀 Turbonet Initialized. Multi-band lanes ready.");

    // 3. Prepare the file to send
    let file_path = "test_payload.bin"; // Replace with your file path
    let file_bytes = std::fs::read(file_path)?;
    let file_size = file_bytes.len() as u64;

    // 4. Send handshake header (file size) on port 8001
    let header = file_size.to_be_bytes();
    sock_24.send_to(&header, format!("{}:8001", router_ip)).await?;
    println!("🔔 Handshake header sent: {} bytes", file_size);

    // 5. Send the actual data (simulate with a single UDP packet for now)
    sock_24.send_to(&file_bytes, format!("{}:8001", router_ip)).await?;
    println!("📡 Broadcasting file to ASUS Laboratory...");
    Ok(())
}