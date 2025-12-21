use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use rand::Rng;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 ROTATING SHREDDER BLADES (LEVEL 4 SECURITY ENGAGED)...");
    
    let mut rng = rand::thread_rng();
    let salt: u64 = rng.gen();
    println!("🔒 GENERATED SESSION SALT: {:016x}", salt);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_path = PathBuf::from(manifest_dir).join("shredder.cu");
    let ptx_src = fs::read_to_string(&ptx_path).map_err(|e| format!("Failed to read {}: {}", ptx_path.display(), e))?;
    let ptx = compile_ptx(ptx_src).expect("Failed to compile CUDA kernel");

    let dev = CudaDevice::new(0)?;
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    println!("✅ SHREDDER RUNNING ON CUDA CORE 0");

    let input_path = "input.jpg";
    let file_bytes = fs::read(input_path).map_err(|e| format!("Failed to read {}: {}", input_path, e))?;
    
    let key_material = salt.to_be_bytes(); 
    let mut full_key = [0u8; 32];
    for i in 0..4 { full_key[i*8..(i+1)*8].copy_from_slice(&key_material); }
    let key = Key::<Aes256Gcm>::from_slice(&full_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]); 

    let ciphertext = cipher.encrypt(nonce, file_bytes.as_ref()).expect("Encryption failure!");
    let n = ciphertext.len();
    
    println!("📦 Payload Size: {} bytes -> Encrypted: {} bytes", file_bytes.len(), n);

    let mut d_in = dev.htod_copy(ciphertext)?;
    let lane_size = n / 3 + 100; 
    let mut d_24 = dev.alloc_zeros::<u8>(lane_size)?;
    let mut d_5g1 = dev.alloc_zeros::<u8>(lane_size)?;
    let mut d_5g2 = dev.alloc_zeros::<u8>(lane_size)?;

    let f = dev.get_func("shredder", "shred_kernel").unwrap();
    let cfg = LaunchConfig::for_num_elems(n as u32);

    println!("\n🔑 SESSION SALT: {}", salt);
    println!("📦 ENCRYPTED SIZE: {} bytes", n);
    println!("👉 Run: receiver.exe {} {}", salt, n);
    println!("⚠️ PRESS ENTER TO ROTATE BLADES AND BLAST DATA...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, n, salt)) }?;

    let host_24 = dev.dtoh_sync_copy(&d_24)?;
    let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
    let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

    // Bind to 0.0.0.0 for maximum loopback reach
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let magic = salt ^ 0xDEADBEEFDEADBEEF;
    let magic_bytes = magic.to_be_bytes();

    let get_lane_len = |n: usize, salt: u64, lane: u64| -> usize {
        let offset = (lane + 6000000 - (salt % 3)) % 3;
        if (offset as usize) < n { (n - 1 - offset as usize) / 3 + 1 } else { 0 }
    };

    let len0 = get_lane_len(n, salt, 0);
    let len1 = get_lane_len(n, salt, 1);
    let len2 = get_lane_len(n, salt, 2);

    // --- NETWORK CONFIG FROM .ENV ---
    dotenvy::dotenv().ok();
    let target_ip = std::env::var("TURBONET_TARGET_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let p1: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let p2: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let p3: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();

    let chunk_size = 1024; 

    println!("🚀 BLASTING LANES to {} (SEQUENTIAL MODE)...", target_ip);

    // --- SEQUENTIAL TRANSMISSION ---
    async fn blast_lane(s: &UdpSocket, data: &[u8], port: u16, target_ip: &str, magic: &[u8], chunk_size: usize) {
        println!("📡 Engaging Lane ({})...", port);
        s.send_to(magic, format!("{}:{}", target_ip, port)).await.ok();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        for chunk in data.chunks(chunk_size) {
            s.send_to(chunk, format!("{}:{}", target_ip, port)).await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
    }

    blast_lane(&socket, &host_24[0..len0], p1, &target_ip, &magic_bytes, chunk_size).await;
    blast_lane(&socket, &host_5g1[0..len1], p2, &target_ip, &magic_bytes, chunk_size).await;
    blast_lane(&socket, &host_5g2[0..len2], p3, &target_ip, &magic_bytes, chunk_size).await;

    println!("⚡ TOTAL SUCCESS: File shredded and distributed!");
    dev.memset_zeros(&mut d_in)?;
    dev.memset_zeros(&mut d_24)?;
    dev.memset_zeros(&mut d_5g1)?;
    dev.memset_zeros(&mut d_5g2)?;
    Ok(())
}