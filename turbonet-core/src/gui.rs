use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use std::time::Duration;
use pqc_kyber::*;
use std::convert::TryInto;
use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;

#[allow(dead_code)]
enum GuiUpdate {
    Status(String),
    Rtts([f64; 3]),
    Weights((u64, u64, u64)),
    Throughput(f64), // MB/s
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
    // v5.0 Settings
    chunk_size: usize,
    turbo_mode: bool,
    multilane_mode: bool,
    throughput: f64,
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
            chunk_size: self.chunk_size,
            turbo_mode: self.turbo_mode,
            multilane_mode: self.multilane_mode,
            throughput: self.throughput,
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
            // v5.0 Settings
            chunk_size: 1400,
            turbo_mode: false,
            multilane_mode: true,
            throughput: 0.0,
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
                    GuiUpdate::Throughput(t) => self.throughput = t,
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
            
            // v5.0 SETTINGS PANEL
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("⚙️ TRANSFER SETTINGS").strong());
                ui.horizontal(|ui| {
                    ui.label("Chunk Size:");
                    ui.add(egui::Slider::new(&mut self.chunk_size, 512..=60000).suffix(" bytes"));
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.turbo_mode, "🚀 Turbo Mode (no delay)");
                    ui.checkbox(&mut self.multilane_mode, "📡 Multi-Lane (3 sockets)");
                });
                if self.throughput > 0.0 {
                    ui.label(egui::RichText::new(format!("⚡ Live: {:.1} MB/s", self.throughput))
                        .color(egui::Color32::LIGHT_BLUE).strong());
                }
            });
            ui.add_space(15.0);
            
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

                            // Config
                            let p1_port = "8001";
                            let target_addr = format!("{}:{}", target_ip, p1_port);

                            // Setup Socket
                            let sock = Socket::new(Domain::IPV4, Type::DGRAM, None).unwrap();
                            sock.set_reuse_address(true).unwrap();
                            sock.set_recv_buffer_size(4 * 1024 * 1024).unwrap();
                            sock.set_send_buffer_size(4 * 1024 * 1024).unwrap();
                            sock.bind(&"0.0.0.0:0".parse::<SocketAddr>().unwrap().into()).unwrap();
                            let socket = Arc::new(UdpSocket::from_std(sock.into()).unwrap());

                            // Read File
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
                            let total_len = payload.len();
                            let block_size = 5242880; // 5MB
                            let total_blocks = (total_len + block_size - 1) / block_size;
                            send(GuiUpdate::Progress { current: 0, total: total_blocks });

                            // 1. Handshake
                            send(GuiUpdate::Status("Requesting Public Key...".to_string()));
                            if let Err(e) = socket.send_to(b"PK_REQ", &target_addr).await {
                                send(GuiUpdate::Error(format!("Send PK_REQ failed: {}", e))); 
                                return;
                            }
                            let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
                             match tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut pk_buf)).await {
                                Ok(Ok((n, _))) if n == KYBER_PUBLICKEYBYTES => {},
                                _ => {
                                    send(GuiUpdate::Error("Handshake Timed Out".to_string()));
                                    return;
                                }
                            };
                            
                            send(GuiUpdate::Status("Encapsulating Secret...".to_string()));
                            let mut rng = rand::thread_rng();
                            let (_ct, shared_secret) = match encapsulate(&pk_buf, &mut rng) {
                                Ok(res) => res,
                                Err(_) => {
                                    send(GuiUpdate::Error("Encapsulation failed".to_string()));
                                    return;
                                }
                            };
                            let quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());

                            // 2. Metadata
                            send(GuiUpdate::Status("Synchronizing Metadata...".to_string()));
                            let mut meta = vec![b'M'];
                            let fname = file_path.as_ref().unwrap().file_name().unwrap().to_str().unwrap();
                            meta.extend_from_slice(&(fname.len() as u32).to_be_bytes());
                            meta.extend_from_slice(fname.as_bytes());
                            meta.extend_from_slice(&(total_len as u64).to_be_bytes());

                            let mut meta_confirmed = false;
                            for _ in 0..10 { // Try 10 times
                                let _ = socket.send_to(&meta, &target_addr).await;
                                let mut ack_buf = [0u8; 64];
                                if let Ok(Ok((n, _))) = tokio::time::timeout(Duration::from_millis(500), socket.recv_from(&mut ack_buf)).await {
                                    let msg = String::from_utf8_lossy(&ack_buf[..n]);
                                    if msg.starts_with("META_ACK") {
                                        meta_confirmed = true;
                                        break;
                                    }
                                }
                            }
                            if !meta_confirmed {
                                send(GuiUpdate::Error("Metadata Handshake Failed".to_string()));
                                return;
                            }

                            // 3. Stream
                            send(GuiUpdate::Status("Streaming Payload...".to_string()));
                            let w0 = 33; let w1 = 33; let w2 = 34; // Static weights for now

                            for block_idx in 0..total_blocks {
                                let start = block_idx * block_size;
                                let end = (start + block_size).min(total_len);
                                let block_data = &payload[start..end];

                                let mut header = [0u8; 28];
                                header[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
                                header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
                                header[12..16].copy_from_slice(&(block_data.len() as u32).to_be_bytes());
                                header[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
                                header[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
                                header[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());

                                // Send Header
                                let _ = socket.send_to(&header, &target_addr).await;
                                tokio::task::yield_now().await;

                                // Send Chunks
                                let chunk_size = 1024;
                                for chunk in block_data.chunks(chunk_size) {
                                    let _ = socket.send_to(chunk, &target_addr).await;
                                    tokio::time::sleep(Duration::from_micros(500)).await;
                                }

                                send(GuiUpdate::Progress { current: block_idx + 1, total: total_blocks });
                            }

                            send(GuiUpdate::Status("Mission Success!".to_string()));
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
