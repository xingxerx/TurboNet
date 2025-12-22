use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeepSeekWeights {
    pub w0: u64,
    pub w1: u64,
    pub w2: u64,
}

impl DeepSeekWeights {
    pub fn validate(&self) -> Result<(), &'static str> {
        let total = self.w0 + self.w1 + self.w2;
        if total != 100 {
            return Err("Weights must sum to 100");
        }
        if self.w0 == 0 || self.w1 == 0 || self.w2 == 0 {
            return Err("No weight can be zero");
        }
        Ok(())
    }
}
