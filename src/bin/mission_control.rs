use serde::{Deserialize, Serialize};
use reqwest;
#[allow(dead_code)]
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[allow(dead_code)]
async fn get_ai_strategy(rtt_data: [f64; 3]) -> Option<AiWeights> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-r1:8b".to_string());
    let prompt = format!(
        "Return a JSON object with weights for Lane 0, 1, and 2 based on these RTTs: Lane 0: {:.2}ms, Lane 1: {:.2}ms, Lane 2: {:.2}ms. The weights must sum to 100. Lower RTT = higher weight. Example: {{\"w0\": 10, \"w1\": 45, \"w2\": 45}}.",
        rtt_data[0] * 1000.0, rtt_data[1] * 1000.0, rtt_data[2] * 1000.0
    );
    let req = OllamaRequest {
        model,
        prompt,
        stream: false,
        format: "json".to_string(),
    };
    if let Ok(resp) = client.post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await
    {
        if let Ok(json_resp) = resp.json::<OllamaResponse>().await {
            if let Ok(weights) = serde_json::from_str::<AiWeights>(&json_resp.response) {
                return Some(weights);
            }
        }
    }
    None
}
// Move this to before impl so it's in scope
async fn shred_logic_cancel(_tx: mpsc::Sender<MissionEvent>, _path: PathBuf, _ip: String, _cancel_flag: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement cancellation-aware transfer logic here.
    Ok(())
}
use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;


#[derive(Deserialize, Debug, Clone)]
struct AiWeights {
    w0: i32,
    w1: i32,
    w2: i32,
}

enum MissionEvent {
    Error,
}

struct MissionControlApp {
    file_path: Option<PathBuf>,
    target_ip: String,
    lane_rtts: [f64; 3],
    ai_status: String,
    current_block: usize,
    total_blocks: usize,
    is_blasting: bool,
    event_tx: mpsc::Sender<MissionEvent>,
    runtime: Arc<tokio::runtime::Runtime>,
    ai_weights: Option<AiWeights>,
    cancel_flag: Arc<AtomicBool>,
}

impl MissionControlApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, _rx) = mpsc::channel(100);
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            file_path: None,
            // SECURITY: Always use .env for IP, never fallback to hardcoded IP
            target_ip: std::env::var("TURBONET_TARGET_IP").expect("TURBONET_TARGET_IP must be set in .env for security"),
            lane_rtts: [0.0; 3],
            ai_status: "Awaiting Command...".to_string(),
            current_block: 0,
            total_blocks: 0,
            is_blasting: false,
            event_tx: tx,
            runtime: Arc::new(rt),
            ai_weights: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn run_shredder(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        let tx = self.event_tx.clone();
        let path = self.file_path.clone().unwrap();
        let ip = self.target_ip.clone();
        let cancel_flag = self.cancel_flag.clone();
        self.runtime.spawn(async move {
            if let Err(_e) = shred_logic_cancel(tx.clone(), path, ip, cancel_flag).await {
                let _ = tx.send(MissionEvent::Error).await;
            }
        });
    }
    fn stop_transfer(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

}

impl eframe::App for MissionControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_height(80.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("📦 THE HANGAR").strong());
                if ui.button(egui::RichText::new("📂 SELECT TARGET PAYLOAD").size(16.0)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.file_path = Some(path);
                    }
                }
                if let Some(path) = &self.file_path {
                    ui.colored_label(egui::Color32::LIGHT_BLUE, format!("PAYLOAD: {}", path.file_name().unwrap().to_str().unwrap_or("Data")));
                } else {
                    ui.label("NO DATA STAGED");
                }
            });
            ui.add_space(10.0);

            // 2. THE NEURAL RADAR (Physics visualization)
            ui.columns(3, |columns| {
                let labels = ["📡 2.4GHz", "⚡ 5GHz-1", "⚡ 5GHz-2"];
                for i in 0..3 {
                    columns[i].vertical_centered(|ui| {
                        ui.label(labels[i]);
                        let rtt_ms = self.lane_rtts[i] * 1000.0;
                        let color = if rtt_ms < 1.0 { egui::Color32::GREEN } else if rtt_ms < 10.0 { egui::Color32::YELLOW } else { egui::Color32::RED };
                        let progress = (1.0 - (rtt_ms / 100.0).min(1.0)) as f32;
                        ui.add(egui::ProgressBar::new(progress).text(format!("{:.2}ms", rtt_ms)).desired_width(120.0).fill(color));
                    });
                }
            });

            ui.add_space(15.0);

            // 3. THE AI LOG (Reasoning logs)
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label("🧠 NEURAL STRATEGIST LOG:");
                ui.label(egui::RichText::new(&self.ai_status).monospace().color(egui::Color32::LIGHT_GREEN));
                if let Some(w) = &self.ai_weights {
                    ui.label(format!("Lattice Decision: {}% | {}% | {}%", w.w0, w.w1, w.w2));
                }
            });

            ui.add_space(20.0);

            // 4. THE BLAST CONSOLE (Execution)
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("RECEIVER IP:");
                    ui.text_edit_singleline(&mut self.target_ip);
                });
                ui.add_space(15.0);

                let btn_text = if self.is_blasting { "🌊 STREAMING..." } else { "🚀 INITIATE QUANTUM BLAST" };
                let btn = ui.add_enabled(!self.is_blasting && self.file_path.is_some(), egui::Button::new(egui::RichText::new(btn_text).size(24.0).strong()));
                if btn.clicked() {
                    self.run_shredder();
                }

                // Always show the STOP button, but only enable it if blasting
                ui.add_space(10.0);
                let stop_btn = ui.add_enabled(self.is_blasting, egui::Button::new(egui::RichText::new("🛑 STOP").size(20.0).color(egui::Color32::RED)));
                if stop_btn.clicked() {
                    self.stop_transfer();
                }

                if self.is_blasting {
                    ui.add_space(15.0);
                    let prog = self.current_block as f32 / self.total_blocks as f32;
                    ui.add(egui::ProgressBar::new(prog).text(format!("Block {} / {}", self.current_block, self.total_blocks)).animate(true));
                }
            });

            // Always keep the GUI responsive
            ctx.request_repaint();
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 700.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "TurboNet Mission Control",
        options,
        Box::new(|cc| Ok(Box::new(MissionControlApp::new(cc)))),
    )
}
