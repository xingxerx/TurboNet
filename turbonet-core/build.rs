
fn main() {
    println!("cargo:rerun-if-changed=shredder.cu");
    println!("cargo:rerun-if-changed=spectre.cu");

    // Allow skipping CUDA build if TURBONET_NO_CUDA is set
    if std::env::var("TURBONET_NO_CUDA").is_ok() {
        println!("cargo:warning=Skipping CUDA build (TURBONET_NO_CUDA set)");
        return;
    }

    // Skip CUDA build if PTX files already exist (pre-compiled)
    let shredder_exists = std::path::Path::new("shredder.ptx").exists();
    let spectre_exists = std::path::Path::new("spectre.ptx").exists();
    
    if shredder_exists && spectre_exists {
        // Both already compiled
        return;
    }


    // Compile shredder.cu to shredder.ptx using nvcc
    // On Windows, check if cl.exe is in PATH before running nvcc
    #[cfg(windows)]
    {
        use std::process::Command;
        let cl_check = Command::new("where")
            .arg("cl.exe")
            .output()
            .expect("Failed to check for cl.exe");
        if !cl_check.status.success() {
            eprintln!("\nERROR: Microsoft Visual C++ compiler (cl.exe) not found in PATH.\n\
Please open an 'x64 Native Tools Command Prompt for VS' and run 'cargo build' from there.\n\
See CUDA_BUILD_WINDOWS.md for details.\n");
            std::process::exit(1);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::process::Command;
        let nvcc_check = Command::new("which")
            .arg("nvcc")
            .output()
            .expect("Failed to check for nvcc");
        if !nvcc_check.status.success() {
            eprintln!("\nERROR: CUDA compiler (nvcc) not found in PATH.\n\
Please install the CUDA Toolkit and ensure nvcc is available.\n\
See CUDA_BUILD_LINUX_MAC.md for details.\n");
            std::process::exit(1);
        }
    }

    // Compile CUDA kernels to PTX using nvcc
    if !shredder_exists {
        let status = std::process::Command::new("nvcc")
            .args(&["-ptx", "shredder.cu", "-o", "shredder.ptx"])
            .status()
            .expect("Failed to compile shredder.cu");
        if !status.success() {
            panic!("nvcc failed for shredder.cu with status: {}", status);
        }
    }
    
    if !spectre_exists {
        let status = std::process::Command::new("nvcc")
            .args(&["-ptx", "spectre.cu", "-o", "spectre.ptx"])
            .status()
            .expect("Failed to compile spectre.cu");
        if !status.success() {
            panic!("nvcc failed for spectre.cu with status: {}", status);
        }
    }
}
