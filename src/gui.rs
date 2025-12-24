use eframe::egui;
use std::path::PathBuf;

enum GuiUpdate {
    Status(String),
    Rtts([f64; 3]),
    Weights((u64, u64, u64)),
    Progress { current: usize, total: usize },
    Error(String),
    Finished,
}

#[derive(Debug)]
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
    update_rx: Option<std::sync::mpsc::Receiver<GuiUpdate>>,
}

impl Clone for MissionControlGui {
    fn clone(&self) -> Self {
        Self {
            file_path: self.file_path.clone(),
            target_ip: self.target_ip.clone(),
            lane_rtts: self.lane_rtts.clone(),
            ai_status: self.ai_status.clone(),
            current_block: self.current_block,
            total_blocks: self.total_blocks,
            is_blasting: self.is_blasting,
            ai_weights: self.ai_weights.clone(),
            blast_error: self.blast_error.clone(),
            update_rx: None, // Receiver cannot be cloned
        }
    }
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
            blast_error: None,
            update_rx: None,
        }
    }
}

impl eframe::App for MissionControlGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for updates from background thread
        let mut finished = false;
        if let Some(rx) = &self.update_rx {
            while let Ok(update) = rx.try_recv() {
                match update {
                    GuiUpdate::Status(s) => self.ai_status = s,
                    GuiUpdate::Rtts(r) => self.lane_rtts = r,
                    GuiUpdate::Weights(w) => self.ai_weights = Some(w),
                    GuiUpdate::Progress { current, total } => {
                        self.current_block = current;
                        self.total_blocks = total;
                    }
                    GuiUpdate::Error(e) => {
                        self.blast_error = Some(e);
                        self.is_blasting = false;
                        finished = true;
                    }
                    GuiUpdate::Finished => {
                        self.is_blasting = false;
                        finished = true;
                    }
                }
            }
        }
        if finished {
            self.update_rx = None;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_height(80.0);

            // Hardware/Environment warning
            #[cfg(not(target_os = "windows"))]
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Not running natively on Windows. GPU and network performance may be degraded.");
            #[cfg(target_os = "windows")]
            {
                if cudarc::driver::CudaDevice::new(0).is_err() {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ CUDA device not found. Ensure NVIDIA drivers and CUDA Toolkit are installed.");
                }
            }

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
                    
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.update_rx = Some(rx);
                    let ctx_clone = ctx.clone();

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            let send = |update: GuiUpdate| {
                                let _ = tx.send(update);
                                ctx_clone.request_repaint();
                            };

                            // 1. Probe lanes
                            send(GuiUpdate::Status("Probing lanes...".to_string()));
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
                            send(GuiUpdate::Rtts(rtts));

                            // 2. AI weights
                            send(GuiUpdate::Status("Querying DeepSeek-R1 for weights...".to_string()));
                            let weights = crate::deepseek_weights::DeepSeekWeights { w0: 33, w1: 33, w2: 34 };
                            if let Err(e) = weights.validate() {
                                send(GuiUpdate::Error(e.to_string()));
                                return;
                            }
                            send(GuiUpdate::Weights((weights.w0, weights.w1, weights.w2)));

                            // 3. Handshake
                            send(GuiUpdate::Status("Performing quantum handshake...".to_string()));
                            let pk_bytes = vec![0u8; 32];
                            let (_ct, session) = match crate::crypto::QuantumSession::initiate(&pk_bytes) {
                                Ok(res) => res,
                                Err(e) => {
                                    send(GuiUpdate::Error(e.to_string()));
                                    return;
                                }
                            };

                            // 4. Read and Encrypt
                            send(GuiUpdate::Status("Encrypting payload...".to_string()));
                            let payload = match &file_path {
                                Some(path) => match std::fs::read(path) {
                                    Ok(data) => data,
                                    Err(e) => {
                                        send(GuiUpdate::Error(format!("Read error: {}", e)));
                                        return;
                                    }
                                },
                                None => {
                                    send(GuiUpdate::Error("No file selected".to_string()));
                                    return;
                                }
                            };
                            let encrypted = session.encrypt_payload(&payload);

                            // 5. CUDA
                            send(GuiUpdate::Status("Launching CUDA kernel...".to_string()));
                            let dev = match cudarc::driver::CudaDevice::new(0) {
                                Ok(d) => std::sync::Arc::new(d),
                                Err(e) => {
                                    send(GuiUpdate::Error(format!("CUDA error: {}", e)));
                                    return;
                                }
                            };
                            let weights_arr = [weights.w0, weights.w1, weights.w2];
                            let salt = 42u64;
                            let (frag24, frag5g1, frag5g2) = match crate::shredder::apply_ai_strategy(&dev, &encrypted, weights_arr, salt).await {
                                Ok(res) => res,
                                Err(e) => {
                                    send(GuiUpdate::Error(format!("Shredder error: {}", e)));
                                    return;
                                }
                            };

                            // 6. UDP Stream
                            send(GuiUpdate::Status("Streaming fragments...".to_string()));
                            send(GuiUpdate::Progress { current: 0, total: 3 });
                            let frags = [&frag24, &frag5g1, &frag5g2];
                            for (i, frag) in frags.iter().enumerate() {
                                let addr = format!("{}:{}", target_ip, ports[i]);
                                match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                                    Ok(socket) => {
                                        let _ = socket.send_to(frag, &addr).await;
                                        send(GuiUpdate::Progress { current: i + 1, total: 3 });
                                    }
                                    Err(e) => {
                                        send(GuiUpdate::Error(format!("UDP error: {}", e)));
                                        return;
                                    }
                                }
                            }
                            send(GuiUpdate::Status("Quantum Blast complete!".to_string()));
                            send(GuiUpdate::Finished);
                        });
                    });
                }
                ui.add_space(10.0);
                if ui.add_enabled(self.is_blasting, egui::Button::new(egui::RichText::new("🛑 STOP").size(20.0).color(egui::Color32::RED))).clicked() {
                    self.is_blasting = false;
                    self.update_rx = None;
                    self.ai_status = "Blast aborted.".to_string();
                }
                if self.is_blasting {
                    let prog = self.current_block as f32 / self.total_blocks.max(1) as f32;
                    ui.add(egui::ProgressBar::new(prog).text(format!("Block {} / {}", self.current_block, self.total_blocks)).animate(true));
                }
            });
            ctx.request_repaint();
        });
    }
}
