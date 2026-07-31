//! Traffic Monitor - Passive AI-Powered Detection
//!
//! Captures traffic, analyzes it with an LLM, and logs suspicious/malicious
//! source IPs for review. This monitor is strictly passive: it never drops,
//! blocks, filters, or otherwise alters traffic. Detection and logging only.

use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::interval;
use turbonet_core::ai_defense::{
    parse_model_spec, DefenseAdvisor, ThreatAssessment, TrafficPacket,
};
use turbonet_core::neural_link::NeuralBus;

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
            let model = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| "ollama:gpt-oss".to_string());
            run_monitor(port, &model).await?;
        }
        _ => print_usage(&args[0]),
    }

    Ok(())
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!(
        "  {} --run [PORT] [MODEL]   Start passive traffic monitor (detect + log only)",
        prog
    );
}

async fn run_monitor(port: u16, model_spec: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Starting passive Traffic Monitor on port {}...", port);
    println!("🤖 AI Analyst: {}", model_spec);
    println!("ℹ️  Detection + logging only — no traffic is blocked or dropped.");

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

    let packet_buffer: Arc<Mutex<Vec<TrafficPacket>>> = Arc::new(Mutex::new(Vec::new()));

    // Use Tokio's UdpSocket for async operations
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    let buffer_clone = packet_buffer.clone();

    let advisor_arc = Arc::new(advisor);
    let mut ticker = interval(Duration::from_secs(5));
    let mut buf = [0u8; 65535];

    println!("🟢 Monitor Active. Observing traffic... (Press Ctrl+C to stop)");

    loop {
        tokio::select! {
             // 1. Handle Shutdown
            _ = tokio::signal::ctrl_c() => {
                println!("\n🛑 Received shutdown signal. Stopping Traffic Monitor...");
                break;
            }
             // 2. Handle Packet Capture (observe only)
            res = socket.recv_from(&mut buf) => {
                match res {
                    Ok((amt, src)) => {
                        let src_ip = src.ip().to_string();
                        println!("Packet from {}", src);
                        let payload = &buf[..amt];
                        let payload_preview = String::from_utf8_lossy(&payload.iter().take(100).cloned().collect::<Vec<u8>>()).to_string();

                        let packet = TrafficPacket {
                             timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                             src_ip,
                             dst_port: port,
                             protocol: "UDP".to_string(),
                             payload_size: amt,
                             payload_preview,
                        };
                        buffer_clone.lock().unwrap().push(packet);
                    }
                    Err(e) => eprintln!("Error receiving packet: {}", e),
                }
            }
             // 3. Handle Analysis Tick
            _ = ticker.tick() => {
                let batch: Vec<TrafficPacket> = {
                    let mut b = buffer_clone.lock().unwrap();
                    b.drain(..).collect()
                };

                if !batch.is_empty() {
                    println!("🔍 Analyzing batch of {} packets...", batch.len());
                    let advisor = advisor_arc.clone();

                    tokio::spawn(async move {
                         // Calls the AI for passive classification
                         let result = advisor.analyze_traffic_batch(&batch).await;

                         match result {
                             Ok(assessments) => {
                                 let mut active_threats = 0;
                                 let mut impacted_lanes = Vec::new();

                                 for a in assessments {
                                     match a.assessment {
                                         ThreatAssessment::Malicious => {
                                             println!("🚩 THREAT DETECTED (logged, no action taken) {} (Confidence: {}%): {}", a.ip, a.confidence, a.reason);
                                             active_threats += 1;
                                             impacted_lanes.push("UDP".to_string());
                                         }
                                         ThreatAssessment::Suspicious => {
                                             println!("⚠️  SUSPICIOUS {}: {}", a.ip, a.reason);
                                         }
                                         ThreatAssessment::Benign => {}
                                     }
                                 }

                                 // Update Neural Bus (telemetry only)
                                 if active_threats > 0 {
                                     println!("📡 Updating Neural Bus with {} detected threats...", active_threats);
                                     NeuralBus::update(active_threats, impacted_lanes, Some("Suspicious activity detected (logged)".to_string()));
                                 }
                             }
                             Err(e) => eprintln!("❌ Analyst Error: {}", e),
                         }
                    });
                }
            }
        }
    }

    Ok(())
}
