use tokio::net::UdpSocket;
use std::time::{Instant, Duration};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let router_ip = &std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set"); // from .env
    let target = format!("{}:8001", router_ip);
    
    // Create a 64KB chunk of "junk" data to flood the pipe
    let junk_data = vec![0u8; 65507]; // Max UDP packet size
    let data = Arc::new(junk_data);
    
    // Open the socket
    let ip = std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set");
    let socket = Arc::new(UdpSocket::bind(format!("{}:0", ip)).await?);
    
    println!("��� TURBONET SPEED TEST STARTING...");
    println!("Targeting ASUS Lab at {}", target);

    let start = Instant::now();
    let mut bytes_sent = 0u128;

    // Run the flood for 10 seconds
    while start.elapsed() < Duration::from_secs(10) {
        // Synchronous send: wait for OS confirmation
        socket.send_to(&data, &target).await?; 
        bytes_sent += data.len() as u128;
        
        // Print progress every second
        if bytes_sent % (data.len() as u128 * 100) == 0 {
            let gbps = (bytes_sent as f64 * 8.0) / (start.elapsed().as_secs_f64() * 1_000_000_000.0);
            print!("\rCurrent Speed: {:.2} Gbps", gbps);
        }
    }

    Ok(())
}
