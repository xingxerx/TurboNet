# 🚀 TurboNet: High-Performance GPU Network Shredder

**TurboNet** is a specialized High-Performance Computing (HPC) node designed to shred files using GPU kernels and broadcast them simultaneously across multiple network lanes using 10Gbps Ethernet.

## ⚡ Core Architecture

1.  **GPU Shredding**: Uses a custom CUDA kernel (`shredder.cu`) to split data into three logical frequency bands (2.4GHz, 5GHz-1, 5GHz-2).
2.  **Quantum Parallelism**: Blasts data across three distinct UDP ports (`8001`, `8002`, `8003`) simultaneously using `tokio::join!`.
3.  **Router Bounce**: Bypasses Windows loopback restrictions by targeting the router (`192.168.50.1`) and catching the reflected packets on the return trip.

## 🛡️ Level 4 Security Protocol

This system implements military-grade data protection:

*   **Frequency Leak Protection**: A cryptographic `Salt` is generated per session to jitter the shredding pattern, preventing side-channel analysis.
*   **Digital Wax Seal**: Every packet is signed with a 64-bit header (`Salt ^ 0xDEADBEEF...`) to verify integrity.
*   **Interface Lockdown**: attempting to bind specific high-speed interfaces.
*   **VRAM Sanitization**: GPU memory is overwritten with zeros before termination to prevent data forensics.

## 🛠️ Prerequisites

-   **Hardware**: NVIDIA GPU (CUDA Capable), 10Gbps Ethernet Card.
-   **Software**: Rust (Cargo), CUDA Toolkit v13.0+.
-   **Network**: ASUS GT-AX11000 Pro (or similar high-performance router) at `192.168.50.1`.

## 🚦 The "Ghost Receiver" Drill

To verify the pipeline, perform the **Manual Key Exchange**:

### 1. Launch the Receiver (The Ghost)
The receiver waits for the specific session signature.
```bash
cargo run --release --bin receiver -- <SALT_VALUE>
```

### 2. Launch the Broadcaster (The Shredder)
Generates the Salt and prepares the GPU.
```bash
cargo run --release --bin shred
```
*Copy the `SESSION SALT` displayed here and paste it into the Receiver command.*

### 3. Engage
Press **ENTER** on the Shredder terminal.
-   **Shredder**: Encrypts, Splits, Signs, and Blasts.
-   **Receiver**: Catches reflected packets, Verifies Signatures, and Re-assembles.

## 📂 Project Structure

-   `src/bin/shred.rs`: The Broadcaster. GPU management, Kernel Launch, UDP Blast.
-   `src/bin/receiver.rs`: The Receiver. Signature Verification, Packet Re-assembly.
-   `shredder.cu`: The CUDA Kernel. Handles the mathematical splitting/salting logic.
-   `build.rs`: Automates the NVRTC DLL setup for seamless building.
