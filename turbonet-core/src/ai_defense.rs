//! AI-Powered Defense Advisor
//! 
//! Analyzes pentest scan results and provides hardening recommendations
//! using local Ollama or OpenAI-compatible APIs.


use serde::{Deserialize, Serialize};


/// Scan findings from various TurboNet tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFindings {
    pub tool: String,
    pub target: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub description: String,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Defense recommendations from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseReport {
    pub summary: String,
    pub recommendations: Vec<Recommendation>,
    pub firewall_rules: Vec<String>,
    pub patches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: u8,
    pub title: String,
    pub description: String,
    pub implementation: String,
}

use crate::ai_client::AiClient;

/// AI Defense Advisor using local Ollama or OpenAI-compatible API
pub struct DefenseAdvisor {
    client: AiClient,
}

impl DefenseAdvisor {
    /// Create advisor with Ollama backend (default)
    pub fn ollama(model: &str) -> Self {
        Self {
            client: AiClient::ollama(model),
        }
    }

    /// Create advisor with OpenAI-compatible API
    pub fn openai_compatible(api_url: &str, model: &str, api_key: Option<&str>) -> Self {
        Self {
            client: AiClient::openai_compatible(api_url, model, api_key),
        }
    }

    /// Analyze scan findings and return defense recommendations
    pub async fn suggest_defenses(&self, findings: &ScanFindings) -> Result<DefenseReport, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = self.build_prompt(findings);
        
        let response = self.client.generate(&prompt).await?;
        
        // Parse AI response into structured report
        self.parse_response(&response, findings)
    }

    fn build_prompt(&self, findings: &ScanFindings) -> String {
        let findings_json = serde_json::to_string_pretty(&findings.findings).unwrap_or_default();
        
        format!(r#"You are a cybersecurity defense expert. Analyze these penetration test findings and provide defensive recommendations.

## Scan Context
- Tool: {}
- Target: {}

## Findings
```json
{}
```

## Required Response Format
Provide your response in this exact JSON structure:
```json
{{
  "summary": "Brief overview of the security posture",
  "recommendations": [
    {{
      "priority": 1,
      "title": "Recommendation title",
      "description": "What the issue is",
      "implementation": "Exact steps to remediate"
    }}
  ],
  "firewall_rules": ["iptables or Windows Firewall rule examples"],
  "patches": ["Specific patches or updates to apply"]
}}
```

Focus on actionable, specific mitigations. Prioritize by severity."#,
            findings.tool,
            findings.target,
            findings_json
        )
    }

    // Removed local call_llm in favor of self.client.generate()

    fn parse_response(&self, response: &str, findings: &ScanFindings) -> Result<DefenseReport, Box<dyn std::error::Error + Send + Sync>> {
        // Try to extract JSON from response
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            if let Ok(report) = serde_json::from_str::<DefenseReport>(json_str) {
                return Ok(report);
            }
        }

        // Fallback: generate basic report from raw response
        Ok(DefenseReport {
            summary: format!("Analysis of {} findings from {}", findings.findings.len(), findings.tool),
            recommendations: vec![Recommendation {
                priority: 1,
                title: "AI Analysis".to_string(),
                description: response.chars().take(500).collect(),
                implementation: "Review the full AI response for detailed guidance.".to_string(),
            }],
            firewall_rules: vec![],
            patches: vec![],
        })
    }
}

/// Parse model string like "ollama:deepseek-coder" or "openai:gpt-4o"
pub fn parse_model_spec(spec: &str) -> (String, String) {
    crate::ai_client::parse_model_spec(spec)
}

/// Network packet metadata for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPacket {
    pub timestamp: u64,
    pub src_ip: String,
    pub dst_port: u16,
    pub protocol: String,
    pub payload_preview: String,
    pub payload_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionType {
    Allow,
    Block,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDecision {
    pub ip: String,
    pub decision: DecisionType,
    pub confidence: u8,
    pub reason: String,
}

impl DefenseAdvisor {
    /// Analyze a batch of traffic packets and return access control decisions
    pub async fn analyze_traffic_batch(&self, packets: &[TrafficPacket]) -> Result<Vec<TrafficDecision>, Box<dyn std::error::Error + Send + Sync>> {
        if packets.is_empty() {
            return Ok(vec![]);
        }

        let prompt = self.build_traffic_prompt(packets);
        let response = self.client.generate(&prompt).await?;
        self.parse_traffic_response(&response)
    }

    fn build_traffic_prompt(&self, packets: &[TrafficPacket]) -> String {
        let packets_json = serde_json::to_string_pretty(packets).unwrap_or_default();
        
        format!(r#"You are an automated network security analyst (Traffic Guard). Analyze this batch of network packets for malicious activity.

## Traffic Batch
```json
{}
```

## Instructions
1. Analyze the source IP behavior, ports, and payload contents.
2. Look for: Port scanning, C2 beacons, SQL injection, buffer overflows, or unauthorized access attempts.
3. Decide: ALLOW, BLOCK, or MONITOR (flag for review) for each unique Source IP.
4. "payload_preview" is ASCII/Hex representation.

## Required Response Format
Return a JSON array of decisions. Do NOT explain outside JSON.
```json
[
  {{
    "ip": "1.2.3.4",
    "decision": "BLOCK",
    "confidence": 90,
    "reason": "Repeated connection attempts to diverse ports (Scanning)"
  }}
]
```"#, packets_json)
    }

    fn parse_traffic_response(&self, response: &str) -> Result<Vec<TrafficDecision>, Box<dyn std::error::Error + Send + Sync>> {
        // Try to extract JSON from response
        let json_start = response.find('[');
        let json_end = response.rfind(']');
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            if let Ok(decisions) = serde_json::from_str::<Vec<TrafficDecision>>(json_str) {
                return Ok(decisions);
            }
        }

        // Failure fallback - safe fail (allow all but log error)
        // In a real system we might block-all on failure if paranoid
        eprintln!("Failed to parse AI Traffic decision. Raw response: {}", response);
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_spec() {
        let (provider, model) = parse_model_spec("ollama:deepseek-coder");
        assert_eq!(provider, "ollama");
        assert_eq!(model, "deepseek-coder");

        let (provider, model) = parse_model_spec("llama3");
        assert_eq!(provider, "ollama");
        assert_eq!(model, "llama3");
    }

    #[test]
    fn test_severity_serialization() {
        let finding = Finding {
            severity: Severity::Critical,
            category: "RCE".to_string(),
            description: "Remote code execution".to_string(),
            evidence: Some("CVE-2024-1234".to_string()),
        };
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("\"severity\":\"critical\""));
    }
}
