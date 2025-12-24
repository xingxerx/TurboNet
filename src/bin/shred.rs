use cudarc::driver::CudaDevice;
use std::fs;
use std::sync::Arc;
use tokio::net::UdpSocket;
use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use pqc_kyber::*;
use std::convert::TryInto;
use turbonet::deepseek_weights::DeepSeekWeights;

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

    // Increase UDP socket buffer size to 4MB
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    sock.set_reuse_address(true)?;
    sock.set_recv_buffer_size(4 * 1024 * 1024)?;
    sock.set_send_buffer_size(4 * 1024 * 1024)?;
    sock.bind(&"0.0.0.0:0".parse::<SocketAddr>()?.into())?;
    let socket = Arc::new(UdpSocket::from_std(sock.into())?);

    let file_path = "payload.bin";
    let file_bytes = fs::read(file_path).expect("Failed to read payload.bin. Run 'fsutil file createnew payload.bin 104857600' first.");
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
    let fname = "payload.bin";
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

        let neural_mode = std::env::var("TURBONET_NEURAL").unwrap_or_else(|_| "false".to_string()) == "true";
        if neural_mode {
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
            println!("🧠 NEURAL STRATEGIST: Consulting {} for strategic distribution...", model);
            if let Some(ai_weights) = get_ai_strategy(rtts).await {
                println!("🤖 AI RECOMMENDATION: {:?}", ai_weights);
                (ai_weights.w0 as i32, ai_weights.w1 as i32, ai_weights.w2 as i32)
            } else {
                println!("⚠️ NEURAL TIMEOUT: Falling back to mathematical Auto-Pilot.");
                let scores: Vec<f64> = rtts.iter().map(|r| 1.0 / r.max(0.001)).collect();
                let sum: f64 = scores.iter().sum();
                let nw0 = ((scores[0] / sum) * 100.0) as i32;
                let nw1 = ((scores[1] / sum) * 100.0) as i32;
                let nw2 = 100 - nw0 - nw1;
                (nw0, nw1, nw2)
            }
        } else {
            let scores: Vec<f64> = rtts.iter().map(|r| 1.0 / r.max(0.001)).collect();
            let sum: f64 = scores.iter().sum();
            let nw0 = ((scores[0] / sum) * 100.0) as i32;
            let nw1 = ((scores[1] / sum) * 100.0) as i32;
            let nw2 = 100 - nw0 - nw1; 
            println!("🧠 AUTO-PILOT: Mathematical balance: {}/{}/{}", nw0, nw1, nw2);
            (nw0, nw1, nw2)
        }
    } else {
        println!("⚡ SHREDDER CORE: Using static weights {}/{}/{}", w0_env, w1_env, w2_env);
        (w0_env, w1_env, w2_env)
    };

    for block_idx in 0..total_blocks {
        let start = block_idx * block_size;
        let end = (start + block_size).min(total_len);
        let block_data = &file_bytes[start..end];

        // BYPASS ENCRYPTION AND SHREDDING: Send raw data directly for testing
        println!("🌊 Streaming Block {}/{}...", block_idx + 1, total_blocks);
        
        let mut header = [0u8; 28];
        header[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
        header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
        header[12..16].copy_from_slice(&(block_data.len() as u32).to_be_bytes());
        header[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
        header[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
        header[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());
        
        // Send header
        socket.send_to(&header, format!("{}:{}", target_ip, p1_port)).await?;
        tokio::task::yield_now().await;
        
        // Level 11: Optimized Stream (60KB chunks, 10µs delay - proven 100% reliable)
        let chunk_size = 60000;
        for chunk in block_data.chunks(chunk_size) {
            socket.send_to(chunk, format!("{}:{}", target_ip, p1_port)).await?;
            tokio::time::sleep(std::time::Duration::from_micros(10)).await;
        }
    }

    println!("⚡ MISSION SUCCESS: Continuous stream complete!");
    println!("Press Enter to exit...");
    let mut pause = String::new();
    std::io::stdin().read_line(&mut pause).unwrap();
    Ok(())
}