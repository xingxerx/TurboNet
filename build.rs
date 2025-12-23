
fn main() {
    println!("cargo:rerun-if-changed=shredder.cu");
    // Compile shredder.cu to shredder.ptx using nvcc
    let status = std::process::Command::new("nvcc")
        .args(&["-ptx", "shredder.cu", "-o", "shredder.ptx"])
        .status()
        .expect("Failed to compile CUDA kernel");
    if !status.success() {
        panic!("nvcc failed with status: {}", status);
    }
}
