# 🛰️ TurboNet: Post-Quantum AI Multipath Transport (v0.2)

**TurboNet** is a software suite for ultra-fast, ultra-secure data transport. It
combines **GPU-accelerated data fragmentation** with **Post-Quantum
Cryptography** and an **AI reasoning engine** to move data across multiple
physical network lanes with an encrypted, quantum-resistant handshake.

This repository is the defensible transport core. Offensive intrusion/evasion
tooling that previously lived here has been removed; the remaining components are
the multipath transport, its cryptography and CUDA core, passive analysis
utilities, and a **passive** AI defense advisor.

---

## 🏗️ Architecture & Tech Stack

TurboNet separates strategic transport optimization from passive traffic
analysis.

### 🧠 AI Assistance
1.  **Strategic Engine (DeepSeek-R1)**:
    *   **Role**: Network optimization & fragmentation weighting.
    *   **Location**: `src/deepseek_weights.rs`.
    *   **Function**: Analyzes real-time lane congestion (RTT / packet loss) to
        calculate optimal `w0, w1, w2` fragmentation weights.
    *   **Config**: Controlled via the `OLLAMA_MODEL` env var.

2.  **Passive Analyst (GPT-OSS)**:
    *   **Role**: Detection-only traffic analysis and defense recommendations.
    *   **Location**: `turbonet-core/src/ai_defense.rs` & `tools/src/net_guard.rs`.
    *   **Function**: Classifies captured UDP payloads as benign / suspicious /
        malicious and logs the result. **It never blocks or drops traffic.**
    *   **Config**: Controlled via CLI args (`--model ollama:gpt-oss`).

### 🛠️ Core Technology
-   **System**: Rust (2021 Edition) + Tokio (async runtime).
-   **Compute**: CUDA (NVIDIA GPU acceleration, optional — see below).
-   **Security**: `pqc_kyber` (post-quantum key exchange) + AES-256-GCM.
-   **GUI**: `egui` (immediate mode).

---

## 🧩 Core Modules

### 1. Post-Quantum Handshake
*Files: `src/crypto.rs` / `src/bin/receiver.rs`*
Provides **Harvest-Now-Decrypt-Later** resistance using **ML-KEM (Kyber-768)** to
derive a 256-bit AES session key, so the key is never exposed to quantum
listeners on the data lanes.

### 2. Multipath Fragmentation
*Files: `src/fragment.rs` / `fragment.cu`*
Splits a payload across multiple UDP lanes using AI-derived weights and a
GPU kernel (`fragment_kernel`), reassembled on the receiver.

### 3. Passive Traffic Monitor
*Files: `tools/src/net_guard.rs` / `turbonet-core/src/ai_defense.rs`*
An independent agent that observes traffic and uses an LLM to classify raw
streams. It **logs** suspicious and malicious source IPs for review and updates a
telemetry bus. It performs **no blocking, dropping, or filtering**.

---

## 🚦 Building & Running

### 1. Prerequisites
-   **Software**: Rust (Cargo). Optional: CUDA Toolkit for GPU acceleration and
    [Ollama](https://ollama.com/) for the AI features.

### 2. Build

```bash
cargo build --release
```

**CUDA gating:** the GPU kernel is compiled by `build.rs` via `nvcc`. If the
pre-compiled `crates/core/turbonet-core/fragment.ptx` is present, the build skips
`nvcc` entirely. To force-skip the CUDA compilation step, set
`TURBONET_NO_CUDA=1`. The workspace builds without a CUDA toolkit installed.

### 3. Run the receiver
The receiver generates the lattice keypair and listens for fragments.
```bash
cargo run -p turbonet-core --bin receiver -- <TOTAL_BYTES>
```

### 4. Run the sender (fragmenter)
```bash
cargo run -p turbonet-core --bin fragment --
```

### 5. Run the passive traffic monitor
```bash
cargo run -p turbonet-core --bin turbonet -- guard start --port 8888 --model ollama:gpt-oss
```

---

## 📂 Project Structure

### 🧱 Core (`crates/core`)
-   `turbonet-core`: the heart of the system — the fragmenter (GPU kernels),
    crypto (Kyber/AES), the passive AI defense advisor, and the GUI.

### 📡 WiFi (`crates/wifi`)
-   `wifi-recon`: passive interface / network scanning (`wifi-scan`).

### 🛠️ Utilities (`crates/utils`)
-   `tools`: general-purpose networking and analysis tools (PE parser, string
    extraction, UDP sniffer, passive net-guard monitor).

---

## 📝 Command-Line Invocations

```bash
# Receiver
cargo run -p turbonet-core --bin receiver -- <TOTAL_BYTES>

# Sender / fragmenter
cargo run -p turbonet-core --bin fragment --

# Passive AI traffic monitor
cargo run -p turbonet-core --bin turbonet -- guard start --port 8888 --model ollama:gpt-oss

# AI defense advisor (analyze scan findings, get hardening recommendations)
cargo run -p turbonet-core --bin turbonet -- defend --demo
```
