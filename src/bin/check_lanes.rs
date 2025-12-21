use std::net::UdpSocket;

fn main() {
    // We try to bind to our specific 'Turbonet' neighborhood IP
    // Replace with the IP you set (e.g., 192.168.50.55)
    match UdpSocket::bind("192.168.50.55:0") {
        Ok(_) => println!("✅ SUCCESS: Rust found the ASUS Ethernet lane!"),
        Err(e) => println!("❌ FAILED: Rust can't find the Ethernet lane. Error: {}", e),
    }

    // Now check for the Starlink/General internet lane
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(_) => println!("✅ SUCCESS: Rust found the Starlink/General lane!"),
        Err(e) => println!("❌ FAILED: Everything is dark. Error: {}", e),
    }
}