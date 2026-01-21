use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::net::UdpSocket;

fn main() {
    println!("🔍 DEEP SCANNING ALL HARDWARE LANES...");

    let interfaces = NetworkInterface::show().unwrap();

    for interface in interfaces {
        let ip = std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set");
        match UdpSocket::bind(format!("{}:0", ip)) {
            Ok(_) => println!("✅ ASUS LANE: FOUND ({}) - Ready!", ip),
            Err(_) => println!("❌ ASUS LANE: STILL NOT FOUND"),
        }

        // Logic to identify our ASUS Laboratory
        if interface.name.contains("Ethernet") || interface.name.contains("eth") {
            println!("   🚀 POTENTIAL TURBONET LANE DETECTED!");
        }
    }
}
