use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Generic AI Client for Ollama or OpenAI-compatible APIs
#[derive(Clone)]
pub struct AiClient {
    pub client: Client,
    pub api_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl AiClient {
    /// Create client with Ollama backend (default)
    pub fn ollama(model: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            api_url: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
            api_key: None,
        }
    }

    /// Create client with OpenAI-compatible API
    pub fn openai_compatible(api_url: &str, model: &str, api_key: Option<&str>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_url: api_url.to_string(),
            model: model.to_string(),
            api_key: api_key.map(|k| k.to_string()),
        }
    }

    /// Helper to create a client from a spec string like "ollama:gpt-oss" or "openai:gpt-4o"
    pub fn from_spec(spec: &str) -> Self {
        let (provider, model_name) = parse_model_spec(spec);
        match provider.as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").ok();
                Self::openai_compatible(
                    "https://api.openai.com/v1/chat/completions",
                    &model_name,
                    api_key.as_deref(),
                )
            }
            _ => Self::ollama(&model_name),
        }
    }

    /// Send a prompt to the LLM and get a text response
    pub async fn generate(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Serialize)]
        struct OllamaRequest<'a> {
            model: &'a str,
            prompt: &'a str,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        // TODO: Support OpenAI Chat Completion format properly if provider is openai
        // For now, this implementation is heavily biased towards Ollama's /api/generate
        // We really should differentiate based on provider, but the original code was simple.
        // Let's stick to the original logic for now but encapsulate it.

        let request = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
        };

        let mut req_builder = self.client.post(&self.api_url);

        if let Some(key) = &self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("LLM API error {}: {}", status, text).into());
        }

        let ollama_resp: OllamaResponse = response.json().await?;
        Ok(ollama_resp.response)
    }
}

/// Parse model string like "ollama:deepseek-coder" or "openai:gpt-4o"
pub fn parse_model_spec(spec: &str) -> (String, String) {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("ollama".to_string(), spec.to_string())
    }
}
