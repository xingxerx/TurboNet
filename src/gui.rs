use eframe::egui;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MissionControlGui {
    file_path: Option<PathBuf>,
    target_ip: String,
    lane_rtts: [f64; 3],
    ai_status: String,
    current_block: usize,
    total_blocks: usize,
    is_blasting: bool,
    ai_weights: Option<(u64, u64, u64)>,
    blast_error: Option<String>,
}

impl Default for MissionControlGui {
    fn default() -> Self {
        Self {
            file_path: None,
            target_ip: std::env::var("TURBONET_TARGET_IP").unwrap_or_default(),
            lane_rtts: [0.0; 3],
            ai_status: "Awaiting Command...".to_string(),
            current_block: 0,
            total_blocks: 0,
            is_blasting: false,
            ai_weights: None,
        }
    }
}

impl eframe::App for MissionControlGui {
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
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label("🧠 NEURAL STRATEGIST LOG:");
                ui.label(egui::RichText::new(&self.ai_status).monospace().color(egui::Color32::LIGHT_GREEN));
                if let Some((w0, w1, w2)) = &self.ai_weights {
                    ui.label(format!("Lattice Decision: {}% | {}% | {}%", w0, w1, w2));
                }
                if let Some(err) = &self.blast_error {
                    ui.label(egui::RichText::new(format!("❌ ERROR: {}", err)).color(egui::Color32::RED));
                }
            });
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("RECEIVER IP:");
                    ui.text_edit_singleline(&mut self.target_ip);
                });
                ui.add_space(15.0);
                let btn_text = if self.is_blasting { "🌊 STREAMING..." } else { "🚀 INITIATE QUANTUM BLAST" };
                let btn = ui.add_enabled(!self.is_blasting && self.file_path.is_some(), egui::Button::new(egui::RichText::new(btn_text).size(24.0).strong()));
                if btn.clicked() {
                    self.is_blasting = true;
                    self.blast_error = None;
                    self.ai_status = "Initializing Quantum Blast...".to_string();
                    let file_path = self.file_path.clone();
                    let target_ip = self.target_ip.clone();
                    let ai_status = &mut self.ai_status;
                    let lane_rtts = &mut self.lane_rtts;
                    let ai_weights = &mut self.ai_weights;
                    let current_block = &mut self.current_block;
                    let total_blocks = &mut self.total_blocks;
                    let blast_error = &mut self.blast_error;
                    // Spawn async backend logic (Tokio)
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            // 1. Probe lanes
                            let mut rtts = [0.0; 3];
                            let ports = ["8001", "8002", "8003"];
                            for (i, port) in ports.iter().enumerate() {
                                let addr = format!("{}:{}", target_ip, port);
                                match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                                    Ok(socket) => {
                                        let telemetry = crate::network::probe_lane(&socket, &addr).await;
                                        rtts[i] = telemetry.rtt.as_secs_f64();
                                    }
                                    Err(_) => {
                                        rtts[i] = 999.0;
                                    }
                                }
                            }
                            *lane_rtts = rtts;
                            *ai_status = "Probed lanes. Querying DeepSeek-R1 for weights...".to_string();
                            // 2. Get AI weights (stub: use default)
                            let weights = crate::deepseek_weights::DeepSeekWeights { w0: 33, w1: 33, w2: 34 };
                            if let Err(e) = weights.validate() {
                                *blast_error = Some(e.to_string());
                                return;
                            }
                            *ai_weights = Some((weights.w0, weights.w1, weights.w2));
                            *ai_status = "AI weights acquired. Performing quantum handshake...".to_string();
                            // 3. Quantum handshake (stub: use dummy key)
                            let pk_bytes = vec![0u8; 32];
                            let (ct, session) = match crate::crypto::QuantumSession::initiate(&pk_bytes) {
                                Ok(res) => res,
                                Err(e) => {
                                    *blast_error = Some(e.to_string());
                                    return;
                                }
                            };
                            *ai_status = "Handshake complete. Encrypting payload...".to_string();
                            // 4. Read file and encrypt
                            let payload = match &file_path {
                                Some(path) => match std::fs::read(path) {
                                    Ok(data) => data,
                                    Err(e) => {
                                        *blast_error = Some(format!("Failed to read file: {}", e));
                                        return;
                                    }
                                },
                                None => {
                                    *blast_error = Some("No file selected".to_string());
                                    return;
                                }
                            };
                            let encrypted = session.encrypt_payload(&payload);
                            *ai_status = "Payload encrypted. Launching CUDA kernel...".to_string();
                            // 5. CUDA shredding
                            let dev = match cudarc::driver::CudaDevice::new(0) {
                                Ok(d) => std::sync::Arc::new(d),
                                Err(e) => {
                                    *blast_error = Some(format!("CUDA error: {}", e));
                                    return;
                                }
                            };
                            let weights_arr = [weights.w0, weights.w1, weights.w2];
                            let salt = 42u64;
                            let (frag24, frag5g1, frag5g2) = match crate::shredder::apply_ai_strategy(&dev, &encrypted, weights_arr, salt).await {
                                Ok(res) => res,
                                Err(e) => {
                                    *blast_error = Some(format!("Shredder error: {}", e));
                                    return;
                                }
                            };
                            *ai_status = "Shredding complete. Streaming fragments...".to_string();
                            *total_blocks = 3;
                            *current_block = 0;
                            // 6. UDP streaming
                            let frags = [&frag24, &frag5g1, &frag5g2];
                            for (i, frag) in frags.iter().enumerate() {
                                let port = ports[i];
                                let addr = format!("{}:{}", target_ip, port);
                                match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                                    Ok(socket) => {
                                        let _ = socket.send_to(frag, &addr).await;
                                        *current_block += 1;
                                    }
                                    Err(e) => {
                                        *blast_error = Some(format!("UDP error: {}", e));
                                        return;
                                    }
                                }
                            }
                            *ai_status = "Quantum Blast complete!".to_string();
                        });
                    });
                }
                ui.add_space(10.0);
                let _stop_btn = ui.add_enabled(self.is_blasting, egui::Button::new(egui::RichText::new("🛑 STOP").size(20.0).color(egui::Color32::RED)));
                if self.is_blasting {
                    let prog = self.current_block as f32 / self.total_blocks.max(1) as f32;
                    ui.add(egui::ProgressBar::new(prog).text(format!("Block {} / {}", self.current_block, self.total_blocks)).animate(true));
                }
            });
            ctx.request_repaint();
        });
    }
}
