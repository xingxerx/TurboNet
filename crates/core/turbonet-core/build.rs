fn main() {
    println!("cargo:rerun-if-changed=fragment.cu");

    // Allow skipping CUDA build if TURBONET_NO_CUDA is set
    if std::env::var("TURBONET_NO_CUDA").is_ok() {
        println!("cargo:warning=Skipping CUDA build (TURBONET_NO_CUDA set)");
        return;
    }

    // Skip CUDA build if PTX file already exists (pre-compiled)
    let fragment_exists = std::path::Path::new("fragment.ptx").exists();
    if fragment_exists {
        return;
    }

    // Compile fragment.cu to fragment.ptx using nvcc
    // On Windows, check if cl.exe is in PATH before running nvcc
    #[cfg(windows)]
    {
        use std::process::Command;
        let cl_check = Command::new("where")
            .arg("cl.exe")
            .output()
            .expect("Failed to check for cl.exe");
        if !cl_check.status.success() {
            eprintln!(
                "\nERROR: Microsoft Visual C++ compiler (cl.exe) not found in PATH.\n\
Please open an 'x64 Native Tools Command Prompt for VS' and run 'cargo build' from there.\n\
See CUDA_BUILD_WINDOWS.md for details.\n"
            );
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
            eprintln!(
                "\nERROR: CUDA compiler (nvcc) not found in PATH.\n\
Please install the CUDA Toolkit and ensure nvcc is available.\n\
See CUDA_BUILD_LINUX_MAC.md for details.\n"
            );
            std::process::exit(1);
        }
    }

    // Compile CUDA kernel to PTX using nvcc
    let status = std::process::Command::new("nvcc")
        .args(["-ptx", "fragment.cu", "-o", "fragment.ptx"])
        .status()
        .expect("Failed to compile fragment.cu");
    if !status.success() {
        panic!("nvcc failed for fragment.cu with status: {}", status);
    }
}
