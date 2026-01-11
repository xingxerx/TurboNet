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

// Added for Neural Link / GUI Integration
use reqwest::Client;
#[derive(serde::Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(serde::Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn query_deepseek_strategy(rtt_data: [f64; 3], active_threat: bool) -> Option<DeepSeekWeights> {
    let client = Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
    
    let prompt = format!(r#"
    <system>
    You are the 'Neural Strategist' for a quantum-safe data shredder. 
    Your goal: Maximize throughput by distributing data chunks across 3 network lanes.
    Calculate integer percentage weights (w0, w1, w2) summing to exactly 100.
    
    Lane Status (RTT in seconds):
    - Lane 0 (2.4GHz): {:.4}
    - Lane 1 (5GHz-1): {:.4}
    - Lane 2 (5GHz-2): {:.4}
    
    {}

    Output ONLY JSON: {{ "w0": 33, "w1": 33, "w2": 34 }}
    </system>
    "#, 
    rtt_data[0], rtt_data[1], rtt_data[2],
    if active_threat { "CRITICAL WARNING: Active Cyber Attack detected on Lane 0 (UDP). AVOID Lane 0 if possible!" } else { "" }
    );

    let req = OllamaRequest {
        model: &model,
        prompt: &prompt,
        stream: false,
    };

    match client.post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await 
    {
        Ok(resp) => {
            if let Ok(json_resp) = resp.json::<OllamaResponse>().await {
                match DeepSeekWeights::from_raw_response(&json_resp.response) {
                    Ok(weights) => return Some(weights),
                    Err(e) => eprintln!("AI Parse Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Ollama Connection Failed: {}", e),
    }
    None
}
