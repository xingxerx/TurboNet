# TurboNet CUDA Build Instructions (Linux/Mac)

## Prerequisites
- CUDA Toolkit installed (nvcc in PATH)
- Compatible GPU and drivers

## Build Steps
1. Open a terminal.
2. Navigate to your project directory:
   ```
   cd /path/to/TurboNet
   ```
3. Build with Cargo:
   ```
   cargo build
   ```

## Troubleshooting
- If you see `nvcc: command not found`, ensure CUDA Toolkit is installed and `nvcc` is in your PATH.
- If you want to skip CUDA, comment out the `nvcc` command in `build.rs` (not recommended for production).

## For Developers
- See `build.rs` for the CUDA build step.
- If you want to make CUDA optional, see the cross-platform notes in this repo.
