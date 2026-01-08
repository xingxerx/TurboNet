use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct DeepSeekWeights {
    pub w0: u64,
    pub w1: u64,
    pub w2: u64,
}

impl DeepSeekWeights {
    /// Sanitizes DeepSeek output by stripping <think> tags before parsing JSON.
    pub fn from_raw_response(raw: &str) -> Result<Self, Box<dyn Error>> {
        let cleaned = if let Some(end_tag) = raw.find("</think>") {
            &raw[end_tag + 8..] // Skip past the thinking process
        } else {
            raw
        };
        let weights: Self = serde_json::from_str(cleaned.trim())?;
        weights.validate()?;
        Ok(weights)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        let total = self.w0 + self.w1 + self.w2;
        if total != 100 {
            return Err("Weights must sum to 100");
        }
        // Level 11 Requirement: Enforce minimum 5% fragmentation per lane 
        // to maintain quantum-safe physical scattering.
        if self.w0 < 5 || self.w1 < 5 || self.w2 < 5 {
            return Err("Tactical Error: AI attempted to bypass multi-lane mesh.");
        }
        Ok(())
    }
}
