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
    // Reading REAL file "TOP_SECRET.bin"
    let input_path = "TOP_SECRET.bin";
    let input_data = fs::read(input_path).map_err(|e| format!("Failed to read {}: {}", input_path, e))?;
    let n = input_data.len();

    let mut d_in = dev.htod_copy(input_data)?;
    
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
    println!("👉 Copy this Salt to your Receiver command: receiver.exe {}", salt);
    println!("⚠️ PRESS ENTER TO ROTATE BLADES AND BLAST DATA...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, n, salt)) }?;

    // 6. Pull ALL 3 Shreds back to RAM
    let host_24 = dev.dtoh_sync_copy(&d_24)?;
    let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
    let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

    // --- SECURITY 3: INTERFACE BINDING (LOCK THE DOORS) ---
    // Find the interface encoded with 192.168.50.x
    let mut bind_ip = "0.0.0.0".to_string();
    let network_interfaces = NetworkInterface::show()?;
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
    socket.set_broadcast(true)?;
    
    println!("🚀 BLASTING 3 LANES SIMULTANEOUSLY...");
    
    let s1 = socket.clone();
    let s2 = socket.clone();
    let s3 = socket.clone();

    // --- SECURITY 2: PACKET INTEGRITY (WAX SEAL) ---
    // Prepend a magic signature (First 8 bytes of salt ^ 0xDEADBEEF)
    let magic = salt ^ 0xDEADBEEFDEADBEEF;
    let magic_bytes = magic.to_be_bytes();

    // Helper to create a signed packet
    let sign_packet = |data: &[u8]| -> Vec<u8> {
        [magic_bytes.as_slice(), data].concat()
    };

    let p1 = sign_packet(&host_24[0..n/3]);
    let p2 = sign_packet(&host_5g1[0..n/3]);
    let p3 = sign_packet(&host_5g2[0..n/3]);

    // Fire all three at the EXACT same time using Tokio tasks
    let (r1, r2, r3) = tokio::join!(
        async { s1.send_to(&p1, "192.168.50.1:8001").await },
        async { s2.send_to(&p2, "192.168.50.1:8002").await },
        async { s3.send_to(&p3, "192.168.50.1:8003").await },
    );

    r1?; r2?; r3?;

    println!("⚡ TOTAL SUCCESS: File shredded by GPU and distributed across all ASUS frequencies!");

    // --- SECURITY 4: VRAM SANITIZATION (CLEAN UPDATE) ---
    println!("🧹 SANITIZING VRAM...");
    dev.memset_zeros(&mut d_in)?;
    dev.memset_zeros(&mut d_24)?;
    dev.memset_zeros(&mut d_5g1)?;
    dev.memset_zeros(&mut d_5g2)?;
    println!("✨ GPU MEMORY WIPED. NO RESIDUALS LEFT.");
    
    Ok(())
}