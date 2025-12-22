use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        // Source path for the DLL - hardcoded based on our previous findings
        let src_dll_path = PathBuf::from(r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0\bin\nvrtc64_120_0.dll"); // adjust version as needed
        
        // Determine output directory
        let _out_dir = env::var("OUT_DIR").unwrap();
        let profile = env::var("PROFILE").unwrap();
        
        // We need to place it where the executable is found.
        // OUT_DIR is deep inside target, so we need to copy to target/release or target/debug
        // A common trick is to copy to the directory of the executable which isn't easily exposed in build scripts,
        // but often 'target/profile' is a good guess relative to manifest.
        
        // However, robustly finding the *final* output directory from build.rs is tricky because artifacts go to `target/debug/deps` etc.
        // A simpler approach for the user is asking cargo to run a post-build script, but build.rs runs *before* compilation.
        
        // Strategy: Copy to the same directory as the manifest + target/profile
        // Note: usage of 'target' dir assumes standard layout.
        let target_dir = Path::new(&manifest_dir).join("target").join(&profile);
        
        let dest_dll_path = target_dir.join("nvrtc64.dll");

        if src_dll_path.exists() {
            // Ensure target directory exists
            fs::create_dir_all(&target_dir).ok();
            
            match fs::copy(&src_dll_path, &dest_dll_path) {
                Ok(_) => println!("Successfully copied nvrtc64.dll to {:?}", dest_dll_path),
                Err(e) => println!("cargo:warning=Failed to copy nvrtc64.dll: {}", e),
            }
        } else {
            println!("cargo:warning=Could not find CUDA DLL at {:?}", src_dll_path);
        }
    }
}
