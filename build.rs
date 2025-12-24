
fn main() {
    println!("cargo:rerun-if-changed=shredder.cu");

    // Allow skipping CUDA build if TURBONET_NO_CUDA is set
    if std::env::var("TURBONET_NO_CUDA").is_ok() {
        println!("cargo:warning=Skipping CUDA build (TURBONET_NO_CUDA set)");
        return;
    }

    // Skip CUDA build if shredder.ptx already exists (pre-compiled)
    if std::path::Path::new("shredder.ptx").exists() {
        println!("cargo:warning=Using existing shredder.ptx (skip CUDA compilation)");
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

    // Compile shredder.cu to shredder.ptx using nvcc
    let status = std::process::Command::new("nvcc")
        .args(&["-ptx", "shredder.cu", "-o", "shredder.ptx"])
        .status()
        .expect("Failed to compile CUDA kernel");
    if !status.success() {
        panic!("nvcc failed with status: {}", status);
    }
}
