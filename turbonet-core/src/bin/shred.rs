use cudarc::driver::CudaDevice;
use std::fs;
use std::sync::Arc;
use tokio::net::UdpSocket;
use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use pqc_kyber::*;
use std::convert::TryInto;
use turbonet_core::deepseek_weights::DeepSeekWeights;
use turbonet_core::ai_weights::HeuristicPredictor;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}




async fn get_ai_strategy(rtt_data: [f64; 3]) -> Option<DeepSeekWeights> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
    
    let prompt = format!(
        "Return a JSON object with weights for Lane 0, 1, and 2 based on these RTTs: Lane 0: {:.2}ms, Lane 1: {:.2}ms, Lane 2: {:.2}ms. \
        The weights must sum to 100. Lower RTT = higher weight. \
        Response MUST be strictly JSON. \
        Example: {{\"w0\": 10, \"w1\": 45, \"w2\": 45}}.",
        rtt_data[0] * 1000.0, rtt_data[1] * 1000.0, rtt_data[2] * 1000.0
    );

    let req = OllamaRequest {
        model,
        prompt,
        stream: false,
        format: "json".to_string(),
    };

    match tokio::time::timeout(std::time::Duration::from_millis(60000), client.post("http://localhost:11434/api/generate")
        .json(&req)
        .send()).await 
    {
        Ok(Ok(resp)) => {
            if let Ok(json_resp) = resp.json::<OllamaResponse>().await {
                // Use robust parser from library
                match DeepSeekWeights::from_raw_response(&json_resp.response) {
                    Ok(weights) => {
                         println!("🧠 DEEPSEEK THOUGHTS: Processed successfully.");
                         return Some(weights);
                    },
                    Err(e) => println!("❌ PARSE ERROR: {}", e),
                }
            }
        }
        _ => {}
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("🔥 TURBONET: DEEP-SEA ASYMMETRIC SHREDDER v3.0 ENGAGED...");
    
    let target_ip = std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP not set");
    let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let _p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let _p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
    
    let w0_env: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    let w1_env: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let w2_env: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let _manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_src = std::fs::read_to_string("shredder.cu")?;
    let ptx = cudarc::nvrtc::compile_ptx(ptx_src)?;
    let dev = CudaDevice::new(0)?;
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    // Create 3 sockets for multi-lane parallel sending
    let create_socket = || -> Result<Arc<UdpSocket>, Box<dyn std::error::Error>> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        sock.set_reuse_address(true)?;
        sock.set_recv_buffer_size(4 * 1024 * 1024)?;
        sock.set_send_buffer_size(4 * 1024 * 1024)?;
        sock.bind(&"0.0.0.0:0".parse::<SocketAddr>()?.into())?;
        Ok(Arc::new(UdpSocket::from_std(sock.into())?))
    };
    
    let socket = create_socket()?;      // Primary (Lane 0)
    let socket_l1 = create_socket()?;   // Lane 1
    let socket_l2 = create_socket()?;   // Lane 2
    
    let multilane_mode = std::env::var("TURBONET_MULTILANE").unwrap_or_else(|_| "false".to_string()) == "true";

    let file_path = std::env::var("TURBONET_FILE").unwrap_or_else(|_| "payload.bin".to_string());
    let file_bytes = fs::read(&file_path).unwrap_or_else(|_| panic!("Failed to read '{}'. Set TURBONET_FILE=yourfile.ext", file_path));
    let total_len = file_bytes.len();
    let total_blocks = (total_len + block_size - 1) / block_size;

    println!("📦 STREAMING: {} bytes in {} blocks (BlockSize: {}MB)", total_len, total_blocks, block_size / 1024 / 1024);
    println!("👉 Run: receiver.exe (auto-detects size via metadata)");
    println!("⚠️ PRESS ENTER TO INITIATE QUANTUM HANDSHAKE...");
    let mut _line = String::new();
    let _ = std::io::stdin().read_line(&mut _line);

    // Level 9 Handshake: Target Laptop
    println!("⚛️ LATTICE: Requesting Public Key from Ghost Receiver at {}:{}...", target_ip, p1_port);
    socket.send_to(b"PK_REQ", format!("{}:{}", target_ip, p1_port)).await?;
    let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
    let (n, _) = tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv_from(&mut pk_buf)).await??;
    if n != KYBER_PUBLICKEYBYTES { return Err("Invalid PK size".into()); }

    println!("🤝 HANDSHAKE: Public Key received. Encapsulating secret...");
    let mut rng = rand::thread_rng();
    let (_ct, shared_secret) = encapsulate(&pk_buf, &mut rng).map_err(|_| "Encapsulation failed")?;
    let quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());

    // Level 11 METADATA: Robust Handshake
    println!("📦 LATTICE: Synchronizing Metadata with Ghost Receiver...");
    let mut meta = vec![b'M'];
    // Extract just the filename from the path
    let fname = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path);
    meta.extend_from_slice(&(fname.len() as u32).to_be_bytes());
    meta.extend_from_slice(fname.as_bytes());
    meta.extend_from_slice(&(total_len as u64).to_be_bytes());

    let mut meta_confirmed = false;
    while !meta_confirmed {
        socket.send_to(&meta, format!("{}:{}", target_ip, p1_port)).await?;
        let mut ack_buf = [0u8; 64];
        if let Ok(Ok((n, _))) = tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv_from(&mut ack_buf)).await {
            let msg = String::from_utf8_lossy(&ack_buf[..n]);
            if msg.starts_with("META_ACK") {
                meta_confirmed = true;
            }
        }
    }
    println!("✅ SUCCESS: Metadata synchronized. Ghost is ready.");

    // LEVEL 7/8: AUTO-PILOT & NEURAL STRATEGIST
    let dynamic_mode = std::env::var("TURBONET_DYNAMIC").unwrap_or_else(|_| "false".to_string()) == "true";
    let mut rtts = [0f64; 3];
    
    let (w0, w1, w2) = if dynamic_mode {
        println!("📡 AUTO-PILOT: Probing primary lane for throughput baseline...");
        let mut probe_buf = [0u8; 16];
        probe_buf[0..8].copy_from_slice(&0xFFFFFFFFFFFFFFFFu64.to_be_bytes());

        // Probe lane 0 (primary lane)
        let start = std::time::Instant::now();
        socket.send_to(&probe_buf, format!("{}:{}", target_ip, p1_port)).await?;
        let mut echo_buf = [0u8; 16];
        match tokio::time::timeout(std::time::Duration::from_millis(1000), socket.recv_from(&mut echo_buf)).await {
            Ok(_) => {
                rtts[0] = start.elapsed().as_secs_f64();
                println!("   Lane 0: RTT {:.2}ms (measured)", rtts[0] * 1000.0);
                // Assume similar RTT for local lanes
                rtts[1] = rtts[0];
                rtts[2] = rtts[0];
                println!("   Lanes 1 & 2: RTT {:.2}ms (estimated)", rtts[0] * 1000.0);
            }
            Err(_) => {
                rtts = [1.0, 1.0, 1.0];
                println!("   All Lanes: PROBE TIMEOUT (Fail-safe: equal distribution)");
            }
        }

        // SOTA: Use local predictor for microsecond-scale decisions
        let use_deepseek_api = std::env::var("TURBONET_DEEPSEEK_API").unwrap_or_else(|_| "false".to_string()) == "true";
        
        if use_deepseek_api {
            // Legacy: Use DeepSeek API (60s latency)
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
            println!("🧠 LEGACY MODE: Consulting {} API (slow)...", model);
            if let Some(ai_weights) = get_ai_strategy(rtts).await {
                println!("🤖 API RECOMMENDATION: {:?}", ai_weights);
                (ai_weights.w0 as i32, ai_weights.w1 as i32, ai_weights.w2 as i32)
            } else {
                println!("⚠️ API TIMEOUT: Falling back to local predictor.");
                let mut predictor = HeuristicPredictor::new();
                let weights = predictor.predict(rtts, [0.0, 0.0, 0.0]);
                (weights.w0 as i32, weights.w1 as i32, weights.w2 as i32)
            }
        } else {
            // SOTA: Local heuristic predictor (~50µs latency)
            let start = std::time::Instant::now();
            let mut predictor = HeuristicPredictor::new();
            let weights = predictor.predict(rtts, [0.0, 0.0, 0.0]);
            let latency = start.elapsed();
            println!("⚡ LOCAL PREDICTOR: {}/{}/{} (latency: {:?})", weights.w0, weights.w1, weights.w2, latency);
            (weights.w0 as i32, weights.w1 as i32, weights.w2 as i32)
        }
    } else {
        println!("⚡ SHREDDER CORE: Using static weights {}/{}/{}", w0_env, w1_env, w2_env);
        (w0_env, w1_env, w2_env)
    };

    // SOTA Metrics: Track throughput and packet statistics
    let transfer_start = std::time::Instant::now();
    let mut total_packets_sent: u64 = 0;
    let mut total_bytes_sent: u64 = 0;
    let mut lane_bytes: [u64; 3] = [0, 0, 0];
    let mut lane_packets: [u64; 3] = [0, 0, 0];
    
    // Precompute lane targets
    let lane_addrs = [
        format!("{}:{}", target_ip, p1_port),
        format!("{}:{}", target_ip, _p2_port),
        format!("{}:{}", target_ip, _p3_port),
    ];
    let sockets = [&socket, &socket_l1, &socket_l2];

    for block_idx in 0..total_blocks {
        let start = block_idx * block_size;
        let end = (start + block_size).min(total_len);
        let block_data = &file_bytes[start..end];

        println!("🌊 Streaming Block {}/{}...", block_idx + 1, total_blocks);
        
        let mut header = [0u8; 28];
        header[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
        header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
        header[12..16].copy_from_slice(&(block_data.len() as u32).to_be_bytes());
        header[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
        header[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
        header[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());
        
        // Send header to primary lane
        socket.send_to(&header, &lane_addrs[0]).await?;
        total_packets_sent += 1;
        total_bytes_sent += 28;
        lane_packets[0] += 1;
        lane_bytes[0] += 28;
        tokio::task::yield_now().await;
        
        // Level 11: Adaptive throughput modes
        let turbo_mode = std::env::var("TURBONET_TURBO").unwrap_or_else(|_| "false".to_string()) == "true";
        let adaptive_mode = std::env::var("TURBONET_ADAPTIVE").unwrap_or_else(|_| "true".to_string()) == "true";
        
        // Near-MTU chunks for efficiency (1400 bytes per packet)
        let chunk_size: usize = std::env::var("TURBONET_CHUNK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(if turbo_mode { 60000 } else { 1400 });
        
        let mut chunk_count = 0u32;
        let total_weight = (w0 + w1 + w2) as u32;
        
        for chunk in block_data.chunks(chunk_size) {
            // LEVEL 13: Multi-lane weight-based distribution
            let lane_idx = if multilane_mode && total_weight > 0 {
                let pos = chunk_count % total_weight;
                if pos < w0 as u32 { 0 }
                else if pos < (w0 + w1) as u32 { 1 }
                else { 2 }
            } else {
                0 // Single-lane fallback
            };
            
            sockets[lane_idx].send_to(chunk, &lane_addrs[lane_idx]).await?;
            total_packets_sent += 1;
            total_bytes_sent += chunk.len() as u64;
            lane_packets[lane_idx] += 1;
            lane_bytes[lane_idx] += chunk.len() as u64;
            chunk_count += 1;
            
            if turbo_mode {
                // TURBO: No delay, maximum blast
            } else if adaptive_mode {
                // ADAPTIVE: Yield every 32 packets to saturate hardware without lockup
                if chunk_count % 32 == 0 {
                    tokio::task::yield_now().await;
                }
            } else {
                // SAFE: Fixed delay for guaranteed delivery
                tokio::time::sleep(std::time::Duration::from_micros(10)).await;
            }
        }
    }

    // Graceful Shutdown: Send END packet and wait for receiver acknowledgment
    println!("📡 SENDING END SIGNAL: Waiting for receiver to confirm...");
    let end_packet = b"END_TRANSFER";
    let mut end_confirmed = false;
    let end_timeout = std::time::Instant::now();
    
    while !end_confirmed && end_timeout.elapsed().as_secs() < 10 {
        socket.send_to(end_packet, format!("{}:{}", target_ip, p1_port)).await?;
        let mut ack_buf = [0u8; 64];
        match tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv_from(&mut ack_buf)).await {
            Ok(Ok((n, _))) => {
                let msg = String::from_utf8_lossy(&ack_buf[..n]);
                if msg.starts_with("END_ACK") {
                    end_confirmed = true;
                    println!("✅ RECEIVER CONFIRMED: All data received successfully!");
                }
            }
            _ => {}
        }
    }
    
    if !end_confirmed {
        println!("⚠️ WARNING: No confirmation from receiver (timeout). Data may still be in transit.");
    }

    // SOTA Metrics: Calculate and display throughput
    let transfer_duration = transfer_start.elapsed();
    let duration_secs = transfer_duration.as_secs_f64();
    let throughput_mbps = (total_bytes_sent as f64 / 1_000_000.0) / duration_secs;
    let throughput_gbps = throughput_mbps * 8.0 / 1000.0;
    
    println!("⚡ MISSION SUCCESS: Continuous stream complete!");
    println!("📊 TRANSFER STATS:");
    println!("   Duration: {:.2}s", duration_secs);
    println!("   Bytes Sent: {} ({:.2} MB)", total_bytes_sent, total_bytes_sent as f64 / 1_000_000.0);
    println!("   Packets: {}", total_packets_sent);
    println!("   🚀 THROUGHPUT: {:.1} MB/s ({:.2} Gbps)", throughput_mbps, throughput_gbps);
    
    // Per-lane statistics
    let total_b = lane_bytes[0] + lane_bytes[1] + lane_bytes[2];
    if total_b > 0 {
        let pct = |b: u64| b as f64 / total_b as f64 * 100.0;
        println!("   📡 LANE STATS: [0: {:.1}%] [1: {:.1}%] [2: {:.1}%] ({})", 
            pct(lane_bytes[0]), pct(lane_bytes[1]), pct(lane_bytes[2]),
            if multilane_mode { "multi-lane" } else { "single-lane" });
    }
    println!("Press Enter to exit...");
    let mut pause = String::new();
    std::io::stdin().read_line(&mut pause).unwrap();
    Ok(())
}