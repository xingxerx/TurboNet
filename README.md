# TurboNet: Quantum-Hardened Security Toolkit

TurboNet is an open-source, modular security research platform written in Rust. Designed for high-speed packet analysis, GPU-accelerated polymorphic generation, and post-quantum threat assessment.

## 🚀 Modules

| Module | Description |
|--------|-------------|
| **turbonet-core** | Core networking library, file transfer (`shred`/`receiver`) |
| **spectre** | GPU-accelerated polymorphic payload generator |
| **sentinel** | Memory forensics: memscan, hook detector, token stealing |
| **tools** | PE parser, strings extractor, port scanner, beacon generator |
| **wifi-recon** | WiFi auditing and quantum threat analysis |

## � Structure

```
TurboNet/
├── Cargo.toml          # Workspace manifest
├── turbonet-core/      # Core library + file transfer
├── spectre/            # GPU polymorphic engine
├── sentinel/           # Memory forensics tools
├── tools/              # Analysis utilities
├── wifi-recon/         # WiFi reconnaissance
└── py_src/             # Python quantum scripts
```

## 🔨 Building

```bash
# Build all modules
cargo build --release --workspace

# Run specific tools
cargo run -p spectre -- mutate --input payload.bin
cargo run -p sentinel --bin sentinel-memscan -- --list
cargo run -p tools --bin pe-parser -- notepad.exe
cargo run -p wifi-recon --bin quantum-hound -- hunt
```

## 🛡️ Available Tools

### Spectre (GPU Engine)
- `spectre mutate` - Polymorphic payload generation
- `spectre quantum` - Quantum threat analysis
- `spectre entropy` - File entropy calculation

### Sentinel (Memory Forensics)
- `sentinel-memscan` - RWX/MZ header detection
- `hook-detector` - Inline hook scanning
- `token-steal` - Access token enumeration
- `proc-hollow` - Process injection demo

### Tools (Analysis)
- `pe-parser` - PE file analysis
- `strings-extract` - String extraction
- `net-sniffer` - UDP listener + port scan
- `beacon-gen` - C2 beacon generator

### WiFi Recon
- `quantum-hound` - AI-driven WiFi auditing
- `wifi-scan` - Network interface detection

## ⚠️ Disclaimer

This toolkit is for educational and authorized security testing only. Use responsibly.
