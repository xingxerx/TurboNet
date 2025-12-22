use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::net::UdpSocket;
use std::fs;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use pqc_kyber::*;
use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize, Debug, Clone)]
struct AiWeights {
    w0: i32,
    w1: i32,
    w2: i32,
}

enum MissionEvent {
    HandshakeStarted,
    HandshakeComplete,
    ProbingLanes,
    LaneRTT(usize, f64),
    AiConsulting,
    AiDecision(AiWeights),
    BlastingBlock(usize, usize),
    MissionSuccess,
    Error(String),
}

struct MissionControlApp {
    file_path: Option<PathBuf>,
    target_ip: String,
    lane_rtts: [f64; 3],
    ai_status: String,
    current_block: usize,
    total_blocks: usize,
    is_blasting: bool,
    event_rx: mpsc::Receiver<MissionEvent>,
    event_tx: mpsc::Sender<MissionEvent>,
    runtime: Arc<tokio::runtime::Runtime>,
    ai_weights: Option<AiWeights>,
}

impl MissionControlApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        Self {
            file_path: None,
            target_ip: std::env::var("TURBONET_TARGET_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            lane_rtts: [0.0; 3],
            ai_status: "Awaiting Command...".to_string(),
            current_block: 0,
            total_blocks: 0,
            is_blasting: false,
            event_rx: rx,
            event_tx: tx,
            runtime: Arc::new(rt),
            ai_weights: None,
        }
    }

    fn run_shredder(&self) {
        let tx = self.event_tx.clone();
        let path = self.file_path.clone().unwrap();
        let ip = self.target_ip.clone();

        self.runtime.spawn(async move {
            if let Err(e) = shred_logic(tx.clone(), path, ip).await {
                let _ = tx.send(MissionEvent::Error(e.to_string())).await;
            }
        });
    }
}

// Helper for weighted distribution hits
fn get_hits(t: u64, w_total: u64, w_target: u64, offset: u64) -> u64 {
    if t == 0 { return 0; }
    let cycles = t / w_total;
    let rem = t % w_total;
    let mut hits = cycles * w_target;
    if rem > offset {
        hits += (rem - offset).min(w_target);
    }
    hits
}

fn get_lane_len_asymmetric(n: usize, salt: u64, w0: i32, w1: i32, w2: i32, lane: i32) -> usize {
    let w_total = (w0 + w1 + w2) as u64;
    let pattern_offset = salt % w_total;
    let t_start = pattern_offset;
    let t_end = pattern_offset + n as u64;
    
    let (w_target, offset) = match lane {
        0 => (w0 as u64, 0u64),
        1 => (w1 as u64, w0 as u64),
        2 => (w2 as u64, (w0 + w1) as u64),
        _ => unreachable!(),
    };

    let hits_end = get_hits(t_end, w_total, w_target, offset);
    let hits_start = get_hits(t_start, w_total, w_target, offset);
    (hits_end - hits_start) as usize
}

async fn blast_lane_gui(s: &UdpSocket, data: &[u8], port: u16, target_ip: &str, head: &[u8]) {
    let _ = s.send_to(head, format!("{}:{}", target_ip, port)).await;
    // Bandwidth Beast: No sleep after header, just a yield
    tokio::task::yield_now().await;
    for chunk in data.chunks(1400) { // Increased chunk size for MTU efficiency
        let _ = s.send_to(chunk, format!("{}:{}", target_ip, port)).await;
        // High-speed burst with zero delay
    }
}

async fn shred_logic(tx: mpsc::Sender<MissionEvent>, path: PathBuf, ip: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let dev = match CudaDevice::new(0) {
        Ok(d) => {
            let ptx_src = fs::read_to_string("shredder.cu")?;
            let ptx = compile_ptx(ptx_src)?;
            d.load_ptx(ptx, "shredder", &["shred_kernel"])?;
            Some(d)
        },
        Err(_) => {
            println!("⚠️ CUDA not available. Running in CPU-only mode.");
            None
        }
    };

    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let file_bytes = fs::read(&path)?;
    let total_len = file_bytes.len();
    let total_blocks = (total_len + block_size - 1) / block_size;

    // Level 9 Handshake
    let _ = tx.send(MissionEvent::HandshakeStarted).await;
    socket.send_to(b"PK_REQ", format!("{}:{}", ip, p1_port)).await?;
    let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
    let recv_res = tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv_from(&mut pk_buf)).await;
    
    let (n, _) = match recv_res {
        Ok(Ok(res)) => res,
        Ok(Err(e)) if e.raw_os_error() == Some(10054) => {
            return Err("GHOST RECEIVER NOT FOUND. Please launch receiver.exe first on the target host.".into());
        }
        Ok(Err(e)) => return Err(format!("Socket Error: {}", e).into()),
        Err(_) => return Err("Handshake Timeout: Receiver did not respond.".into()),
    };
    if n != KYBER_PUBLICKEYBYTES { return Err("Invalid PK size".into()); }
    
    let (ct, shared_secret, quantum_salt) = {
        let mut rng = rand::thread_rng();
        let (ct, ss) = encapsulate(&pk_buf, &mut rng).map_err(|e| format!("KEM Fail: {:?}", e))?;
        let salt = u64::from_be_bytes(ss[0..8].try_into().unwrap());
        (ct, ss, salt)
    };
    // Send 8-byte file size header to receiver (required for handshake)
    socket.send_to(&(total_len as u64).to_be_bytes(), format!("{}:{}", ip, p1_port)).await?;
    
    // Fragment CT across lanes
    let chunk_ct = (ct.len() + 2) / 3;
    let _ = socket.send_to(&ct[0..chunk_ct], format!("{}:{}", ip, p1_port)).await;
    let _ = socket.send_to(&ct[chunk_ct..chunk_ct*2], format!("{}:{}", ip, p2_port)).await;
    let _ = socket.send_to(&ct[chunk_ct*2..], format!("{}:{}", ip, p3_port)).await;
    
    let _ = tx.send(MissionEvent::HandshakeComplete).await;

    // Level 11 METADATA: Robust Handshake
    let mut meta = vec![b'M'];
    let fname = path.file_name().unwrap().to_str().unwrap().as_bytes();
    meta.extend_from_slice(&(fname.len() as u32).to_be_bytes());
    meta.extend_from_slice(fname);
    meta.extend_from_slice(&(total_len as u64).to_be_bytes());
    
    let mut meta_confirmed = false;
    while !meta_confirmed {
        let _ = tx.send(MissionEvent::HandshakeStarted).await; // Re-use for status
        socket.send_to(&meta, format!("{}:{}", ip, p1_port)).await?;
        
        let mut ack_buf = [0u8; 64];
        if let Ok(Ok((n, _))) = tokio::time::timeout(std::time::Duration::from_millis(300), socket.recv_from(&mut ack_buf)).await {
            let msg = String::from_utf8_lossy(&ack_buf[..n]);
            if msg.starts_with("META_ACK") {
                meta_confirmed = true;
            }
        }
    }

    // Probing & AI
    let _ = tx.send(MissionEvent::ProbingLanes).await;
    let mut rtts = [0f64; 3];
    let ports = [p1_port, p2_port, p3_port];
    for i in 0..3 {
        let start = std::time::Instant::now();
        socket.send_to(&[0xFF; 16], format!("{}:{}", ip, ports[i])).await?;
        let mut b = [0u8; 16];
        if tokio::time::timeout(std::time::Duration::from_millis(1000), socket.recv_from(&mut b)).await.is_ok() {
            rtts[i] = start.elapsed().as_secs_f64();
        } else { rtts[i] = 1.0; }
        let _ = tx.send(MissionEvent::LaneRTT(i, rtts[i])).await;
    }

    let _ = tx.send(MissionEvent::AiConsulting).await;
    let weights = match get_ai_strategy_gui(rtts).await {
        Some(w) => w,
        None => AiWeights { w0: 33, w1: 33, w2: 34 },
    };
    let _ = tx.send(MissionEvent::AiDecision(weights.clone())).await;

    // FULL BLASTING LOOP (CUDACore Integration)
    let (w0, w1, w2) = (weights.w0, weights.w1, weights.w2);
    for b_idx in 0..total_blocks {
        let _ = tx.send(MissionEvent::BlastingBlock(b_idx + 1, total_blocks)).await;
        
        let start = b_idx * block_size;
        let end = (start + block_size).min(total_len);
        let block_data = &file_bytes[start..end];

        let key = Key::<Aes256Gcm>::from_slice(&shared_secret);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&(b_idx as u32).to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, block_data).expect("ENC FAIL");
        let enc_n = ciphertext.len();

        let dev_ref = dev.as_ref().expect("CUDA device not available");
        let d_in = dev_ref.htod_copy(ciphertext)?;
        let mut d_24 = dev_ref.alloc_zeros::<u8>(enc_n + 1024)?;
        let mut d_5g1 = dev_ref.alloc_zeros::<u8>(enc_n + 1024)?;
        let mut d_5g2 = dev_ref.alloc_zeros::<u8>(enc_n + 1024)?;

        let w_total = (w0 + w1 + w2) as u64;
        let p_off = quantum_salt % w_total;
        let i0 = get_hits(p_off, w_total, w0 as u64, 0);
        let i1 = get_hits(p_off, w_total, w1 as u64, w0 as u64);
        let i2 = get_hits(p_off, w_total, w2 as u64, (w0 + w1) as u64);

        let cfg = LaunchConfig::for_num_elems(enc_n as u32);
        let f = dev_ref.get_func("shredder", "shred_kernel").unwrap();
        unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, enc_n as u64, quantum_salt, w0 as u64, w1 as u64, w2 as u64, i0, i1, i2)) }?;

        let host_24 = dev_ref.dtoh_sync_copy(&d_24)?;
        let host_5g1 = dev_ref.dtoh_sync_copy(&d_5g1)?;
        let host_5g2 = dev_ref.dtoh_sync_copy(&d_5g2)?;

        let l0 = get_lane_len_asymmetric(enc_n, quantum_salt, w0, w1, w2, 0);
        let l1 = get_lane_len_asymmetric(enc_n, quantum_salt, w0, w1, w2, 1);
        let l2 = get_lane_len_asymmetric(enc_n, quantum_salt, w0, w1, w2, 2);

        let mut h = [0u8; 28];
        h[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
        h[8..12].copy_from_slice(&(b_idx as u32).to_be_bytes());
        h[12..16].copy_from_slice(&(enc_n as u32).to_be_bytes());
        h[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
        h[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
        h[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());

        let mut confirmed = false;
        while !confirmed {
            let _ = tokio::join!(
                blast_lane_gui(&socket, &host_24[0..l0], p1_port, &ip, &h),
                blast_lane_gui(&socket, &host_5g1[0..l1], p2_port, &ip, &h),
                blast_lane_gui(&socket, &host_5g2[0..l2], p3_port, &ip, &h),
            );

            // Level 12 Sequencer: Wait for ACK
            let mut res_buf = [0u8; 32];
            match tokio::time::timeout(std::time::Duration::from_millis(2000), socket.recv_from(&mut res_buf)).await {
                Ok(Ok((len, _))) => {
                    let msg = String::from_utf8_lossy(&res_buf[..len]);
                    if msg == format!("ACK:{}", b_idx) {
                        confirmed = true;
                    } else if msg == format!("NACK:{}", b_idx) {
                        println!("🔄 NACK RECEIVED: Retransmitting Block {}", b_idx);
                    }
                }
                _ => {
                    println!("⌛ ACK TIMEOUT: Retransmitting Block {}", b_idx);
                }
            }
        }
    }

    let _ = tx.send(MissionEvent::MissionSuccess).await;
    Ok(())
}

async fn get_ai_strategy_gui(rtts: [f64; 3]) -> Option<AiWeights> {
    let client = reqwest::Client::new();
    let prompt = format!("JSON weights for RTTs: Lane0: {:.2}ms, Lane1: {:.2}ms, Lane2: {:.2}ms. Sum 100.", rtts[0]*1000.0, rtts[1]*1000.0, rtts[2]*1000.0);
    let req = OllamaRequest { model: "deepseek-r1:8b".to_string(), prompt, stream: false, format: "json".to_string() };
    if let Ok(Ok(resp)) = tokio::time::timeout(std::time::Duration::from_secs(30), client.post("http://localhost:11434/api/generate").json(&req).send()).await {
        if let Ok(jr) = resp.json::<OllamaResponse>().await {
            return serde_json::from_str::<AiWeights>(&jr.response).ok();
        }
    }
    None
}

impl eframe::App for MissionControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                MissionEvent::HandshakeStarted => self.ai_status = "Initiating Quantum Handshake...".to_string(),
                MissionEvent::HandshakeComplete => self.ai_status = "Lattice Secured. Negotiating Lanes...".to_string(),
                MissionEvent::ProbingLanes => self.ai_status = "Sonic Probing Physical Layer...".to_string(),
                MissionEvent::LaneRTT(i, rtt) => self.lane_rtts[i] = rtt,
                MissionEvent::AiConsulting => self.ai_status = "Consulting DeepSeek R1...".to_string(),
                MissionEvent::AiDecision(w) => { 
                    self.ai_status = format!("AI STRATEGY: {} / {} / {}", w.w0, w.w1, w.w2);
                    self.ai_weights = Some(w);
                },
                MissionEvent::BlastingBlock(curr, total) => { self.current_block = curr; self.total_blocks = total; self.is_blasting = true; },
                MissionEvent::MissionSuccess => { self.is_blasting = false; self.ai_status = "MISSION SUCCESS: Stream Complete.".to_string(); },
                MissionEvent::Error(e) => { self.ai_status = format!("ERROR: {}", e); self.is_blasting = false; }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("🔥 TURBONET MISSION CONTROL").size(32.0).strong().color(egui::Color32::from_rgb(255, 100, 0)));
                ui.separator();
            });

            ui.add_space(20.0);

            // 1. THE HANGAR (Data Selection)
            ui.group(|ui| {
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

                if self.is_blasting {
                    ui.add_space(15.0);
                    let prog = self.current_block as f32 / self.total_blocks as f32;
                    ui.add(egui::ProgressBar::new(prog).text(format!("Block {} / {}", self.current_block, self.total_blocks)).animate(true));
                }
            });
        });

        if self.is_blasting {
            ctx.request_repaint();
        }
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
