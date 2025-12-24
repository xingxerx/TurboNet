use std::time::{Instant, Duration};
use tokio::net::UdpSocket;

use std::process::Command;

pub struct LaneTelemetry {
    pub rtt: Duration,
    pub packet_loss: f32,
}

pub async fn probe_lane(socket: &UdpSocket, target: &str) -> LaneTelemetry {
    let start = Instant::now();
    let payload = b"PROBE";
    let _ = socket.send_to(payload, target).await;
    // Implement a 100ms timeout for RTT calculation
    // If timeout, packet_loss = 1.0
    LaneTelemetry {
        rtt: start.elapsed(),
        packet_loss: 0.0, // Logic for sequence tracking goes here
    }
}
// Suggested update for Broadcaster.rs logic
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

fn _get_wifi_ip() -> Option<String> {
    let interfaces = NetworkInterface::show().ok()?;
    // Look for the interface we just prioritized in PowerShell
    let wifi = interfaces.iter().find(|iface| iface.name == "Wi-Fi")?;
    
    wifi.addr.iter().find_map(|addr| {
        if let std::net::IpAddr::V4(ipv4) = addr.ip() {
            Some(ipv4.to_string())
        } else {
            None
        }
    })
}

// Design Pattern: Explicit Hardware Binding
// Chosen to bypass OS routing logic entirely and ensure 
// fragments physically traverse the prioritized Wi-Fi radio.

/// Heartbeat check for Ethernet lane (ASUS router gateway)
/// Returns true if the gateway responds to a ping within 100ms
pub fn check_ethernet_lane_health() -> bool {
    // Ping the ASUS router directly (192.168.50.1)
    // -n 1: only one packet
    // -w 100: 100ms timeout for high-speed gaming router response
    let status = Command::new("ping")
        .args(["-n", "1", "-w", "100", "192.168.50.1"])
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}