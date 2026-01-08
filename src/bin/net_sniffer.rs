//! Network Sniffer - Raw socket packet capture
//!
//! Part of TurboNet Security Toolkit

use std::env;
use std::net::UdpSocket;
use std::time::Duration;

fn main() {
    print_banner();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    match args[1].as_str() {
        "--listen" | "-l" => {
            let port = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8888);
            listen_udp(port);
        }
        "--scan" | "-s" => {
            let target = args.get(2).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            port_scan(&target);
        }
        "--help" | "-h" => print_usage(&args[0]),
        _ => print_usage(&args[0]),
    }
}

fn print_banner() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║   ███████╗███╗   ██╗██╗███████╗███████╗███████╗██████╗        ║
║   ██╔════╝████╗  ██║██║██╔════╝██╔════╝██╔════╝██╔══██╗       ║
║   ███████╗██╔██╗ ██║██║█████╗  █████╗  █████╗  ██████╔╝       ║
║   ╚════██║██║╚██╗██║██║██╔══╝  ██╔══╝  ██╔══╝  ██╔══██╗       ║
║   ███████║██║ ╚████║██║██║     ██║     ███████╗██║  ██║       ║
║   ╚══════╝╚═╝  ╚═══╝╚═╝╚═╝     ╚═╝     ╚══════╝╚═╝  ╚═╝       ║
║              Network Tool v0.1                                 ║
╚═══════════════════════════════════════════════════════════════╝
"#);
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --listen [PORT]     Listen for UDP packets (default: 8888)", prog);
    println!("  {} --scan <HOST>       Quick TCP port scan", prog);
    println!();
    println!("Examples:");
    println!("  {} --listen 9999", prog);
    println!("  {} --scan 192.168.1.1", prog);
}

fn listen_udp(port: u16) {
    println!("[*] Starting UDP listener on port {}...\n", port);
    
    let socket = match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[!] Failed to bind: {}", e);
            return;
        }
    };

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Listening for UDP packets... (Ctrl+C to stop)                ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Source             | Size    | Preview                       ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");

    let mut buf = [0u8; 65535];
    let mut packet_count = 0u64;

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                packet_count += 1;
                
                // Create preview (first 20 bytes as hex or ASCII)
                let preview: String = buf[..std::cmp::min(size, 16)]
                    .iter()
                    .map(|&b| {
                        if b >= 0x20 && b < 0x7F {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();

                println!("║  {:18} | {:>7} | {:30} ║", 
                         src, 
                         format!("{} B", size),
                         preview);

                // Show hex dump for small packets
                if size <= 64 {
                    let hex: String = buf[..size]
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("║    Hex: {:58} ║", 
                             if hex.len() > 58 { &hex[..55] } else { &hex });
                }
            }
            Err(e) => {
                eprintln!("[!] Error: {}", e);
                break;
            }
        }
    }

    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("\n[*] Total packets received: {}", packet_count);
}

fn port_scan(target: &str) {
    use std::net::TcpStream;
    
    println!("[*] Scanning {}...\n", target);
    
    let common_ports = [
        21, 22, 23, 25, 53, 80, 110, 135, 139, 143, 443, 445, 993, 995,
        1433, 1521, 3306, 3389, 5432, 5900, 6379, 8080, 8443, 27017
    ];

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  PORT      STATE      SERVICE                                 ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");

    let mut open_count = 0;

    for &port in &common_ports {
        let addr = format!("{}:{}", target, port);
        
        let result = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(200)
        );

        if result.is_ok() {
            open_count += 1;
            let service = get_service_name(port);
            println!("║  {:>5}     OPEN       {:42} ║", port, service);
        }
    }

    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("\n[*] Scan complete: {} open ports found", open_count);
}

fn get_service_name(port: u16) -> &'static str {
    match port {
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        53 => "DNS",
        80 => "HTTP",
        110 => "POP3",
        135 => "MS-RPC",
        139 => "NetBIOS",
        143 => "IMAP",
        443 => "HTTPS",
        445 => "SMB",
        993 => "IMAPS",
        995 => "POP3S",
        1433 => "MSSQL",
        1521 => "Oracle",
        3306 => "MySQL",
        3389 => "RDP",
        5432 => "PostgreSQL",
        5900 => "VNC",
        6379 => "Redis",
        8080 => "HTTP-Alt",
        8443 => "HTTPS-Alt",
        27017 => "MongoDB",
        _ => "Unknown",
    }
}
