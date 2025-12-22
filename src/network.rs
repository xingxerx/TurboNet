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