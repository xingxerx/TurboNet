// TurboNet Local AI Weights Predictor
// Provides microsecond-scale lane weight prediction using local inference

#[cfg(feature = "onnx")]
use std::error::Error;

/// Lane weights for traffic distribution
#[derive(Debug, Clone, Copy)]
pub struct LaneWeights {
    pub w0: u64,
    pub w1: u64,
    pub w2: u64,
}

impl LaneWeights {
    /// Convert to array format
    pub fn as_array(&self) -> [u64; 3] {
        [self.w0, self.w1, self.w2]
    }
    
    /// Validate weights sum to 100 and meet minimum thresholds
    pub fn validate(&self) -> Result<(), &'static str> {
        let total = self.w0 + self.w1 + self.w2;
        if total != 100 {
            return Err("Weights must sum to 100");
        }
        if self.w0 < 5 || self.w1 < 5 || self.w2 < 5 {
            return Err("Each lane must have at least 5% allocation");
        }
        Ok(())
    }
}

/// Heuristic-based weight predictor (no ML required)
/// Uses inverse-RTT weighting with smoothing
pub struct HeuristicPredictor {
    alpha: f64, // Smoothing factor (0.0-1.0)
    prev_weights: Option<LaneWeights>,
}

impl HeuristicPredictor {
    pub fn new() -> Self {
        Self {
            alpha: 0.3, // 30% weight to new observation
            prev_weights: None,
        }
    }
    
    /// Predict optimal weights based on RTT measurements
    /// Lower RTT = higher weight allocation
    pub fn predict(&mut self, rtt_ms: [f64; 3], _loss_pct: [f64; 3]) -> LaneWeights {
        // Inverse RTT scoring (avoid division by zero)
        let scores: Vec<f64> = rtt_ms.iter()
            .map(|r| 1.0 / r.max(0.001))
            .collect();
        let sum: f64 = scores.iter().sum();
        
        // Raw weights from scores
        let raw_w0 = ((scores[0] / sum) * 100.0).round() as u64;
        let raw_w1 = ((scores[1] / sum) * 100.0).round() as u64;
        let raw_w2 = 100 - raw_w0 - raw_w1;
        
        // Apply minimum threshold (5% per lane)
        let (w0, w1, w2) = enforce_minimums(raw_w0, raw_w1, raw_w2);
        
        let new_weights = LaneWeights { w0, w1, w2 };
        
        // Apply exponential smoothing if we have previous weights
        let smoothed = if let Some(prev) = self.prev_weights {
            LaneWeights {
                w0: smooth(prev.w0, new_weights.w0, self.alpha),
                w1: smooth(prev.w1, new_weights.w1, self.alpha),
                w2: smooth(prev.w2, new_weights.w2, self.alpha),
            }
        } else {
            new_weights
        };
        
        // Normalize to ensure sum is exactly 100
        let normalized = normalize_weights(smoothed);
        self.prev_weights = Some(normalized);
        normalized
    }
}

impl Default for HeuristicPredictor {
    fn default() -> Self {
        Self::new()
    }
}

/// ONNX-based neural predictor (requires 'onnx' feature)
#[cfg(feature = "onnx")]
pub struct OnnxPredictor {
    session: ort::Session,
}

#[cfg(feature = "onnx")]
impl OnnxPredictor {
    /// Load model from file
    pub fn from_file(model_path: &str) -> Result<Self, Box<dyn Error>> {
        let session = ort::Session::builder()?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
            .commit_from_file(model_path)?;
        Ok(Self { session })
    }
    
    /// Load model from embedded bytes
    pub fn from_bytes(model_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let session = ort::Session::builder()?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
            .commit_from_memory(model_bytes)?;
        Ok(Self { session })
    }
    
    /// Predict weights with ~50µs latency
    pub fn predict(&self, rtt_ms: [f32; 3], loss_pct: [f32; 3]) -> Result<LaneWeights, Box<dyn Error>> {
        use ndarray::Array2;
        
        // Prepare input: [batch=1, features=6]
        let input_data = vec![
            rtt_ms[0], rtt_ms[1], rtt_ms[2],
            loss_pct[0], loss_pct[1], loss_pct[2],
        ];
        let input = Array2::from_shape_vec((1, 6), input_data)?;
        
        let outputs = self.session.run(ort::inputs!["input" => input.view()]?)?;
        let output = outputs["output"].try_extract_tensor::<f32>()?;
        
        // Output: [w0, w1, w2] (already normalized by model)
        let w0 = (output[[0, 0]] * 100.0).round() as u64;
        let w1 = (output[[0, 1]] * 100.0).round() as u64;
        let w2 = 100 - w0 - w1;
        
        let (w0, w1, w2) = enforce_minimums(w0, w1, w2);
        Ok(LaneWeights { w0, w1, w2 })
    }
}

/// Unified predictor that tries ONNX first, falls back to heuristic
pub struct AdaptivePredictor {
    #[cfg(feature = "onnx")]
    onnx: Option<OnnxPredictor>,
    heuristic: HeuristicPredictor,
}

impl AdaptivePredictor {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "onnx")]
            onnx: None,
            heuristic: HeuristicPredictor::new(),
        }
    }
    
    /// Try to load ONNX model
    #[cfg(feature = "onnx")]
    pub fn with_onnx_model(mut self, model_path: &str) -> Self {
        match OnnxPredictor::from_file(model_path) {
            Ok(predictor) => {
                self.onnx = Some(predictor);
                println!("🧠 ONNX model loaded: {}", model_path);
            }
            Err(e) => {
                eprintln!("⚠️ ONNX load failed, using heuristic: {}", e);
            }
        }
        self
    }
    
    /// Predict using best available method
    pub fn predict(&mut self, rtt_ms: [f64; 3], loss_pct: [f64; 3]) -> LaneWeights {
        #[cfg(feature = "onnx")]
        if let Some(ref onnx) = self.onnx {
            let rtt_f32 = [rtt_ms[0] as f32, rtt_ms[1] as f32, rtt_ms[2] as f32];
            let loss_f32 = [loss_pct[0] as f32, loss_pct[1] as f32, loss_pct[2] as f32];
            if let Ok(weights) = onnx.predict(rtt_f32, loss_f32) {
                return weights;
            }
        }
        
        // Fallback to heuristic
        self.heuristic.predict(rtt_ms, loss_pct)
    }
}

impl Default for AdaptivePredictor {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn enforce_minimums(w0: u64, w1: u64, w2: u64) -> (u64, u64, u64) {
    const MIN: u64 = 5;
    let mut weights = [w0.max(MIN), w1.max(MIN), w2.max(MIN)];
    
    // Normalize if over 100
    let sum: u64 = weights.iter().sum();
    if sum > 100 {
        let excess = sum - 100;
        // Take from the largest weight
        if let Some(max_idx) = weights.iter().enumerate().max_by_key(|(_, &v)| v).map(|(i, _)| i) {
            weights[max_idx] = weights[max_idx].saturating_sub(excess);
        }
    }
    
    (weights[0], weights[1], weights[2])
}

fn smooth(prev: u64, new: u64, alpha: f64) -> u64 {
    let result = (alpha * new as f64 + (1.0 - alpha) * prev as f64).round();
    result as u64
}

fn normalize_weights(weights: LaneWeights) -> LaneWeights {
    let sum = weights.w0 + weights.w1 + weights.w2;
    if sum == 100 {
        return weights;
    }
    
    let diff = 100i64 - sum as i64;
    // Adjust the largest weight
    let mut arr = [weights.w0, weights.w1, weights.w2];
    if let Some(max_idx) = arr.iter().enumerate().max_by_key(|(_, &v)| v).map(|(i, _)| i) {
        arr[max_idx] = (arr[max_idx] as i64 + diff) as u64;
    }
    
    LaneWeights {
        w0: arr[0],
        w1: arr[1],
        w2: arr[2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_equal_rtt() {
        let mut pred = HeuristicPredictor::new();
        let weights = pred.predict([1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        assert_eq!(weights.w0 + weights.w1 + weights.w2, 100);
    }

    #[test]
    fn test_heuristic_fast_lane() {
        let mut pred = HeuristicPredictor::new();
        // Lane 0 is 10x faster
        let weights = pred.predict([0.1, 1.0, 1.0], [0.0, 0.0, 0.0]);
        assert!(weights.w0 > weights.w1);
        assert!(weights.w0 > weights.w2);
    }

    #[test]
    fn test_minimum_enforcement() {
        let (w0, w1, w2) = enforce_minimums(95, 3, 2);
        assert!(w0 >= 5 && w1 >= 5 && w2 >= 5);
    }
}
