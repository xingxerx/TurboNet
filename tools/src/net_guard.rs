//! Traffic Guard - AI-Powered Active Defense
//!
//! Captures traffic, analyzes it with GPT-OSS, and blocks malicious IPs.

use std::collections::HashSet;

use std::env;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use turbonet_core::ai_defense::{DefenseAdvisor, TrafficPacket, DecisionType, parse_model_spec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    let mode = args[1].as_str();
    match mode {
        "--run" => {
            let port = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8888);
            let model = args.get(3).cloned().unwrap_or_else(|| "ollama:gpt-oss".to_string());
            // Check for --mock flag in remaining args
            let mock = args.iter().any(|a| a == "--mock");
            run_guard(port, &model, mock).await?;
        }
        _ => print_usage(&args[0]),
    }

    Ok(())
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --run [PORT] [MODEL]   Start active traffic guard", prog);
}

async fn run_guard(port: u16, model_spec: &str, mock: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Starting Traffic Guard on port {}...", port);
    println!("🤖 AI Analyst: {}{}", model_spec, if mock { " (MOCK MODE)" } else { "" });

    let (provider, model_name) = parse_model_spec(model_spec);
    let advisor = match provider.as_str() {
        "ollama" => DefenseAdvisor::ollama(&model_name),
        "openai" => DefenseAdvisor::openai_compatible(
             "https://api.openai.com/v1/chat/completions",
             &model_name,
             std::env::var("OPENAI_API_KEY").ok().as_deref(),
        ),
        _ => DefenseAdvisor::ollama(&model_name),
    };

    let blocked_ips: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let packet_buffer: Arc<Mutex<Vec<TrafficPacket>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Packet capture thread
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    
    let blocked_clone = blocked_ips.clone();
    let buffer_clone = packet_buffer.clone();
    let socket_clone = socket.try_clone()?;

    // Analysis loop
    let advisor_arc = Arc::new(advisor);
    let blocked_analysis = blocked_ips.clone();
    let buffer_analysis = packet_buffer.clone();

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            
            let batch: Vec<TrafficPacket> = {
                let mut buf = buffer_analysis.lock().unwrap();
                let batch = buf.drain(..).collect();
                batch
            };

            if batch.is_empty() {
                continue;
            }

            println!("🔍 Analyzing batch of {} packets...", batch.len());
            
            let result = if mock {
                // Mock Analysis Logic
                Ok(batch.iter().map(|p| {
                     let decision = if p.payload_preview.to_lowercase().contains("malicious") || p.payload_preview.contains("DROP TABLE") {
                         turbonet_core::ai_defense::TrafficDecision {
                             ip: p.src_ip.clone(),
                             decision: DecisionType::Block,
                             confidence: 99,
                             reason: "Mock Mode: Detected malicious keyword".to_string()
                         }
                     } else {
                         turbonet_core::ai_defense::TrafficDecision {
                             ip: p.src_ip.clone(),
                             decision: DecisionType::Allow,
                             confidence: 90,
                             reason: "Mock Mode: Traffic seems compliant".to_string()
                         }
                     };
                     decision
                }).collect())
            } else {
                advisor_arc.analyze_traffic_batch(&batch).await
            };

            match result {
                Ok(decisions) => {
                    let mut blocked = blocked_analysis.lock().unwrap();
                    for d in decisions {
                        if d.decision == DecisionType::Block {
                            println!("🚫 BLOCKING {} (Confidence: {}%): {}", d.ip, d.confidence, d.reason);
                            blocked.insert(d.ip);
                        } else if d.decision == DecisionType::Monitor {
                             println!("⚠️  MONITORING {}: {}", d.ip, d.reason);
                        }
                    }
                }
                Err(e) => eprintln!("❌ Analyst Error: {}", e),
            }
        }
    });

    // Capture loop (Main thread)
    let mut buf = [0u8; 65535];
    println!("🟢 Guard Active. Waiting for traffic...");
    
    loop {
        match socket_clone.recv_from(&mut buf) {
            Ok((size, src)) => {
                let src_ip = src.ip().to_string();
                
                // Check blocklist
                {
                    let blocked = blocked_clone.lock().unwrap();
                    if blocked.contains(&src_ip) {
                        // Silent drop
                        continue;
                    }
                }

                println!("Packet from {}", src);

                // Add to buffer
                let packet = TrafficPacket {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    src_ip,
                    dst_port: port,
                    protocol: "UDP".to_string(),
                    payload_size: size,
                    payload_preview: String::from_utf8_lossy(&buf[..std::cmp::min(size, 32)]).to_string(),
                };

                let mut buffer = buffer_clone.lock().unwrap();
                if buffer.len() < 1000 { // Cap buffer
                    buffer.push(packet);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(e) => eprintln!("Socket error: {}", e),
        }
    }
}
