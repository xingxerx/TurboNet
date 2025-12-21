use cudarc::driver::CudaDevice;
use cudarc::nvrtc::compile_ptx;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 ROTATING SHREDDER BLADES...");
    
    // 1. Read the CUDA kernel source code
    // Use the crate root to find the file reliably, regardless of where the binary is run from
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ptx_path = PathBuf::from(manifest_dir).join("shredder.cu");
    let ptx_src = fs::read_to_string(&ptx_path).map_err(|e| format!("Failed to read {}: {}", ptx_path.display(), e))?;
    
    // 2. Compile it to PTX (Parallel Thread Execution) assembly
    // We use compile_ptx since compile_file might not be available or requires reading content anyway
    let ptx = compile_ptx(ptx_src).expect("Failed to compile CUDA kernel");

    println!("✅ KERNEL COMPILED. LOADING TO GPU...");

    // 3. Initialize the GPU
    let dev = CudaDevice::new(0)?;
    
    // 4. Load the module
    // The image name is 'shred_kernel' effectively, but we load it into the module
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    println!("✅ SHREDDER RUNNING ON CUDA CORE 0");
    
    Ok(())
}