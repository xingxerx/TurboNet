use std::sync::atomic::{AtomicBool, Ordering};
// Architectural Design: Isolated Async Task
pub fn start_mission_control_task(_app_arc: std::sync::Arc<std::sync::Mutex<MissionControlApp>>) {
    std::thread::spawn(move || {
        // 1. Initialize Runtime
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        // 2. Execute Async Block
        rt.block_on(async {
            // Logic: Your network/shredder initialization here
            let res: Result<(), Box<dyn std::error::Error>> = async {
                // Example: while loop logic
                // ...existing code for your async logic...
                Ok(())
            }.await;

            if let Err(e) = res {
                eprintln!("[Systems] Mission Control Task Failure: {}", e);
            }
        }); // End block_on
    }); // End thread::spawn
}
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

// --- NEW: Message-based async logic for GUI state updates ---
use cudarc::driver::CudaDevice;
// use tokio::net::UdpSocket; // Uncomment if used later
use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;
// use aes_gcm::{Aes256Gcm, Key, Nonce}; // Uncomment if needed for encryption
// use aes_gcm::aead::Aead; // Uncomment if used later
use pqc_kyber::*;
use std::fs;

#[allow(dead_code)]
enum GuiMsg {
    Progress(usize, usize),
    Done,
    Error(String),
}
#[allow(dead_code)]
fn spawn_gui_shred_logic(
    path: PathBuf,
    ip: String,
    _cancel_flag: Arc<AtomicBool>,
    gui_tx: std::sync::mpsc::Sender<GuiMsg>,
) {
    #[allow(dead_code)]
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let gui_tx_clone = gui_tx.clone();
        let res: Result<(), Box<dyn std::error::Error>> = rt.block_on(async move {
            let p1_port: u16 = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap();
            let _p2_port: u16 = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string()).parse().unwrap();
            let _p3_port: u16 = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string()).parse().unwrap();
            let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();
            let _w0_env: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
            let _w1_env: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
            let _w2_env: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
            let ptx_src = std::fs::read_to_string("shredder.cu")?;
            let ptx = cudarc::nvrtc::compile_ptx(ptx_src)?;
            let dev = CudaDevice::new(0)?;
            dev.load_ptx(ptx, "shredder", &["shred_kernel"])?;

            let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
            sock.set_reuse_address(true)?;
            sock.set_recv_buffer_size(4 * 1024 * 1024)?;
            sock.set_send_buffer_size(4 * 1024 * 1024)?;
            sock.bind(&"0.0.0.0:0".parse::<SocketAddr>()?.into())?;
            let std_socket: std::net::UdpSocket = sock.into();
            let socket = std::sync::Arc::new(std_socket);

            let file_bytes = fs::read(&path)?;
            let total_len = file_bytes.len();
            let _total_blocks = (total_len + block_size - 1) / block_size;

            // Handshake
            socket.send_to(b"PK_REQ", format!("{}:{}", ip, p1_port))?;
            let mut pk_buf = [0u8; KYBER_PUBLICKEYBYTES];
            let (n, _) = socket.recv_from(&mut pk_buf)?;
            if n != KYBER_PUBLICKEYBYTES { return Err("Invalid PK size".into()); }
            let mut rng = rand::rngs::OsRng;
            let (_ct, shared_secret) = encapsulate(&pk_buf, &mut rng).map_err(|_| "Encapsulation failed")?;
            let _quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());

            // Metadata
            let mut meta = vec![b'M'];
            let fname = path.file_name().unwrap().to_str().unwrap_or("payload");
            meta.extend_from_slice(&(fname.len() as u32).to_be_bytes());
            meta.extend_from_slice(fname.as_bytes());
            meta.extend_from_slice(&(total_len as u64).to_be_bytes());
            let mut meta_confirmed = false;
            while !meta_confirmed {
                socket.send_to(&meta, format!("{}:{}", ip, p1_port))?;
                let mut ack_buf = [0u8; 64];
                if let Ok((n, _)) = socket.recv_from(&mut ack_buf) {
                    let msg = String::from_utf8_lossy(&ack_buf[..n]);
                    if msg.starts_with("META_ACK") {
                        meta_confirmed = true;
                    }
                }
            }

            // ...rest of the logic for blasting lanes and progress...
            // (You may need to re-add CUDA-accelerated logic and progress updates here)

            gui_tx_clone.send(GuiMsg::Done).ok();
            Ok(())
        });
        if let Err(e) = res {
            gui_tx.send(GuiMsg::Error(format!("{:?}", e))).ok();
        }
    });
}
// ...existing code...

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
#[allow(dead_code)]
    Error,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct MissionControlApp {
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
    #[allow(dead_code)]
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

    fn run_shredder(app_arc: std::sync::Arc<std::sync::Mutex<MissionControlApp>>) {
        // All async/threaded logic now uses start_mission_control_task
        start_mission_control_task(app_arc);
    }
    #[allow(dead_code)]

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
                    // Use Arc<Mutex<...>> for thread-safe state as required
                    let app_arc = std::sync::Arc::new(std::sync::Mutex::new(self.clone()));
                    MissionControlApp::run_shredder(app_arc);
                }

                // Always show the STOP button, but only enable it if blasting
                ui.add_space(10.0);
                let _stop_btn = ui.add_enabled(self.is_blasting, egui::Button::new(egui::RichText::new("🛑 STOP").size(20.0).color(egui::Color32::RED)));
                // STOP button logic can be implemented here if needed
                if self.is_blasting {
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
    // Initialize MissionControlApp inside Arc<Mutex<...>> for safe cross-thread access
    // Removed unused app_arc variable
    // Pass a clone to the GUI/event loop if needed (for future expansion)
    eframe::run_native(
        "TurboNet Mission Control",
        options,
        Box::new(|cc| {
            // The GUI gets its own Arc<Mutex<...>>
            let app = MissionControlApp::new(cc);
            Ok(Box::new(app))
        }),
    )
}
// Helper for correct socket setup: returns a TokioUdpSocket from a std::net::UdpSocket
#[allow(dead_code)]
fn setup_tokio_socket(sock: std::net::UdpSocket) -> std::io::Result<tokio::net::UdpSocket> {
    sock.set_nonblocking(true)?;
    tokio::net::UdpSocket::from_std(sock)
}

