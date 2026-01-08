//! SPECTRE-GPU: Rust bindings for polymorphic payload mutation
//!
//! Provides safe Rust interface to the SPECTRE CUDA kernel for
//! GPU-accelerated payload obfuscation.

use cudarc::driver::{CudaDevice, CudaSlice, LaunchAsync, LaunchConfig};
use std::sync::Arc;

/// Mutation mode for payload obfuscation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MutationMode {
    /// XOR with thread-unique key derivation
    Xor = 0,
    /// Bit rotation (ROL/ROR)
    Rotate = 1,
    /// S-box byte substitution
    Substitute = 2,
    /// Cascaded: XOR + ROL + XOR
    Cascade = 3,
}

/// Result of polymorphic mutation
#[derive(Debug)]
pub struct MutationResult {
    /// The mutated payload bytes
    pub payload: Vec<u8>,
    /// Shannon entropy score (0.0 - 8.0)
    pub entropy: f32,
    /// Index of the variant (for decoding)
    pub variant_index: u32,
    /// Mutation mode used
    pub mode: MutationMode,
}

/// SPECTRE-GPU engine for polymorphic payload generation
pub struct SpectreEngine {
    device: Arc<CudaDevice>,
}

impl SpectreEngine {
    /// Initialize the SPECTRE-GPU engine
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let device = CudaDevice::new(0)?;
        
        // Load the SPECTRE PTX module
        let ptx = std::fs::read_to_string("spectre.ptx")
            .map_err(|_| "spectre.ptx not found - run cargo build first")?;
        device.load_ptx(ptx.into(), "spectre", &[
            "spectre_mutate_kernel",
            "spectre_find_best_kernel",
            "spectre_decode_kernel",
        ])?;
        
        Ok(Self { device })
    }
    
    /// Initialize from existing device (reuse TurboNet's CUDA context)
    pub fn from_device(device: Arc<CudaDevice>) -> Result<Self, Box<dyn std::error::Error>> {
        // Load the SPECTRE PTX module
        let ptx = std::fs::read_to_string("spectre.ptx")
            .map_err(|_| "spectre.ptx not found - run cargo build first")?;
        device.load_ptx(ptx.into(), "spectre", &[
            "spectre_mutate_kernel",
            "spectre_find_best_kernel",
            "spectre_decode_kernel",
        ])?;
        
        Ok(Self { device })
    }
    
    /// Generate polymorphic payload variants on GPU
    ///
    /// # Arguments
    /// * `payload` - Original payload bytes
    /// * `num_variants` - Number of variants to generate (higher = better entropy selection)
    /// * `salt` - Session entropy (use Kyber shared secret for deterministic decode)
    /// * `mode` - Mutation algorithm to use
    ///
    /// # Returns
    /// The highest-entropy variant result
    pub async fn generate_polymorphic(
        &self,
        payload: &[u8],
        num_variants: u32,
        salt: u64,
        mode: MutationMode,
    ) -> Result<MutationResult, Box<dyn std::error::Error>> {
        let len = payload.len();
        let num_variants = num_variants as usize;
        
        // Copy input to GPU
        let d_input = self.device.htod_copy(payload.to_vec())?;
        
        // Allocate output buffer for all variants
        let mut d_output: CudaSlice<u8> = self.device.alloc_zeros(len * num_variants)?;
        
        // Allocate entropy scores
        let mut d_entropies: CudaSlice<f32> = self.device.alloc_zeros(num_variants)?;
        
        // Launch mutation kernel
        let cfg = LaunchConfig::for_num_elems(num_variants as u32);
        let mutate_func = self.device
            .get_func("spectre", "spectre_mutate_kernel")
            .expect("spectre_mutate_kernel not found");
        
        unsafe {
            mutate_func.launch(cfg, (
                &d_input,
                &mut d_output,
                &mut d_entropies,
                len as i32,
                salt,
                num_variants as i32,
                mode as i32,
            ))?;
        }
        
        // Copy entropies back to find best variant
        let entropies = self.device.dtoh_sync_copy(&d_entropies)?;
        
        // Find highest entropy variant (CPU-side for simplicity)
        let (best_idx, best_entropy) = entropies
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, &e)| (i, e))
            .unwrap_or((0, 0.0));
        
        // Copy the best variant's output
        let all_outputs = self.device.dtoh_sync_copy(&d_output)?;
        let start = best_idx * len;
        let end = start + len;
        let best_payload = all_outputs[start..end].to_vec();
        
        Ok(MutationResult {
            payload: best_payload,
            entropy: best_entropy,
            variant_index: best_idx as u32,
            mode,
        })
    }
    
    /// CPU fallback for entropy calculation (when CUDA unavailable)
    pub fn calculate_entropy_cpu(data: &[u8]) -> f32 {
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }
        
        let len = data.len() as f32;
        let mut entropy = 0.0f32;
        
        for &count in &counts {
            if count > 0 {
                let p = count as f32 / len;
                entropy -= p * p.log2();
            }
        }
        
        entropy
    }
    
    /// Decode a mutated payload back to original
    ///
    /// Only works for XOR and CASCADE modes (reversible)
    pub async fn decode(
        &self,
        mutated: &[u8],
        salt: u64,
        variant_index: u32,
        mode: MutationMode,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if mode != MutationMode::Xor && mode != MutationMode::Cascade {
            return Err("Only XOR and CASCADE modes are reversible".into());
        }
        
        let len = mutated.len();
        
        // Copy input to GPU
        let d_input = self.device.htod_copy(mutated.to_vec())?;
        let mut d_output: CudaSlice<u8> = self.device.alloc_zeros(len)?;
        
        // Launch decode kernel
        let cfg = LaunchConfig::for_num_elems(len as u32);
        let decode_func = self.device
            .get_func("spectre", "spectre_decode_kernel")
            .expect("spectre_decode_kernel not found");
        
        unsafe {
            decode_func.launch(cfg, (
                &d_input,
                &mut d_output,
                len as i32,
                salt,
                variant_index as i32,
                mode as i32,
            ))?;
        }
        
        Ok(self.device.dtoh_sync_copy(&d_output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entropy_calculation() {
        // Random-looking data should have high entropy
        let random_ish: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let entropy = SpectreEngine::calculate_entropy_cpu(&random_ish);
        assert!(entropy > 7.9, "Expected high entropy for uniform distribution");
        
        // Repeated data should have low entropy
        let repeated = vec![0xAA; 256];
        let entropy = SpectreEngine::calculate_entropy_cpu(&repeated);
        assert!(entropy < 0.1, "Expected low entropy for repeated bytes");
    }
}
