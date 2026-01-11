# ⚠️ LEGAL WARNING

**TurboNet is a security research framework intended for authorized penetration testing and educational use only.**

-   **Authorization Required**: You must have explicit written permission from the network owner before using any offensive capabilities (e.g., "Quantum Blast", "Packet Shredding", "WiFi Deauth").
-   **Compliance**: Users are responsible for complying with all applicable local, state, and federal laws (e.g., CFAA in the US, GDPR in Europe).
-   **Liability**: The authors and contributors are not liable for any misuse or damage caused by this software.

---

# 🛰️ TurboNet: Post-Quantum AI Network Shredder (v4.0)

**TurboNet** is a state-of-the-art software suite designed for ultra-secure, AI-optimized data fragmentation. It bridges **GPU-accelerated shredding** with **Post-Quantum Cryptography** and an **AI Reasoning Engine** to create an un-interceptable data stream across multiple physical network bands.

---

## 🏗️ Architecture & Tech Stack

TurboNet is built on a **Dual-AI Architecture**, separating strategic optimization from tactical defense.

### 🧠 The Dual-Brain System
1.  **Strategic Engine (DeepSeek-R1)**:
    *   **Role**: Network Optimization & Shredding Logic.
    *   **Location**: `src/deepseek_weights.rs` & `mission_control`.
    *   **Function**: Analyzes real-time lane congestion (RTT/Packet Loss) to calculate optimal `w0, w1, w2` shredding weights.
    *   **Config**: Controlled via `OLLAMA_MODEL` env var.

2.  **Tactical Engine (GPT-OSS)**:
    *   **Role**: Unsupervised Traffic Analysis & Active Defense.
    *   **Location**: `turbonet-core/src/ai_defense.rs` & `tools/src/net_guard.rs`.
    *   **Function**: Analyzes captured UDP/TCP payloads for anomalies (SQLi, beacons) to enforce blocking rules.
    *   **Config**: Controlled via CLI args (`--model ollama:gpt-oss`).

### 🛠️ Core Technology
-   **System**: Rust (2021 Edition) + Tokio (Async Runtime).
-   **Compute**: CUDA 13.0 (NVIDIA GPU Acceleration).
-   **Security**: `pqc_kyber` (Post-Quantum Key Exchange) + AES-256-GCM.
-   **GUI**: `egui` (Immediate Mode, OpenGL backend).

---

## � Core Modules

### 1. Quantum Ghost (Level 9)
*Directory: `src/crypto.rs` / `src/bin/receiver.rs`*
Implements the **Harvest Now, Decrypt Later** resistance mechanism. Uses **ML-KEM (Kyber-768)** to derive a 256-bit AES session key. This handshake occurs *out-of-band* or pre-shared if configured, ensuring the key is never exposed to quantum listeners on the data lanes.

### 2. Neural Strategist (Level 8)
*Directory: `src/shredder.rs` / `src/bin/mission_control.rs`*
The "Planner". It connects to your local Ollama instance running **DeepSeek-R1**. It queries the model with network telemetry ("Lane 1: 50ms, Lane 2: 200ms") and applies the returned weights to the GPU Kernel.

### 3. Cyber Sentinel (Level 11)
*Directory: `tools/src/net_guard.rs` / `turbonet-core`*
The "Enforcer". An independent agent that sits on the edge network. It uses **GPT-OSS** (like `llama3` or `gpt-4o`) to classify raw hex streams. It implements a user-space firewall that silently drops packets from IP addresses flagged by the LLM.

---


## 🚦 How to Run the Suite

> **Important:** For full GPU and network performance, always run TurboNet natively on Windows. WSL2 is not recommended for production use due to limited hardware passthrough and routing issues.


### 1. Prerequisites
-   **Hardware**: Windows PC with an NVIDIA GPU and multiple network interfaces.
-   **Software**: Rust (Cargo), CUDA Toolkit, and [Ollama](https://ollama.com/) (running `deepseek-r1:8b`).

### 2. Install the TurboNet CLI (Recommended)
To run the `turbonet` command globally:
```bash
cargo install --path turbonet-core
```
*This allows you to run `turbonet guard` from any terminal.*


### 3. Launch the Ghost Receiver
The receiver must be running first to generate the lattice keypair and listen for fragments.
```bash
# You must provide the expected total size in bytes (logged by the sender)
./target/release/receiver.exe <TOTAL_BYTES>
```


### 4. Launch Mission Control (The Controller)
Open the GUI dashboard to manage the mission. **Run this from Windows PowerShell or Command Prompt, not WSL.**
```powershell
cd D:\TurboNet
cargo clean
cargo build --release --bin mission_control
$env:OLLAMA_MODEL="deepseek-r1:8b"
./target/release/mission_control.exe
```

---

### ⚠️ WSL2/X11 Troubleshooting (Advanced)
If you must run the GUI from WSL2, set your DISPLAY to the Windows host bridge IP:

```bash
grep nameserver /etc/resolv.conf | awk '{print $2}'
export DISPLAY=YOUR_NAMESERVER_IP:0.0
export LIBGL_ALWAYS_SOFTWARE=1
cargo run --release --bin mission_control --
```
> **Note:** WSL2 will not provide full GPU or network performance. CUDA and 2.5Gbps Ethernet are only available natively on Windows.

---

## 📝 All Command-Line Invocations

You can run the main binaries using Cargo as follows:

```bash
# Run the Ghost Receiver (replace <TOTAL_BYTES> with your value)
cargo run --release --bin receiver -- <TOTAL_BYTES>

# Run Mission Control (GUI Controller)
cargo run --release --bin mission_control --

# Run the Legacy Command-Line Shredder
cargo run --release --bin shred --

# Run the AI Traffic Guard (Active Defense)
turbonet guard start --port 8888 --model ollama:gpt-oss

# OR run from source:
cargo run -p turbonet-core --bin turbonet -- guard start --port 8888 --model ollama:gpt-oss
```

Or run the built executables directly:

```bash
./target/release/receiver.exe <TOTAL_BYTES>
./target/release/mission_control.exe
./target/release/shred.exe
```

1.  **Select Payload**: Click "📂 SELECT TARGET PAYLOAD" to pick your file.
2.  **Telemetry**: Observe the **Neural Radar** (gauges) as they probe your 2.4GHz/5GHz lanes.
3.  **Blast**: Click "🚀 INITIATE QUANTUM BLAST".
    -   Handshake: Derives session entropy via Kyber shards.
    -   AI Strategy: DeepSeek-R1 decides the lane distribution.
    -   Streaming: CUDA shards and encrypts data in real-time.

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
- `src/bin/mission_control.rs`: The v4.0 GUI Controller (AI, dashboard, async UDP).
- `src/bin/receiver.rs`: The Ghost Receiver (reassembles fragments, quantum handshake).
- `src/bin/turbonet.rs`: Unified CLI orchestrator for all security tools.
- `src/bin/shred.rs`: Legacy Command-Line Shredder (manual blast).
- `src/bin/Memory-Safe_Listener.rs`: UDP echo node for connection verification.
- `src/bin/broadcaster.rs`: Multi-band broadcaster (demo, placeholder).
- `src/bin/check_lanes.rs`: Lane detection utility (Ethernet/Starlink).
- `src/bin/flood.rs`: UDP flood speed test.
- `src/bin/scan.rs`: Hardware lane scanner.
- `src/bin/tokio.rs`: Tokio async demo.
- `tools/src/net_guard.rs`: AI active defense and traffic blocking.

### Core Source Files
- `src/lib.rs`: Links modules (deepseek_weights, network, shredder).
- `src/deepseek_weights.rs`: DeepSeek AI weights logic and validation.
- `src/network.rs`: Lane probing, hardware binding, network interface logic.
- `src/shredder.rs`: Orchestrates GPU kernel, applies AI weights.
- `src/crypto.rs`: Quantum session handshake (Kyber, AES-GCM).

### CUDA & PTX
- `shredder.cu`: CUDA kernel for asymmetric shredding.
- `shredder.ptx`: Compiled PTX for GPU execution.
- `build.rs`: Auto-compiles CUDA kernel to PTX.

### Other
- `Cargo.toml`: Rust dependencies and features.
- `README.md`: This documentation.

---
**Available binaries:**
Memory-Safe_Listener, broadcaster, check_lanes, flood, mission_control, receiver, scan, shred, tokio

---
**The Mission is now complete. The Ghost is in the Lattice.** 🫡⚛️🧠🏁🏆
