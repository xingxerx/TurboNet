pub mod crypto; // Links src/crypto.rs
pub mod deepseek_weights; // Links src/deepseek_weights.rs
pub mod gui;
pub mod network; // Links src/network.rs
pub mod shredder; // Links src/shredder.rs
pub mod spectre; // Links src/spectre.rs - SPECTRE-GPU polymorphic engine // Links src/gui.rs

// SOTA Performance Modules
pub mod ai_client; // Shared AI Client logic
pub mod ai_defense;
pub mod ai_weights; // Local AI inference (Phase 3)
pub mod brain;
pub mod io_backend; // Pluggable I/O backends (Phase 1)
pub mod neural_link; // AI-powered defense advisor
pub mod physics_world;
pub mod world_gen; // Infinite procedural world generator // The new Brain Orchestrator

#[cfg(feature = "fec")]
pub mod fec; // Forward Error Correction (Phase 2)
