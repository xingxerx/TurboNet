//! TurboNet Unified CLI Orchestrator
//! 
//! Single entry point to run all TurboNet security tools with AI-driven defense.
//! 
//! Usage:
//!     cargo run -p turbonet-core --bin turbonet -- help
//!     cargo run -p turbonet-core --bin turbonet -- defend --input scan.json

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use turbonet_core::ai_defense::{DefenseAdvisor, ScanFindings, Finding, Severity};
use turbonet_core::ai_client::{AiClient, parse_model_spec};
use turbonet_core::world_gen::WorldGenerator;
use turbonet_core::brain::{Brain, Intent}; // Import Brain

#[derive(Parser)]
#[command(name = "turbonet")]
#[command(author = "xingxerx")]
#[command(version = "0.2.0")]
#[command(about = "TurboNet: Quantum-Hardened Security Toolkit with AI Defense", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// AI Defense Advisor - analyze scan results for hardening recommendations
    Defend {
        /// Path to scan findings JSON file (or use --demo for sample data)
        #[arg(long)]
        input: Option<PathBuf>,

        /// Run with demo/sample findings
        #[arg(long)]
        demo: bool,

        /// AI model to use (format: provider:model, e.g., ollama:gpt-oss)
        #[arg(long, default_value = "ollama:gpt-oss:20b")]
        model: String,

        /// Output format (json or text)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Infinite Procedural World Generator
    World {
        /// AI model to use
        #[arg(long, default_value = "ollama:gpt-oss")]
        model: String,

        /// World theme (e.g. "Cyberpunk", "Fantasy", "Zombie Apocalypse")
        #[arg(long, default_value = "Cyberpunk Neo-Tokyo")]
        theme: String,

        /// Enable NVIDIA GPU Acceleration for Chaos Theory (requires CUDA)
        #[arg(long)]
        nvidia: bool,

    },

    /// Run Physics Simulation (Spectre + Rapier3D)
    Simulate {
        /// Number of worlds to generate on GPU (Batch Size)
        #[arg(long, default_value_t = 16)]
        batch_size: usize,

        /// Simulation steps to run
        #[arg(long, default_value_t = 600)]
        steps: usize,
    },

    /// Run network scanner (port scan)
    Scan {
        /// Target IP or hostname
        target: String,

        /// Port range (e.g., 1-1000)
        #[arg(long, default_value = "1-1024")]
        ports: String,
    },

    /// Spectre GPU engine commands
    Spectre {
        #[command(subcommand)]
        action: SpectreAction,
    },

    /// Sentinel memory forensics
    Sentinel {
        #[command(subcommand)]
        action: SentinelAction,
    },

    /// Cyber Security Analyst - Active Traffic Guard
    Guard {
        /// Network interface to monitor (e.g., eth0, any)
        #[arg(long, default_value = "any")]
        interface: String,

        /// AI model for real-time traffic analysis
        #[arg(long, default_value = "ollama:gpt-oss:20b")]
        model: String,

        #[command(subcommand)]
        action: GuardAction,
    },

    /// The Brain: Intent-based Automatic Mode
    Brain {
        /// The query or intent (e.g., "scan localhost", "analyze malware")
        query: Vec<String>,

        /// AI model for intent classification
        #[arg(long, default_value = "ollama:gpt-oss")]
        model: String,

        /// Execute the action immediately without asking
        #[arg(long)]
        force: bool,
    },

    /// List all available tools
    List,

    /// Show system info and dependencies
    Info,
}

#[derive(Subcommand)]
enum SpectreAction {
    /// Mutate payload with polymorphic engine
    Mutate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Quantum threat analysis
    Quantum {
        #[arg(long, default_value = "aes")]
        algorithm: String,
        #[arg(long, default_value = "256")]
        key_size: u32,
    },
    /// Calculate file entropy
    Entropy {
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Subcommand)]
enum SentinelAction {
    /// Memory scan for RWX regions
    Memscan {
        #[arg(long)]
        pid: Option<u32>,
    },
    /// Detect inline hooks
    Hooks,
    /// Token enumeration
    Tokens,
}

#[derive(Subcommand)]
enum GuardAction {
    /// Start active defense
    Start {
        #[arg(long, default_value = "8888")]
        port: u16,
        #[arg(long, default_value = "ollama:gpt-oss")]
        model: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If arguments are provided (other than the binary name), run once.
    if std::env::args().count() > 1 {
        let cli = Cli::parse();
        process_command(cli.command).await?;
    } else {
        // Otherwise enter interactive mode
        run_interactive_mode().await?;
    }
    Ok(())
}

async fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to TurboNet Interactive Mode (v0.2.0)");
    println!("Type 'help' for commands, 'exit' to quit.");

    let mut rl = rustyline::DefaultEditor::new()?;
    
    loop {
        let readline = rl.readline("turbonet> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() { continue; }
                if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
                    break;
                }
                
                let _ = rl.add_history_entry(line);
                
                let args = match shlex::split(line) {
                    Some(a) => a,
                    None => {
                        eprintln!("Error: Invalid quoting");
                        continue;
                    }
                };
                
                let mut full_args = vec!["turbonet".to_string()];
                full_args.extend(args);

                match Cli::try_parse_from(full_args) {
                    Ok(cli) => {
                        if let Err(e) = process_command(cli.command).await {
                             eprintln!("Error: {}", e);
                        }
                    },
                    Err(e) => {
                       let _ = e.print();
                    }
                }
            },
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}

async fn process_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Defend { input, demo, model, format } => {
            run_defense_advisor(input, demo, &model, &format).await?;
        }
        Commands::World { model, theme, nvidia } => {
            run_world_generator(&model, &theme, nvidia).await?;
        }
        Commands::Scan { target, ports } => {
            println!("🔍 Scanning {} ports {}...", target, ports);
            println!("   → Run: cargo run -p tools --bin net-sniffer -- scan {}", target);
        }
        Commands::Simulate { batch_size, steps } => {
            run_simulation(batch_size, steps).await?;
        }
        Commands::Spectre { action } => {
            match action {
                SpectreAction::Mutate { input, output } => {
                    let out = output.unwrap_or_else(|| input.with_extension("mutated.bin"));
                    println!("🦠 Spectre Mutate: {:?} → {:?}", input, out);
                    println!("   → Run: cargo run -p spectre -- mutate --input {:?}", input);
                }
                SpectreAction::Quantum { algorithm, key_size } => {
                    println!("⚛️ Quantum Threat Analysis: {} @ {} bits", algorithm, key_size);
                    println!("   → Run: python py_src/quantum_engine.py --algorithm {} --key-size {}", algorithm, key_size);
                }
                SpectreAction::Entropy { input } => {
                    println!("📊 Entropy Analysis: {:?}", input);
                    println!("   → Run: cargo run -p spectre -- entropy --input {:?}", input);
                }
            }
        }
        Commands::Sentinel { action } => {
            match action {
                SentinelAction::Memscan { pid } => {
                    let target = pid.map(|p| format!("PID {}", p)).unwrap_or("all processes".to_string());
                    println!("🛡️ Memory Scan: {}", target);
                    println!("   → Run: cargo run -p sentinel --bin sentinel-memscan -- --list");
                }
                SentinelAction::Hooks => {
                    println!("🪝 Hook Detection");
                    println!("   → Run: cargo run -p sentinel --bin hook-detector");
                }
                SentinelAction::Tokens => {
                    println!("🎫 Token Enumeration");
                    println!("   → Run: cargo run -p sentinel --bin token-steal");
                }
            }
        }
        Commands::List => {
            print_tool_list();
        }
        Commands::Info => {
            print_system_info();
        }
        Commands::Guard { interface: _interface, model, action, .. } => {
            match action {
                GuardAction::Start { port, .. } => {
                     println!("🛡️ Starting AI Traffic Guard...");
                     let port_str = port.to_string();
                     let args = vec!["run", "-p", "tools", "--bin", "net-guard", "--", "--run", &port_str, &model];
                     println!("   → Run: cargo {}", args.join(" "));
                    
                    // In a real CLI we might spawn this directly, but for now we print the command 
                    // consistent with other tools in this CLIwrapper.
                    // However, let's actually run it for the user if they want interactive mode.
                    use std::process::Command;
                    
                    let status = Command::new("cargo")
                        .args(&args)
                        .status()?;
                        
                    if !status.success() {
                        eprintln!("Guard process exited with error");
                    }
                }
            }
        }
        Commands::Brain { query, model, force } => {
            let query_str = query.join(" ");
            run_brain_mode(&query_str, &model, force).await?;
        }
    }
    Ok(())
}

async fn run_defense_advisor(
    input: Option<PathBuf>,
    demo: bool,
    model: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ TurboNet AI Defense Advisor");
    println!("═══════════════════════════════════════════════════════════");

    // Load findings
    let findings = if demo {
        println!("📋 Using demo scan findings...\n");
        demo_findings()
    } else if let Some(path) = input {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        return Err("Provide --input <file> or use --demo".into());
    };

    // Parse model spec
    let (provider, model_name) = parse_model_spec(model);
    println!("🤖 AI Model: {}:{}", provider, model_name);
    println!("📍 Target: {}", findings.target);
    println!("🔍 Findings: {} items\n", findings.findings.len());

    // Create advisor
    let advisor = match provider.as_str() {
        "ollama" => DefenseAdvisor::ollama(&model_name),
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            DefenseAdvisor::openai_compatible(
                "https://api.openai.com/v1/chat/completions",
                &model_name,
                api_key.as_deref(),
            )
        }
        _ => DefenseAdvisor::ollama(&model_name),
    };

    println!("⏳ Analyzing with AI (this may take a moment)...\n");

    match advisor.suggest_defenses(&findings).await {
        Ok(report) => {
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_defense_report(&report);
            }
        }
        Err(e) => {
            eprintln!("❌ AI Analysis Failed: {}", e);
            eprintln!("\n💡 Tips:");
            eprintln!("   • Ensure Ollama is running: ollama serve");
            eprintln!("   • Pull the model: ollama pull {}", model_name);
            eprintln!("   • Or use a different model: --model ollama:llama3");
        }
    }

    Ok(())
}

fn demo_findings() -> ScanFindings {
    ScanFindings {
        tool: "net-sniffer".to_string(),
        target: "192.168.1.100".to_string(),
        findings: vec![
            Finding {
                severity: Severity::Critical,
                category: "Open Ports".to_string(),
                description: "SSH port 22 exposed with password auth enabled".to_string(),
                evidence: Some("OpenSSH 7.9p1 detected".to_string()),
            },
            Finding {
                severity: Severity::High,
                category: "Outdated Software".to_string(),
                description: "Apache 2.4.29 has known CVEs".to_string(),
                evidence: Some("CVE-2021-44790, CVE-2022-22720".to_string()),
            },
            Finding {
                severity: Severity::Medium,
                category: "Misconfiguration".to_string(),
                description: "SMB signing not required".to_string(),
                evidence: Some("Port 445/tcp open".to_string()),
            },
            Finding {
                severity: Severity::Low,
                category: "Information Disclosure".to_string(),
                description: "Server banner reveals version info".to_string(),
                evidence: Some("HTTP Server: Apache/2.4.29 (Ubuntu)".to_string()),
            },
        ],
    }
}

fn print_defense_report(report: &turbonet_core::ai_defense::DefenseReport) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              🛡️ AI DEFENSE RECOMMENDATIONS               ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("📝 Summary:");
    println!("   {}\n", report.summary);

    if !report.recommendations.is_empty() {
        println!("🎯 Recommendations:");
        for rec in &report.recommendations {
            println!("   [P{}] {} ", rec.priority, rec.title);
            println!("       └─ {}", rec.description);
            println!("       └─ Fix: {}\n", rec.implementation);
        }
    }

    if !report.firewall_rules.is_empty() {
        println!("🔥 Firewall Rules:");
        for rule in &report.firewall_rules {
            println!("   • {}", rule);
        }
        println!();
    }

    if !report.patches.is_empty() {
        println!("📦 Patches/Updates:");
        for patch in &report.patches {
            println!("   • {}", patch);
        }
        println!();
    }
    println!("═══════════════════════════════════════════════════════════");
}

use turbonet_core::spectre::SpectreEngine;

async fn run_simulation(batch_size: usize, steps: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running Physics Simulation (Spectre + Rapier3D)");
    println!("   • Batch Size: {}", batch_size);
    println!("   • Steps: {}", steps);
    
    // For now, just a placeholder or basic loop
    // In a real implementation we would initialize PhysicsWorld here
    println!("   (Simulation logic would run here)");
    
    Ok(())
}

async fn run_world_generator(model: &str, theme: &str, use_nvidia: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 TurboNet Infinite World Generator");
    println!("Theme: {}", theme);
    
    // Select model logic
    let mut selected_model = model.to_string();
    if use_nvidia && model == "ollama:gpt-oss" {
        println!("🚀 NVIDIA Mode Active: Switching to DeepSeek-Coder for advanced reasoning...");
        selected_model = "ollama:deepseek-coder".to_string();
    }
    println!("Model: {}", selected_model);

    // Initialize Spectre (Chaos Engine) if requested
    let mut chaos_engine = None;
    if use_nvidia {
        println!("⚛️ Initializing SPECTRE-GPU for Quantum Entropy...");
        match SpectreEngine::new() {
            Ok(engine) => {
                println!("   → GPU Online. Chaos Injection Enabled.");
                chaos_engine = Some(engine);
            }
            Err(e) => {
                eprintln!("   ⚠️ Failed to load CUDA engine: {}", e);
                eprintln!("   → Falling back to standard CPU physics.");
            }
        }
    }

    println!("Loading...");

    let client = AiClient::from_spec(&selected_model);
    let mut world = WorldGenerator::new(client, theme, chaos_engine);

    // Initial scene
    match world.initialize().await {
        Ok(scene) => {
            println!("\n╔══════════════════════════════════════════════════════════╗");
            println!("{}\n", textwrap::fill(&scene, 60));
            println!("╚══════════════════════════════════════════════════════════╝");
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("connection refused") || err_msg.contains("target machine actively refused") {
                 eprintln!("\n❌ Connection Failed: Could not connect to the AI engine.");
                 eprintln!("   Please ensure Ollama is running (e.g., `ollama serve`).");
            } else if err_msg.contains("404") || err_msg.contains("not found") {
                 eprintln!("\n❌ Model Missing: The AI model could not be found.");
                 eprintln!("   Please run: `ollama pull deepseek-coder`");
                 eprintln!("   (Or specify a different model with --model)");
            } else {
                 eprintln!("Error initializing world: {}", e);
            }
            return Ok(());
        }
    }

    let mut rl = rustyline::DefaultEditor::new()?;
    
    loop {
        let readline = rl.readline("\n> ");
        match readline {
            Ok(line) => {
                let action = line.trim();
                if action.is_empty() { continue; }
                if action.eq_ignore_ascii_case("exit") || action.eq_ignore_ascii_case("quit") {
                    break;
                }
                
                let _ = rl.add_history_entry(action);
                
                println!("Thinking...");
                match world.next_turn(action).await {
                    Ok(response) => {
                        println!("\n{}", textwrap::fill(&response, 60));
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
            Err(_) => break,
        }
    }
    Ok(())
}

fn print_tool_list() {
    println!(r#"
╔══════════════════════════════════════════════════════════════════╗
║                    🚀 TURBONET TOOLKIT                           ║
╠══════════════════════════════════════════════════════════════════╣
║  CORE                                                            ║
║    shred          High-speed file transfer (sender)              ║
║    receiver       Multi-lane UDP receiver                        ║
║                                                                  ║
║  SPECTRE (GPU Engine)                                            ║
║    spectre mutate     Polymorphic payload generation             ║
║    spectre quantum    Post-quantum threat analysis               ║
║    spectre entropy    File entropy calculation                   ║
║                                                                  ║
║  SENTINEL (Memory Forensics)                                     ║
║    sentinel-memscan   RWX/MZ region detection                    ║
║    hook-detector      Inline hook scanning                       ║
║    token-steal        Access token enumeration                   ║
║    proc-hollow        Process injection demo                     ║
║                                                                  ║
║  TOOLS (Analysis)                                                ║
║    pe-parser          PE file structure analysis                 ║
║    strings-extract    String extraction                          ║
║    net-sniffer        UDP listener + port scan                   ║
║    beacon-gen         C2 beacon generator                        ║
║                                                                  ║
║  WIFI-RECON                                                      ║
║    quantum-hound      AI-driven WiFi auditing                    ║
║    wifi-scan          Network interface detection                ║
║                                                                  ║
║  AI DEFENSE                                                      ║
║    turbonet defend    AI-powered defense recommendations         ║
║                                                                  ║
║  WORLD GEN                                                       ║
║    turbonet world     Infinite Procedural World Generator        ║
╚══════════════════════════════════════════════════════════════════╝
"#);
}

fn print_system_info() {
    println!("🖥️ TurboNet System Info");
    println!("═══════════════════════════════════════════════════════════");
    println!("  Version:    0.2.0");
    println!("  Edition:    2021");
    println!("  OS:         {}", std::env::consts::OS);
    println!("  Arch:       {}", std::env::consts::ARCH);
    println!();
    println!("📦 Workspace Crates:");
    println!("  • turbonet-core   (file transfer, AI defense)");
    println!("  • spectre         (GPU polymorphic engine)");
    println!("  • sentinel        (memory forensics)");
    println!("  • tools           (PE parser, sniffer, beacon)");
    println!("  • wifi-recon      (quantum-hound)");
    println!();
    println!("🤖 AI Backends Supported:");
    println!("  • GPT-OSS (default) → ollama:gpt-oss:20b (OpenAI open-weight)");
    println!("  • Ollama (local)    → ollama:llama3, ollama:deepseek-coder");
    println!("  • OpenAI (cloud)    → openai:gpt-4o (requires OPENAI_API_KEY)");
    println!("═══════════════════════════════════════════════════════════");
}

async fn run_brain_mode(
    query: &str,
    model: &str,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 TurboNet Brain: Analyzing intent for '{}'...", query);

    let brain = Brain::new(Some(model.to_string()));
    let intent = brain.perceive(query).await?;
    
    // In a real agentic loop, we would act on the intent here.
    // For now, we delegate to the Brain to tell us what it WOULD do.
    let plan = brain.process_intent(intent.clone());

    println!("{}", plan);

    if force {
        println!("🚀 Force enabled. Executing logic (mock)...");
        // Here we would actually call the sub-functions (run_scan, etc.)
        // This connects the specific "section that is needed".
        match intent {
             Intent::Scan { target, ports } => {
                 // Call the existing logic
                 println!("   [Firing Network Scanner]");
                 println!("🔍 Scanning {} ports {}...", target, ports);
             }
             Intent::Defend { input: _ } => {
                 println!("   [Firing Defense Advisor]");
                 // In real impl, we would need the input path
             }
             Intent::World { theme } => {
                 println!("   [Firing World Gen]");
                 run_world_generator(model, &theme, false).await?;
             }
             _ => {}
        }
    } else {
        println!("\n(Run with --force to execute automatically)");
    }

    Ok(())
}

