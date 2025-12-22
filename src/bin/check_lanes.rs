use std::net::UdpSocket;

fn main() {
    // We try to bind to our specific 'Turbonet' neighborhood IP
    // Use IP from .env
    let ip = std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set");
    match UdpSocket::bind(format!("{}:0", ip)) {
        Ok(_) => println!("✅ SUCCESS: Rust found the ASUS Ethernet lane!"),
        Err(e) => println!("❌ FAILED: Rust can't find the Ethernet lane. Error: {}", e),
    }

    // Now check for the Starlink/General internet lane
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(_) => println!("✅ SUCCESS: Rust found the Starlink/General lane!"),
        Err(e) => println!("❌ FAILED: Everything is dark. Error: {}", e),
    }
}