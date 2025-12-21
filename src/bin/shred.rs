use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use serde::{Deserialize, Serialize};
use pqc_kyber::*;
use std::convert::TryInto;

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

#[derive(Deserialize, Debug)]
struct AiWeights {
    w0: i32,
    w1: i32,
    w2: i32,
}

fn get_hits(t: u64, w_total: u64, w_target: u64, offset: u64) -> u64 {
    if t == 0 { return 0; }
    let cycles = t / w_total;
    let rem = t % w_total;
    let mut hits = cycles * w_target;
    if rem > offset {
        hits += (rem - offset).min(w_target);
    }
    hits
}

fn get_lane_len_asymmetric(n: usize, salt: u64, w0: i32, w1: i32, w2: i32, lane: i32) -> usize {
    let w_total = (w0 + w1 + w2) as u64;
    let pattern_offset = salt % w_total;
    let t_start = pattern_offset;
    let t_end = pattern_offset + n as u64;
    
    let (w_target, offset) = match lane {
        0 => (w0 as u64, 0u64),
        1 => (w1 as u64, w0 as u64),
        2 => (w2 as u64, (w0 + w1) as u64),
        _ => unreachable!(),
    };

    let hits_end = get_hits(t_end, w_total, w_target, offset);
    let hits_start = get_hits(t_start, w_total, w_target, offset);
    (hits_end - hits_start) as usize
}

async fn get_ai_strategy(rtt_data: [f64; 3]) -> Option<AiWeights> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
    
    let prompt = format!(
        "Return a JSON object with weights for Lane 0, 1, and 2 based on these RTTs: Lane 0: {:.2}ms, Lane 1: {:.2}ms, Lane 2: {:.2}ms. \
        The weights must sum to 100. Lower RTT = higher weight. \
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
                if let Ok(weights) = serde_json::from_str::<AiWeights>(&json_resp.response) {
                    return Some(weights);
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
    
    let target_ip = std::env::var("TURBONET_TARGET_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
    
    let w0_env: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    let w1_env: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let w2_env: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_src = fs::read_to_string(PathBuf::from(manifest_dir).join("shredder.cu"))?;
    let ptx = compile_ptx(ptx_src)?;
    let dev = CudaDevice::new(0)?;
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

    let file_path = "input.jpg";
    let file_bytes = fs::read(file_path)?;
    let total_len = file_bytes.len();
    let total_blocks = (total_len + block_size - 1) / block_size;

    println!("📦 STREAMING: {} bytes in {} blocks (BlockSize: {}MB)", total_len, total_blocks, block_size / 1024 / 1024);
    println!("👉 Run: receiver.exe {}", total_len);
    println!("⚠️ PRESS ENTER TO INITIATE QUANTUM HANDSHAKE...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    // LEVEL 9: QUANTUM HANDSHAKE
    println!("⚛️ LATTICE: Requesting Public Key from Ghost Receiver...");
    socket.send_to(b"PK_REQ", format!("{}:{}", target_ip, p1_port)).await?;
    let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
    let (n, _) = tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv_from(&mut pk_buf)).await??;
    if n != KYBER_PUBLICKEYBYTES { return Err("Invalid PK size".into()); }

    println!("🤝 HANDSHAKE: Public Key received. Encapsulating secret...");
    let mut rng = rand::thread_rng();
    let (ct, shared_secret) = encapsulate(&pk_buf, &mut rng).map_err(|_| "Encapsulation failed")?;
    let quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());

    // Quantum Mesh: Shred Kyber Ciphertext across 3 lanes
    println!("⚛️ LATTICE: Shredding Ciphertext across 3 physical lanes...");
    let chunk_size_ct = (ct.len() + 2) / 3;
    let f1 = &ct[0..chunk_size_ct];
    let f2 = &ct[chunk_size_ct..chunk_size_ct*2];
    let f3 = &ct[chunk_size_ct*2..];

    socket.send_to(f1, format!("{}:{}", target_ip, p1_port)).await?;
    socket.send_to(f2, format!("{}:{}", target_ip, p2_port)).await?;
    socket.send_to(f3, format!("{}:{}", target_ip, p3_port)).await?;

    println!("✅ SUCCESS: Quantum Handshake complete. Session locked.");

    // LEVEL 7/8: AUTO-PILOT & NEURAL STRATEGIST
    let dynamic_mode = std::env::var("TURBONET_DYNAMIC").unwrap_or_else(|_| "false".to_string()) == "true";
    let mut rtts = [0f64; 3];
    
    let (w0, w1, w2) = if dynamic_mode {
        println!("📡 AUTO-PILOT: Probing lanes for optimal throughput...");
        let ports = [p1_port, p2_port, p3_port];
        let mut probe_buf = [0u8; 16];
        probe_buf[0..8].copy_from_slice(&0xFFFFFFFFFFFFFFFFu64.to_be_bytes());

        for i in 0..3 {
            let start = std::time::Instant::now();
            socket.send_to(&probe_buf, format!("{}:{}", target_ip, ports[i])).await?;
            let mut echo_buf = [0u8; 16];
            match tokio::time::timeout(std::time::Duration::from_millis(1000), socket.recv_from(&mut echo_buf)).await {
                Ok(_) => {
                    rtts[i] = start.elapsed().as_secs_f64();
                    println!("   Lane {}: RTT {:.2}ms", i, rtts[i] * 1000.0);
                }
                Err(_) => {
                    rtts[i] = 1.0; 
                    println!("   Lane {}: PROBE TIMEOUT (Fail-safe engaged)", i);
                }
            }
        }

        let neural_mode = std::env::var("TURBONET_NEURAL").unwrap_or_else(|_| "false".to_string()) == "true";
        if neural_mode {
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
            println!("🧠 NEURAL STRATEGIST: Consulting {} for strategic distribution...", model);
            if let Some(ai_weights) = get_ai_strategy(rtts).await {
                println!("🤖 AI RECOMMENDATION: {:?}", ai_weights);
                (ai_weights.w0, ai_weights.w1, ai_weights.w2)
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

        let key = Key::<Aes256Gcm>::from_slice(&shared_secret);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&(block_idx as u32).to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, block_data).expect("Block encryption failed");
        let encrypted_n = ciphertext.len();

        let d_in = dev.htod_copy(ciphertext.clone())?;
        let lane_size = encrypted_n + 1024; 
        let mut d_24 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g1 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g2 = dev.alloc_zeros::<u8>(lane_size)?;

        let w_total = (w0 + w1 + w2) as u64;
        let pattern_offset = quantum_salt % w_total;
        let i0 = get_hits(pattern_offset, w_total, w0 as u64, 0);
        let i1 = get_hits(pattern_offset, w_total, w1 as u64, w0 as u64);
        let i2 = get_hits(pattern_offset, w_total, w2 as u64, (w0 + w1) as u64);

        let cfg = LaunchConfig::for_num_elems(encrypted_n as u32);
        let f = dev.get_func("shredder", "shred_kernel").expect("Func not found");
        unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, encrypted_n as u64, quantum_salt, w0 as u64, w1 as u64, w2 as u64, i0, i1, i2)) }?;

        let host_24 = dev.dtoh_sync_copy(&d_24)?;
        let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
        let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

        let len0 = get_lane_len_asymmetric(encrypted_n, quantum_salt, w0, w1, w2, 0);
        let len1 = get_lane_len_asymmetric(encrypted_n, quantum_salt, w0, w1, w2, 1);
        let len2 = get_lane_len_asymmetric(encrypted_n, quantum_salt, w0, w1, w2, 2);

        let mut header = [0u8; 28];
        header[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
        header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
        header[12..16].copy_from_slice(&(encrypted_n as u32).to_be_bytes());
        header[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
        header[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
        header[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());

        let chunk_size = 1024;

        async fn blast_lane(s: &UdpSocket, data: &[u8], port: u16, target_ip: &str, head: &[u8], chunk_size: usize) {
            let _ = s.send_to(head, format!("{}:{}", target_ip, port)).await;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            for chunk in data.chunks(chunk_size) {
                let _ = s.send_to(chunk, format!("{}:{}", target_ip, port)).await;
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        }

        println!("🌊 Streaming Block {}/{}...", block_idx + 1, total_blocks);
        let _ = tokio::join!(
            blast_lane(&socket, &host_24[0..len0], p1_port, &target_ip, &header, chunk_size),
            blast_lane(&socket, &host_5g1[0..len1], p2_port, &target_ip, &header, chunk_size),
            blast_lane(&socket, &host_5g2[0..len2], p3_port, &target_ip, &header, chunk_size),
        );
    }

    println!("⚡ MISSION SUCCESS: Continuous stream complete!");
    Ok(())
}