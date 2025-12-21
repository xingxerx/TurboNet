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
-   `src/bin/mission_control.rs`: The v4.0 GUI Controller.
-   `src/bin/receiver.rs`: The Ghost Receiver.
-   `src/bin/shred.rs`: Legacy Command-Line Shredder.
-   `shredder.cu`: The mathematical "heart"—the CUDA kernel for asymmetric shredding.

---
**The Mission is now complete. The Ghost is in the Lattice.** 🫡⚛️🧠🏁🏆
