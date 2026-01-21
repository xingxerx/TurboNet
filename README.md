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

## 🏢 Government & Compliance (Modules)
TurboNet includes modules specifically designed for authorized government and defense sectors, adhering to strict compliance and auditing standards.

### 🛡️ Cyber Sentinel (`crates/cybersecurity/sentinel`)
-   **Security Clearance**: Level 11
-   **Function**: Autonomous Active Defense & Traffic Filtering.
-   **Compliance**: Implements standard blocking rules and logs all neutralizing actions for audit trails.
-   **Use Case**: perimeter defense for sensitive infrastructure.

### 📜 Operations (`turbonet_ops`)
-   **Standard**: ISO/IEC 27001 & NIST 800-53 compatiable workflows.
-   **Auditing**: Automated Snyk security scans and Criterion performance benchmarking.

---

## 📂 Project Structure

The codebase is organized into modular crates for security isolation and maintainability:

### 🧱 Core (`crates/core`)
-   `turbonet-core`: The heart of the system. Contains the `Brain` (DeepSeek-R1 logic), `Shredder` (GPU Kernels), `Crypto` (Kyber/AES), and the main `Mission Control` GUI.

### 📡 WiFi (`crates/wifi`)
-   `wifi-recon`: Quantum-ready WiFi scanning and analysis tools (`quantum_hound`).

### ⚔️ Pentesting (`crates/pentesting`)
-   `spectre`: Advanced polymorphic payload generator and quantum threat analyzer.

### 🛡️ Cybersecurity (`crates/cybersecurity`)
-   `sentinel`: AI-driven active defense and packet inspection algorithms.

### 🛠️ Utilities (`crates/utils`)
-   `tools`: General purpose networking and diagnostic tools.

---

## 📝 All Command-Line Invocations

You can run the main binaries using Cargo as follows (paths updated for new structure):

```bash
# Run the Ghost Receiver
cargo run -p turbonet-core --bin receiver -- <TOTAL_BYTES>

# Run Mission Control (GUI Controller)
cargo run -p turbonet-core --bin mission_control --

# Run the Legacy Command-Line Shredder
cargo run -p turbonet-core --bin shred --

# Run the AI Traffic Guard
cargo run -p turbonet-core --bin turbonet -- guard start --port 8888 --model ollama:gpt-oss
```

---
**The Mission is now complete. The Ghost is in the Lattice.** 🫡⚛️🧠🏁🏆
