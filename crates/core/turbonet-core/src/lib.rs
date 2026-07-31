pub mod crypto; // Links src/crypto.rs
pub mod deepseek_weights; // Links src/deepseek_weights.rs
pub mod fragment; // Links src/fragment.rs - GPU multi-lane fragmentation
pub mod gui; // Links src/gui.rs
pub mod network; // Links src/network.rs

// SOTA Performance Modules
pub mod ai_client; // Shared AI Client logic
pub mod ai_defense; // Passive traffic detection + defense advisor
pub mod ai_weights; // Local AI inference (Phase 3)
pub mod io_backend; // Pluggable I/O backends (Phase 1)
pub mod neural_link; // Shared defense telemetry bus

#[cfg(feature = "fec")]
pub mod fec; // Forward Error Correction (Phase 2)
