//! Strings Extractor - Extract ASCII/Unicode strings with entropy scoring
//!
//! Part of TurboNet Security Toolkit

use std::env;
use std::fs::File;
use std::io::Read;

const MIN_STRING_LEN: usize = 4;

fn main() {
    print_banner();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }
    
    let mut min_len = MIN_STRING_LEN;
    let mut show_offset = false;
    let mut path = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                if i + 1 < args.len() {
                    min_len = args[i + 1].parse().unwrap_or(MIN_STRING_LEN);
                    i += 1;
                }
            }
            "-o" => show_offset = true,
            "--help" | "-h" => {
                print_usage(&args[0]);
                return;
            }
            _ => path = Some(args[i].clone()),
        }
        i += 1;
    }
    
    if let Some(p) = path {
        if let Err(e) = extract_strings(&p, min_len, show_offset) {
            eprintln!("[!] Error: {}", e);
        }
    } else {
        print_usage(&args[0]);
    }
}

fn print_banner() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║   ███████╗████████╗██████╗ ██╗███╗   ██╗ ██████╗ ███████╗     ║
║   ██╔════╝╚══██╔══╝██╔══██╗██║████╗  ██║██╔════╝ ██╔════╝     ║
║   ███████╗   ██║   ██████╔╝██║██╔██╗ ██║██║  ███╗███████╗     ║
║   ╚════██║   ██║   ██╔══██╗██║██║╚██╗██║██║   ██║╚════██║     ║
║   ███████║   ██║   ██║  ██║██║██║ ╚████║╚██████╔╝███████║     ║
║   ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝     ║
║              String Extractor v0.1                             ║
╚═══════════════════════════════════════════════════════════════╝
"#);
}

fn print_usage(prog: &str) {
    println!("Usage: {} [OPTIONS] <FILE>", prog);
    println!();
    println!("Options:");
    println!("  -n <LEN>   Minimum string length (default: 4)");
    println!("  -o         Show file offset");
    println!();
    println!("Examples:");
    println!("  {} malware.exe", prog);
    println!("  {} -n 8 -o suspicious.dll", prog);
}

fn extract_strings(path: &str, min_len: usize, show_offset: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] Extracting strings from: {}", path);
    println!("[*] Minimum length: {}\n", min_len);
    
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    let mut strings: Vec<(usize, String, f32)> = Vec::new();
    
    // Extract ASCII strings
    let mut current = String::new();
    let mut start_offset = 0;
    
    for (i, &byte) in data.iter().enumerate() {
        if byte >= 0x20 && byte < 0x7F {
            if current.is_empty() {
                start_offset = i;
            }
            current.push(byte as char);
        } else {
            if current.len() >= min_len {
                let entropy = calculate_string_entropy(&current);
                strings.push((start_offset, current.clone(), entropy));
            }
            current.clear();
        }
    }
    
    // Extract wide (UTF-16 LE) strings
    for i in (0..data.len().saturating_sub(1)).step_by(2) {
        if data[i + 1] == 0 && data[i] >= 0x20 && data[i] < 0x7F {
            if current.is_empty() {
                start_offset = i;
            }
            current.push(data[i] as char);
        } else {
            if current.len() >= min_len {
                let entropy = calculate_string_entropy(&current);
                // Avoid duplicates
                if !strings.iter().any(|(_, s, _)| s == &current) {
                    strings.push((start_offset, current.clone(), entropy));
                }
            }
            current.clear();
        }
    }
    
    // Sort by offset
    strings.sort_by_key(|(offset, _, _)| *offset);
    
    // Categorize interesting strings
    let mut urls = Vec::new();
    let mut ips = Vec::new();
    let mut paths = Vec::new();
    let mut registry = Vec::new();
    let mut suspicious = Vec::new();
    
    for (offset, s, entropy) in &strings {
        let lower = s.to_lowercase();
        
        if lower.contains("http://") || lower.contains("https://") {
            urls.push((offset, s, entropy));
        } else if is_ip_address(&lower) {
            ips.push((offset, s, entropy));
        } else if lower.contains(":\\") || lower.starts_with("/") {
            paths.push((offset, s, entropy));
        } else if lower.contains("hkey_") || lower.contains("software\\") {
            registry.push((offset, s, entropy));
        } else if is_suspicious(&lower) {
            suspicious.push((offset, s, entropy));
        }
    }
    
    // Print categorized results
    if !urls.is_empty() {
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  URLs ({})                                                    ║", urls.len());
        println!("╚═══════════════════════════════════════════════════════════════╝");
        for (offset, s, _) in &urls {
            if show_offset {
                println!("  0x{:08X}: {}", offset, s);
            } else {
                println!("  {}", s);
            }
        }
        println!();
    }
    
    if !ips.is_empty() {
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  IP Addresses ({})                                            ║", ips.len());
        println!("╚═══════════════════════════════════════════════════════════════╝");
        for (offset, s, _) in &ips {
            if show_offset {
                println!("  0x{:08X}: {}", offset, s);
            } else {
                println!("  {}", s);
            }
        }
        println!();
    }
    
    if !registry.is_empty() {
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  Registry Keys ({})                                           ║", registry.len());
        println!("╚═══════════════════════════════════════════════════════════════╝");
        for (offset, s, _) in &registry {
            if show_offset {
                println!("  0x{:08X}: {}", offset, s);
            } else {
                println!("  {}", s);
            }
        }
        println!();
    }
    
    if !suspicious.is_empty() {
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ⚠ Suspicious Strings ({})                                    ║", suspicious.len());
        println!("╚═══════════════════════════════════════════════════════════════╝");
        for (offset, s, entropy) in &suspicious {
            if show_offset {
                println!("  0x{:08X}: {} [entropy: {:.2}]", offset, s, entropy);
            } else {
                println!("  {} [entropy: {:.2}]", s, entropy);
            }
        }
        println!();
    }
    
    println!("[*] Total strings found: {}", strings.len());
    println!("[*] URLs: {} | IPs: {} | Paths: {} | Registry: {} | Suspicious: {}",
             urls.len(), ips.len(), paths.len(), registry.len(), suspicious.len());
    
    Ok(())
}

fn calculate_string_entropy(s: &str) -> f32 {
    let mut counts = [0u32; 256];
    for byte in s.bytes() {
        counts[byte as usize] += 1;
    }
    
    let len = s.len() as f32;
    let mut entropy = 0.0f32;
    
    for &count in &counts {
        if count > 0 {
            let p = count as f32 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

fn is_ip_address(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() == 4 {
        parts.iter().all(|p| p.parse::<u8>().is_ok())
    } else {
        false
    }
}

fn is_suspicious(s: &str) -> bool {
    let suspicious_keywords = [
        "password", "passwd", "credential", "login", "admin",
        "shell", "cmd.exe", "powershell", "rundll", "regsvr",
        "keylog", "capture", "inject", "hook", "bypass",
        "decrypt", "encrypt", "base64", "xor", "obfuscate",
        "mimikatz", "metasploit", "cobalt", "beacon",
        "virtualallocex", "writeprocessmemory", "createremotethread",
        "ntwritevirtualmemory", "loadlibrary", "getprocaddress",
    ];
    
    suspicious_keywords.iter().any(|&kw| s.contains(kw))
}
