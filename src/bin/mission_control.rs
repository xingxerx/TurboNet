use dotenvy;
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

use cudarc::driver::{CudaDevice, LaunchAsync, LaunchConfig};
use tokio::net::UdpSocket;
use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use pqc_kyber::*;
use rand::thread_rng;
use std::fs;

async fn gui_shred_logic(
    _tx: mpsc::Sender<MissionEvent>,
    path: PathBuf,
    ip: String,
    cancel_flag: Arc<AtomicBool>,
    update_progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
    let p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
    let p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();
    let w0_env: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    let w1_env: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let w2_env: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();

    let ptx_src = std::fs::read_to_string("shredder.cu")?;
    let ptx = cudarc::nvrtc::compile_ptx(ptx_src)?;
    let dev = CudaDevice::new(0)?;
    dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    sock.set_reuse_address(true)?;
    sock.set_recv_buffer_size(4 * 1024 * 1024)?;
    sock.set_send_buffer_size(4 * 1024 * 1024)?;
    sock.bind(&"0.0.0.0:0".parse::<SocketAddr>()?.into())?;
    let socket = std::sync::Arc::new(UdpSocket::from_std(sock.into())?);

    let file_bytes = fs::read(&path)?;
    let total_len = file_bytes.len();
    let total_blocks = (total_len + block_size - 1) / block_size;

    // Handshake
    socket.send_to(b"PK_REQ", format!("{}:{}", ip, p1_port)).await?;
    let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
    let (n, _) = tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv_from(&mut pk_buf)).await??;
    if n != KYBER_PUBLICKEYBYTES { return Err("Invalid PK size".into()); }
    let mut rng = thread_rng();
    let (_ct, shared_secret) = encapsulate(&pk_buf, &mut rng).map_err(|_| "Encapsulation failed")?;
    let quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());

    // Metadata
    let mut meta = vec![b'M'];
    let fname = path.file_name().unwrap().to_str().unwrap_or("payload");
    meta.extend_from_slice(&(fname.len() as u32).to_be_bytes());
    meta.extend_from_slice(fname.as_bytes());
    meta.extend_from_slice(&(total_len as u64).to_be_bytes());
    let mut meta_confirmed = false;
    while !meta_confirmed {
        socket.send_to(&meta, format!("{}:{}", ip, p1_port)).await?;
        let mut ack_buf = [0u8; 64];
        if let Ok(Ok((n, _))) = tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv_from(&mut ack_buf)).await {
            let msg = String::from_utf8_lossy(&ack_buf[..n]);
            if msg.starts_with("META_ACK") {
                meta_confirmed = true;
            }
        }
    }

    // Weights
    let (w0, w1, w2) = (w0_env, w1_env, w2_env);

    for block_idx in 0..total_blocks {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }
        let start = block_idx * block_size;
        let end = (start + block_size).min(total_len);
        let block_data = &file_bytes[start..end];

        let key = Key::<Aes256Gcm>::from_slice(&shared_secret);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&(block_idx as u32).to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, block_data).expect("Block encryption failed");
        let encrypted_n = ciphertext.len();

        let d_in = dev.htod_copy(ciphertext.clone())?;
        let lane_size = encrypted_n + 1024;
        let mut d_24 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g1 = dev.alloc_zeros::<u8>(lane_size)?;
        let mut d_5g2 = dev.alloc_zeros::<u8>(lane_size)?;

        let w_total = (w0 + w1 + w2) as u64;
        let _pattern_offset = quantum_salt % w_total;
        let i0 = 0u64;
        let i1 = 0u64;
        let i2 = 0u64;
        let cfg = LaunchConfig::for_num_elems(encrypted_n as u32);
        let f = dev.get_func("shredder", "shred_kernel").expect("Func not found");
        unsafe { f.launch(cfg, (&d_in, &mut d_24, &mut d_5g1, &mut d_5g2, encrypted_n as u64, quantum_salt, w0 as u64, w1 as u64, w2 as u64, i0, i1, i2)) }?;
        let host_24 = dev.dtoh_sync_copy(&d_24)?;
        let host_5g1 = dev.dtoh_sync_copy(&d_5g1)?;
        let host_5g2 = dev.dtoh_sync_copy(&d_5g2)?;

        let len0 = host_24.len();
        let len1 = host_5g1.len();
        let len2 = host_5g2.len();

        let mut header = [0u8; 28];
        header[0..8].copy_from_slice(&quantum_salt.to_be_bytes());
        header[8..12].copy_from_slice(&(block_idx as u32).to_be_bytes());
        header[12..16].copy_from_slice(&(encrypted_n as u32).to_be_bytes());
        header[16..20].copy_from_slice(&(w0 as u32).to_be_bytes());
        header[20..24].copy_from_slice(&(w1 as u32).to_be_bytes());
        header[24..28].copy_from_slice(&(w2 as u32).to_be_bytes());

        async fn blast_lane(socket: std::sync::Arc<UdpSocket>, data: Vec<u8>, port: u16, head: [u8; 28], ip: String) {
            let _ = socket.send_to(&head, format!("{}:{}", ip, port)).await;
            tokio::task::yield_now().await;
            let chunk_size = 1024;
            for chunk in data.chunks(chunk_size) {
                let _ = socket.send_to(chunk, format!("{}:{}", ip, port)).await;
                tokio::time::sleep(std::time::Duration::from_micros(500)).await;
            }
        }
        let _ = tokio::join!(
            blast_lane(socket.clone(), host_24[0..len0].to_vec(), p1_port, header, ip.clone()),
            blast_lane(socket.clone(), host_5g1[0..len1].to_vec(), p2_port, header, ip.clone()),
            blast_lane(socket.clone(), host_5g2[0..len2].to_vec(), p3_port, header, ip.clone()),
        );
        update_progress(block_idx + 1, total_blocks);
    }
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

    fn run_shredder(&mut self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        self.is_blasting = true;
        let tx = self.event_tx.clone();
        let path = self.file_path.clone().unwrap();
        let ip = self.target_ip.clone();
        let cancel_flag = self.cancel_flag.clone();
        let progress = std::sync::Arc::new(std::sync::Mutex::new((self.current_block, self.total_blocks)));
        let update_progress = {
            let progress = progress.clone();
            let ctx = egui::Context::default();
            std::sync::Arc::new(move |cur, total| {
                let mut p = progress.lock().unwrap();
                *p = (cur, total);
                ctx.request_repaint();
            }) as Arc<dyn Fn(usize, usize) + Send + Sync>
        };
        self.runtime.spawn({
            let update_progress = update_progress.clone();
            let mut_self: *mut MissionControlApp = self as *mut MissionControlApp;
            async move {
                let res = gui_shred_logic(tx.clone(), path, ip, cancel_flag, update_progress).await;
                // SAFETY: We know the runtime outlives this future
                unsafe {
                    (*mut_self).is_blasting = false;
                }
                if let Err(_e) = res {
                    // Optionally: update GUI with error
                }
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
        // Load environment variables from .env
        dotenvy::dotenv().ok();
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
