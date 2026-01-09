pub mod deepseek_weights; // Links src/deepseek_weights.rs
pub mod network;  // Links src/network.rs
pub mod shredder; // Links src/shredder.rs
pub mod crypto; // Links src/crypto.rs
pub mod spectre; // Links src/spectre.rs - SPECTRE-GPU polymorphic engine
pub mod gui; // Links src/gui.rs

// SOTA Performance Modules
pub mod io_backend;  // Pluggable I/O backends (Phase 1)
pub mod ai_weights;  // Local AI inference (Phase 3)
pub mod ai_defense;  // AI-powered defense advisor

#[cfg(feature = "fec")]
pub mod fec;  // Forward Error Correction (Phase 2)