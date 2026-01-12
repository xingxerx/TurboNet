use crate::ai_client::AiClient;
use crate::spectre::{SpectreEngine, MutationMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub location: String,
    pub description: String,
    pub inventory: Vec<String>,
    pub turn_count: u32,
    pub history: Vec<String>,
    pub physics_override: Option<String>, // e.g., "Low Gravity", "High Radiation"
}

pub struct WorldGenerator {
    client: AiClient,
    state: WorldState,
    theme: String,
    chaos_engine: Option<SpectreEngine>,
}

impl WorldGenerator {
    pub fn new(client: AiClient, theme: &str, chaos_engine: Option<SpectreEngine>) -> Self {
        Self {
            client,
            state: WorldState {
                location: "Unknown".to_string(),
                description: "Initializing world...".to_string(),
                inventory: vec![],
                turn_count: 0,
                history: vec![],
                physics_override: None,
            },
            theme: theme.to_string(),
            chaos_engine,
        }
    }

    pub async fn initialize(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let chaos_prompt = self.inject_chaos().await?;
        
        let prompt = format!(
            "You are a hyper-realistic simulation engine. Theme: '{}'.\n\
            Generate the FIRST scene introduction.\n\
            \n\
            PHYSICS PARAMETERS:\n\
            - Gravity: 9.8 m/s^2 (Standard)\n\
            - Atmosphere: Standard Pressure\n\
            - Material Interactions: Realistic\n\
            {}\n\
            \n\
            Describe the setting, atmosphere, and what the player sees.\n\
            Focus on sensory details (sight, sound, smell) and physical properties of objects.\n\
            Keep it immersive but concise (under 100 words).\n\
            Do NOT ask the player what to do.",
            self.theme,
            chaos_prompt
        );

        let response = self.client.generate(&prompt).await?;
        self.state.description = response.trim().to_string();
        self.state.location = "Start".to_string();
        self.state.history.push(format!("start: {}", self.state.description));
        
        Ok(self.state.description.clone())
    }

    pub async fn next_turn(&mut self, action: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.state.turn_count += 1;
        
        // Keep context reasonable
        let recent_history = self.state.history.iter().rev().take(3).rev().cloned().collect::<Vec<_>>().join("\n");
        let chaos_prompt = self.inject_chaos().await?;
        
        let physics_context = self.state.physics_override.as_deref().unwrap_or("Standard Earth Physics");

        let prompt = format!(
            "You are a hyper-realistic physics and logic engine. Theme: '{}'.\n\
            \n\
            Current Environmental State:\n\
            - Physics Mode: {}\n\
            {}\n\
            \n\
            Recent History:\n\
            {}\n\
            \n\
            Player Action: {}\n\
            \n\
            Calculate the OUTCOME of this action based on physical laws (mass, velocity, friction, material strength).\n\
            1. Analyze the action's feasibility.\n\
            2. Determine immediate physical consequences.\n\
            3. Describe the new scene.\n\
            \n\
            If the action is physically impossible, explain why in the narrative.\n\
            Keep it immersive (under 100 words).\n\
            Do NOT output any JSON or code, just the story text.",
            self.theme,
            physics_context,
            chaos_prompt,
            recent_history,
            action
        );

        let response = self.client.generate(&prompt).await?;
        let result = response.trim().to_string();
        
        self.state.description = result.clone();
        self.state.history.push(format!("> {}\n{}", action, result));
        
        Ok(result)
    }

    pub fn get_state(&self) -> &WorldState {
        &self.state
    }

    /// Uses the GPU (if available) to generate "Chaos Entropy" from the current world description.
    /// This entropy introduces non-deterministic environmental fluctuations.
    async fn inject_chaos(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(engine) = &self.chaos_engine {
            // Use current description as seed
            let payload = self.state.description.as_bytes();
            // Generate variants
            let mutation = engine.generate_polymorphic(
                payload, 
                16, // variants
                self.state.turn_count as u64, // salt
                MutationMode::Xor
            ).await?;
            
            let entropy_level = mutation.entropy; // 0.0 to 8.0
            
            // Interpret entropy as "Environmental Instability"
            if entropy_level > 6.0 {
                Ok(format!("- WARNING: HIGH ENTROPY DETECTED ({:.2}). The environment is unstable. Strange quantum fluctuations or weather anomalies are occurring.", entropy_level))
            } else if entropy_level > 4.0 {
                Ok(format!("- NOTICE: Elevated Entropy ({:.2}). Minor physical anomalies present.", entropy_level))
            } else {
                Ok(format!("- STATUS: Entropy Stable ({:.2}). Physics are nominal.", entropy_level))
            }
        } else {
            Ok(String::new())
        }
    }
}
