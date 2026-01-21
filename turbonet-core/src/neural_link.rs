use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// The shared memory bus between the Strategic and Tactical engines
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeuralBus {
    pub last_updated: u64,
    pub threat_level: ThreatLevel,
    pub active_threats: usize,
    pub impacted_lanes: Vec<String>,
    pub tactical_advice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatLevel {
    Safe,
    Elevated,
    Critical,
}

impl Default for NeuralBus {
    fn default() -> Self {
        Self {
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            threat_level: ThreatLevel::Safe,
            active_threats: 0,
            impacted_lanes: vec![],
            tactical_advice: None,
        }
    }
}

impl NeuralBus {
    fn bus_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or(PathBuf::from("."));
        path.push("neural_bus.json");
        path
    }

    /// Read the current state of the Neural Bus
    pub fn read() -> Self {
        let path = Self::bus_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(bus) = serde_json::from_str(&content) {
                    return bus;
                }
            }
        }
        Self::default()
    }

    /// Write an update to the Neural Bus (Tactical Engine -> Strategic Engine)
    pub fn update(threats: usize, lanes: Vec<String>, advice: Option<String>) {
        let level = if threats > 5 {
            ThreatLevel::Critical
        } else if threats > 0 {
            ThreatLevel::Elevated
        } else {
            ThreatLevel::Safe
        };

        let bus = Self {
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            threat_level: level,
            active_threats: threats,
            impacted_lanes: lanes,
            tactical_advice: advice,
        };

        let path = Self::bus_path();
        if let Ok(json) = serde_json::to_string_pretty(&bus) {
            let _ = fs::write(path, json);
        }
    }
}
