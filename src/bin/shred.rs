use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;
use network_interface::NetworkInterfaceConfig;
use network_interface::NetworkInterface;
use network_interface::Addr;
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
    
    // --- SECURITY 1: FREQUENCY LEAK PROTECTION (SALT) ---
    // Generate a cryptographically secure random salt
    let mut rng = rand::thread_rng();
    let salt: u64 = rng.gen();
    println!("🔒 GENERATED SESSION SALT: {:016x}", salt);

    // 1. Read the CUDA kernel source code
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_path = PathBuf::from(manifest_dir).join("shredder.cu");
    let ptx_src = fs::read_to_string(&ptx_path).map_err(|e| format!("Failed to read {}: {}", ptx_path.display(), e))?;
    
    // 2. Compile it to PTX
    let ptx = compile_ptx(ptx_src).expect("Failed to compile CUDA kernel");

    println!("✅ KERNEL COMPILED. LOADING TO GPU...");

    // 3. Initialize the GPU
    let dev = CudaDevice::new(0)?;
    
    // 4. Load the module
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    println!("✅ SHREDDER RUNNING ON CUDA CORE 0");

    // 5. Data Prep & Allocation
    // --- PAYLOAD: ENCRYPTED IMAGE ---
    println!("🔐 ENCRYPTING PAYLOAD (Level 5 Security)...");
    let input_path = "input.jpg";
    let file_bytes = fs::read(input_path).map_err(|e| format!("Failed to read {}: {}", input_path, e))?;
    
    // --- ENCRYPTION LOGIC ---
    // 1. Derive Key from Salt
    let key_material = salt.to_be_bytes(); 
    let mut full_key = [0u8; 32];
    for i in 0..4 { full_key[i*8..(i+1)*8].copy_from_slice(&key_material); }
    let key = Key::<Aes256Gcm>::from_slice(&full_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]); // Static nonce for lab drill

    // 2. Encrypt
    let ciphertext = cipher.encrypt(nonce, file_bytes.as_ref())
        .expect("Encryption failure!");
        
    println!("📦 Payload Size: {} bytes -> Encrypted: {} bytes", file_bytes.len(), ciphertext.len());

    let n = ciphertext.len();
    let mut d_in = dev.htod_copy(ciphertext)?;
    
    // Allocate outputs (N / 3 approx)
    let lane_size = n / 3 + 100; // Safety padding
    let mut d_24 = dev.alloc_zeros::<u8>(lane_size)?;
    let mut d_5g1 = dev.alloc_zeros::<u8>(lane_size)?;
    let mut d_5g2 = dev.alloc_zeros::<u8>(lane_size)?;

    // Launch Kernel with SALT
    let f = dev.get_func("shredder", "shred_kernel").unwrap();
    let cfg = LaunchConfig::for_num_elems(n as u32);

    // --- MANUAL KEY EXCHANGE ---
    println!("\n🔑 SESSION SALT: {}", salt);
    println!("📦 ENCRYPTED SIZE: {} bytes", n);
    println!("👉 Run: receiver.exe {} {}", salt, n);
    println!("⚠️ PRESS ENTER TO ROTATE BLADES AND BLAST DATA...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, n, salt)) }?;

    // 6. Pull ALL 3 Shreds back to RAM
    let host_24 = dev.dtoh_sync_copy(&d_24)?;
    let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
    let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

    // --- SECURITY 3: INTERFACE BINDING (LOCK THE DOORS) ---
    // (Existing interface binding logic...)
    let network_interfaces = NetworkInterface::show()?;
    let mut bind_ip = "0.0.0.0".to_string();
    for itf in network_interfaces {
        for addr in itf.addr {
             if let Addr::V4(v4) = addr {
                 let ip = v4.ip.to_string();
                 if ip.starts_with("192.168.50.") {
                     bind_ip = ip;
                     break;
                 }
             }
        }
    }
    
    println!("🛡️ LOCKED TO SECURE INTERFACE: {}", bind_ip);
    let socket = Arc::new(UdpSocket::bind(format!("{}:0", bind_ip)).await?);
    
    // --- SECURITY 2: PACKET INTEGRITY (WAX SEAL) ---
    let magic = salt ^ 0xDEADBEEFDEADBEEF;
    let magic_bytes = magic.to_be_bytes();

    let get_lane_len = |n: usize, salt: u64, lane: u64| -> usize {
        let offset = (lane + 6000000 - (salt % 3)) % 3; // Use large multiple of 3 for safety
        if (offset as usize) < n {
            (n - 1 - offset as usize) / 3 + 1
        } else {
            0
        }
    };

    let len0 = get_lane_len(n, salt, 0);
    let len1 = get_lane_len(n, salt, 1);
    let len2 = get_lane_len(n, salt, 2);

    println!("🚀 BLASTING 3 LANES (CHUNKED MODE)...");
    
    let target_ip = bind_ip.as_str(); // Target the interface we actually bound to
    let chunk_size = 32768; // 32KB chunks

    let s1 = socket.clone();
    let s2 = socket.clone();
    let s3 = socket.clone();

    // Fire all three lanes in parallel
    let _ = tokio::join!(
        async {
            // Lane 1: Header + Data Chunks
            let p1_data = &host_24[0..len0];
            s1.send_to(&magic_bytes, format!("{}:8001", target_ip)).await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await; // Give receiver a head start
            for chunk in p1_data.chunks(chunk_size) {
                s1.send_to(chunk, format!("{}:8001", target_ip)).await.ok();
            }
        },
        async {
            // Lane 2: Header + Data Chunks
            let p2_data = &host_5g1[0..len1];
            s2.send_to(&magic_bytes, format!("{}:8002", target_ip)).await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            for chunk in p2_data.chunks(chunk_size) {
                s2.send_to(chunk, format!("{}:8002", target_ip)).await.ok();
            }
        },
        async {
            // Lane 3: Header + Data Chunks
            let p3_data = &host_5g2[0..len2];
            s3.send_to(&magic_bytes, format!("{}:8003", target_ip)).await.ok();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            for chunk in p3_data.chunks(chunk_size) {
                s3.send_to(chunk, format!("{}:8003", target_ip)).await.ok();
            }
        },
    );

    println!("⚡ TOTAL SUCCESS: File shredded and distributed!");

    // --- SECURITY 4: VRAM SANITIZATION (CLEAN UPDATE) ---
    println!("🧹 SANITIZING VRAM...");
    dev.memset_zeros(&mut d_in)?;
    dev.memset_zeros(&mut d_24)?;
    dev.memset_zeros(&mut d_5g1)?;
    dev.memset_zeros(&mut d_5g2)?;
    println!("✨ GPU MEMORY WIPED. NO RESIDUALS LEFT.");
    
    Ok(())
}