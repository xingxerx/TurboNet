use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use rand::Rng;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("🔥 TURBONET: DEEP-SEA ASYMMETRIC SHREDDER v2.0 ENGAGED...");
    
    let target_ip = std::env::var("TURBONET_TARGET_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
    
    let w0: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    let w1: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let w2: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let mut rng = rand::thread_rng();
    let salt: u64 = rng.gen();
    
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_src = fs::read_to_string(PathBuf::from(manifest_dir).join("shredder.cu"))?;
    let ptx = compile_ptx(ptx_src)?;
    let dev = CudaDevice::new(0)?;
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    let file_path = "input.jpg";
    let file_bytes = fs::read(file_path)?;
    let total_len = file_bytes.len();
    let total_blocks = (total_len + block_size - 1) / block_size;

    println!("⚡ SHREDDER CORE: Loaded Weights {}/{}/{}", w0, w1, w2);
    println!("📦 STREAMING: {} bytes in {} blocks (BlockSize: {}MB)", total_len, total_blocks, block_size / 1024 / 1024);
    println!("\n🔑 SESSION SALT: {}", salt);
    println!("👉 Run: receiver.exe {} {}", salt, total_len);
    println!("⚠️ PRESS ENTER TO BLAST DATA...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

    for block_idx in 0..total_blocks {
        let start = block_idx * block_size;
        let end = (start + block_size).min(total_len);
        let block_data = &file_bytes[start..end];

        // 1. Encrypt block
        let key_material = salt.to_be_bytes();
        let mut full_key = [0u8; 32];
        for i in 0..4 { full_key[i*8..(i+1)*8].copy_from_slice(&key_material); }
        let key = Key::<Aes256Gcm>::from_slice(&full_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&(block_idx as u32).to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, block_data).expect("Block encryption failed");
        let encrypted_n = ciphertext.len();

        // 2. Shred on GPU
        let d_in = dev.htod_copy(ciphertext.clone())?;
        let lane_size = encrypted_n + 1024; // Buffer for safety
        let mut d_24 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g1 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g2 = dev.alloc_zeros::<u8>(lane_size)?;

        let w_total = (w0 + w1 + w2) as u64;
        let pattern_offset = salt % w_total;
        let i0 = get_hits(pattern_offset, w_total, w0 as u64, 0);
        let i1 = get_hits(pattern_offset, w_total, w1 as u64, w0 as u64);
        let i2 = get_hits(pattern_offset, w_total, w2 as u64, (w0 + w1) as u64);

        let cfg = LaunchConfig::for_num_elems(encrypted_n as u32);
        let f = dev.get_func("shredder", "shred_kernel").expect("Func not found");
        unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, encrypted_n as u64, salt, w0 as u64, w1 as u64, w2 as u64, i0, i1, i2)) }?;

        let host_24 = dev.dtoh_sync_copy(&d_24)?;
        let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
        let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

        let len0 = get_lane_len_asymmetric(encrypted_n, salt, w0, w1, w2, 0);
        let len1 = get_lane_len_asymmetric(encrypted_n, salt, w0, w1, w2, 1);
        let len2 = get_lane_len_asymmetric(encrypted_n, salt, w0, w1, w2, 2);

        // Header: [SALT(8)][BLOCK_INDEX(4)][ENCRYPTED_SIZE(4)]
        let mut header = [0u8; 16];
        header[0..8].copy_from_slice(&salt.to_be_bytes());
        header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
        header[12..16].copy_from_slice(&(encrypted_n as u32).to_be_bytes());

        let chunk_size = 1024;

        async fn blast_lane(s: &UdpSocket, data: &[u8], port: u16, target_ip: &str, head: &[u8], chunk_size: usize) {
            s.send_to(head, format!("{}:{}", target_ip, port)).await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            for chunk in data.chunks(chunk_size) {
                s.send_to(chunk, format!("{}:{}", target_ip, port)).await.ok();
                tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            }
        }

        println!("🌊 Streaming Block {}/{}...", block_idx + 1, total_blocks);
        blast_lane(&socket, &host_24[0..len0], p1_port, &target_ip, &header, chunk_size).await;
        blast_lane(&socket, &host_5g1[0..len1], p2_port, &target_ip, &header, chunk_size).await;
        blast_lane(&socket, &host_5g2[0..len2], p3_port, &target_ip, &header, chunk_size).await;
    }

    println!("⚡ MISSION SUCCESS: Continuous stream complete!");
    Ok(())
}