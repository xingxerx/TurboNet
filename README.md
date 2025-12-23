# 🛰️ TurboNet: Post-Quantum AI Network Shredder (v4.0)

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

### 🖥️ Level 10: Mission Control Dashboard
A dedicated visual dashboard for selecting files, monitoring "Sonic Probes" (RTT gauges), and initiating the "Quantum Blast" with a single click.

---

## 🚦 How to Run the Suite

### 1. Prerequisites
-   **Hardware**: Windows PC with an NVIDIA GPU and multiple network interfaces.
-   **Software**: Rust (Cargo), CUDA Toolkit, and [Ollama](https://ollama.com/) (running `deepseek-r1:8b`).


### 2. Launch the Ghost Receiver
The receiver must be running first to generate the lattice keypair and listen for fragments.
```bash
# You must provide the expected total size in bytes (logged by the sender)
./target/release/receiver.exe <TOTAL_BYTES>
```

### 3. Launch Mission Control (The Controller)
Open the GUI dashboard to manage the mission.
```bash
./target/release/mission_control.exe
```

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

## 🛡️ Security Protocol (Quantum Mesh)
When you "Blast," the Kyber ciphertext (CT) is itself shredded across the three network ports (`8001`, `8002`, `8003`). An attacker would need to intercept **all three physical bands simultaneously** AND possess a **4000+ qubit Quantum Computer** to compromise the session.

## 📂 Project Structure

### Binaries
- `src/bin/mission_control.rs`: The v4.0 GUI Controller (AI, dashboard, async UDP).
- `src/bin/receiver.rs`: The Ghost Receiver (reassembles fragments, quantum handshake).
- `src/bin/shred.rs`: Legacy Command-Line Shredder (manual blast).
- `src/bin/Memory-Safe_Listener.rs`: UDP echo node for connection verification.
- `src/bin/broadcaster.rs`: Multi-band broadcaster (demo, placeholder).
- `src/bin/check_lanes.rs`: Lane detection utility (Ethernet/Starlink).
- `src/bin/flood.rs`: UDP flood speed test.
- `src/bin/scan.rs`: Hardware lane scanner.
- `src/bin/tokio.rs`: Tokio async demo.

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
