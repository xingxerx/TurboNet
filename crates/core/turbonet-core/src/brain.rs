use crate::ai_client::AiClient;
use std::error::Error;
use std::path::PathBuf;

/// The Brain is the central orchestrator that intelligently decides which module to fire.
///
/// It strictly adheres to the principle of "only firing what is needed" by analyzing
/// the user's intent.
pub struct Brain {
    ai_client: Option<AiClient>,
}

#[derive(Debug, Clone)]
pub enum Intent {
    /// Need to scan a network or host
    Scan { target: String, ports: String },
    /// Need to analyze security findings or advise on defense
    Defend { input: Option<PathBuf> },
    /// Need to generate a world or simulation
    World { theme: String },
    /// Need to run physics simulation
    Simulate,
    /// Need to mutate a payload or analyze entropy
    Spectre { action: String },
    /// Need to analyze memory or processes
    Sentinel { target: String },
    /// Intent unclear
    Unknown(String),
}

impl Brain {
    /// Create a new Brain instance.
    ///
    /// If an AI model is provided, it can be used for advanced intent classification.
    /// Otherwise, we use heuristic keywords.
    pub fn new(model: Option<String>) -> Self {
        let ai_client = model.map(|m| AiClient::from_spec(&m));
        Self { ai_client }
    }

    /// Analyze the input query and decide ONCE what to do.
    pub async fn perceive(&self, query: &str) -> Result<Intent, Box<dyn Error>> {
        // 1. Fast Path: Regex / Heuristics (Zero-latency dispatch)

        let q_lower = query.to_lowercase();

        if q_lower.contains("defend") || q_lower.contains("analyze") || q_lower.contains("advice") {
            return Ok(Intent::Defend { input: None });
        }

        if q_lower.contains("scan") || q_lower.contains("nmap") || q_lower.contains("port") {
            // Extract target crudely for now (in real AGI this would be smarter)
            // Assuming format "scan <target>"
            let parts: Vec<&str> = query.split_whitespace().collect();
            let mut target = "localhost".to_string();
            for (i, part) in parts.iter().enumerate() {
                if (*part == "scan" || *part == "target") && i + 1 < parts.len() {
                    target = parts[i + 1].to_string();
                }
            }
            return Ok(Intent::Scan {
                target,
                ports: "1-1024".to_string(),
            });
        }

        if q_lower.contains("world") || q_lower.contains("generate") {
            return Ok(Intent::World {
                theme: query.to_string(),
            });
        }

        if q_lower.contains("entropy") || q_lower.contains("mutate") {
            return Ok(Intent::Spectre {
                action: "general".to_string(),
            });
        }

        if q_lower.contains("memory") || q_lower.contains("process") || q_lower.contains("hook") {
            return Ok(Intent::Sentinel {
                target: "all".to_string(),
            });
        }

        // 2. Slow Path: LLM Analysis (if enabled and fast path failed)
        if let Some(_client) = &self.ai_client {
            // TODO: Hook up actual LLM call here to classify complex intents
            // For now, fall back to unknown
        }

        Ok(Intent::Unknown(query.to_string()))
    }

    /// Execute the decided intent.
    ///
    /// This function returns a structured Command-like enum or description
    /// that the main CLI can use to dispatch.
    ///
    /// Ideally, this `Brain` module stays pure and returns instructions,
    /// but for pragmatism, we might print what we *would* do or return a config.
    pub fn process_intent(&self, intent: Intent) -> String {
        match intent {
            Intent::Scan { target, ports } => {
                format!("BRAIN: Activating Network Cortex only.\n-> Target: {}\n-> Action: Scan ports {}", target, ports)
            }
            Intent::Defend { .. } => {
                format!("BRAIN: Activating Defense Cortex.\n-> Loading findings...\n-> Consulting Tactical AI...")
            }
            Intent::World { theme } => {
                format!("BRAIN: Activating Creative Cortex.\n-> Spinning up World Generator\n-> Theme: {}", theme)
            }
            Intent::Simulate => {
                format!("BRAIN: Activating Physics Engine.\n-> Running simulation steps.")
            }
            Intent::Spectre { .. } => {
                format!("BRAIN: Activating Spectre GPU Core.\n-> High-performance logic required.")
            }
            Intent::Sentinel { .. } => {
                format!("BRAIN: Activating Sentinel Watchdog.\n-> Scanning memory space.")
            }
            Intent::Unknown(q) => {
                format!(
                    "BRAIN: Cortex confused. Query '{}' did not trigger specific lobes.",
                    q
                )
            }
        }
    }
}
