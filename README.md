# 🛰️ TurboNet: Post-Quantum AI Network Shredder (v5.0)

**TurboNet** is a state-of-the-art software suite designed for ultra-secure, AI-optimized data fragmentation. It bridges **GPU-accelerated shredding** with **Post-Quantum Cryptography** and an **AI Reasoning Engine** to create an un-interceptable data stream across multiple physical network bands.

---

## 🛠️ The Tech Stack

-   **Frontend**: `egui` + `eframe` (High-performance 60FPS Immediate Mode GUI).
-   **Security**: `pqc_kyber` (Kyber-768/ML-KEM) + `aes-gcm` (AES-256).
-   **Intelligence**: `DeepSeek-R1:8b` (Local LLM via Ollama).
-   **Performance**: `CUDA 13.0` (Hardware-parallel shredding).
-   **Network**: `Tokio` (Async UDP multi-band blasting).

---

## 🚀 Key Features

### ⚛️ Level 9: The Lattice-Based Ghost
TurboNet uses **Module-Lattice (ML-KEM)** math to perform a quantum-safe handshake. The session key is never shared over the wire; it is encapsulated and decrypted using Kyber-768, protecting your data against future quantum computers (**Harvest Now, Decrypt Later Resistance**).

### 🧠 Level 8: The Neural Strategist
Integrated **DeepSeek-R1** monitors your network lanes in real-time. If the 2.4GHz band gets congested while the 5GHz bands are clear, the AI automatically re-calculates the GPU shredding weights (w0, w1, w2) to shift traffic to the fastest path.

### 📦 Level 11: Payload Awareness (Metadata Handshake)
Both the CLI and GUI now synchronize file metadata (filename, size) with the receiver before transmission. The receiver automatically adapts to the incoming stream and saves the file as `reborn_<filename>`.

### 🖥️ Level 10: Mission Control Dashboard
A dedicated visual dashboard for selecting files, monitoring "Sonic Probes" (RTT gauges), and initiating the "Quantum Blast" with a single click. **Now fully synchronized with the v5.0 Metadata Protocol.**

### 🔐 Level 12: Cryptographic Hardening (v5.0 NEW)
- **Random Nonce Generation**: Each transfer uses a cryptographically secure 12-byte nonce (no more static nonces)
- **XOR Obfuscation**: CUDA kernel now applies salt-based XOR scrambling in-kernel before distribution
- **Decryption Support**: Full `decrypt_payload()` API for receiver-side decryption

### 📡 Level 13: Multi-Lane Parallel Sending (v5.0 NEW)
- **3-Socket Parallel UDP**: Sender creates 3 UDP sockets and distributes chunks based on AI weights
- **Concurrent Reception**: Receiver uses `tokio::select!` to accept data from all 3 lanes simultaneously
- **Per-Lane Statistics**: Real-time tracking of packets/bytes per lane

---


## 🚦 How to Run the Suite

> **Important:** For full GPU and network performance, always run TurboNet natively on Windows. WSL2 is not recommended for production use due to limited hardware passthrough and routing issues.


### 1. Prerequisites
-   **Hardware**: Windows PC with an NVIDIA GPU and multiple network interfaces.
-   **Software**: Rust (Cargo), CUDA Toolkit, and [Ollama](https://ollama.com/) (running `deepseek-r1:8b`).


### 2. Launch the Ghost Receiver
The receiver must be running first. It waits for a handshake from either the CLI Shredder or Mission Control.
```bashclea
cargo run --release --bin receiver
```
*Note: The receiver will print its listening IP. Use this IP in your .env or Mission Control.*


### 3. Option A: Run the CLI Shredder (Stress Test)
To stress test the system with a large payload:
1. Generate a 100MB dummy file:
   ```powershell
   fsutil file createnew payload.bin 104857600
   ```
2. Run the shredder:
   ```bash
   cargo run --release --bin shred
   ```
   *Press ENTER when prompted to initiate the Quantum Handshake.*


### 3. Option B: Launch Mission Control (GUI)
Open the visual dashboard. **Run this from Windows PowerShell or Command Prompt.**
```powershell
cd D:\TurboNet
cargo run --release --bin mission_control
```
1.  **Select Payload**: Click "📂 SELECT TARGET PAYLOAD" to pick your file.
2.  **Telemetry**: Observe the **Neural Radar** (gauges).
3.  **Blast**: Click "🚀 INITIATE QUANTUM BLAST".


---

## ⚡ Hardware/Environment Warning

> **For best results:**
> - Run all TurboNet binaries from Windows, not WSL.
> - Ensure NVIDIA drivers and CUDA Toolkit are installed on Windows.
> - Your 2.5Gbps Ethernet and GPU will only be fully utilized when running natively.

---

## 🛡️ Security Protocol (Quantum Mesh)
When you "Blast," the Kyber ciphertext (CT) is itself shredded across the three network ports (`8001`, `8002`, `8003`). An attacker would need to intercept **all three physical bands simultaneously** AND possess a **4000+ qubit Quantum Computer** to compromise the session.

## 📂 Project Structure

### Binaries
- `src/bin/mission_control.rs`: The v4.1 GUI Controller (Updated Protocol).
- `src/bin/receiver.rs`: The Ghost Receiver (Auto-sizing, Metadata aware).
- `src/bin/shred.rs`: CLI Shredder (Defaults to `payload.bin`).
- `src/bin/check_lanes.rs`: Lane detection utility (Ethernet/Starlink).
- `src/bin/flood.rs`: UDP flood speed test.
- `src/bin/scan.rs`: Hardware lane scanner.

### Core Source Files
- `src/lib.rs`: Links modules.
- `src/shredder.rs`: Orchestrates GPU kernel.
- `src/crypto.rs`: Quantum session handshake (Kyber, AES-GCM).
- `src/gui.rs`: Mission Control implementation.

### CUDA & PTX
- `shredder.cu`: CUDA kernel for asymmetric shredding.
- `shredder.ptx`: Pre-compiled PTX (allows running without nvcc).

---
**The Mission is now complete. The Ghost is in the Lattice.** 🫡⚛️🧠🏁🏆
