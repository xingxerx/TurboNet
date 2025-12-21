use std::net::UdpSocket;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

fn main() {
    println!("🔍 DEEP SCANNING ALL HARDWARE LANES...");
    
    let interfaces = NetworkInterface::show().unwrap();

    for interface in interfaces {
match UdpSocket::bind("192.168.50.97:0") {
    Ok(_) => println!("✅ ASUS LANE: FOUND (192.168.50.97) - Ready!"),
    Err(_) => println!("❌ ASUS LANE: STILL NOT FOUND"),
}
        
        // Logic to identify our ASUS Laboratory
        if interface.name.contains("Ethernet") || interface.name.contains("eth") {
            println!("   🚀 POTENTIAL TURBONET LANE DETECTED!");
        }
    }
}