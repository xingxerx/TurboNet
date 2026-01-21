//! PE Parser - Portable Executable File Analyzer
//!
//! Analyzes Windows PE files: headers, sections, imports, exports, entropy.
//! Part of TurboNet Security Toolkit

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    match args[1].as_str() {
        "--help" | "-h" => print_usage(&args[0]),
        path => {
            if let Err(e) = analyze_pe(path) {
                eprintln!("[!] Error: {}", e);
            }
        }
    }
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║   ██████╗ ███████╗    ██████╗  █████╗ ██████╗ ███████╗███████╗║
║   ██╔══██╗██╔════╝    ██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔════╝║
║   ██████╔╝█████╗█████╗██████╔╝███████║██████╔╝███████╗█████╗  ║
║   ██╔═══╝ ██╔══╝╚════╝██╔═══╝ ██╔══██║██╔══██╗╚════██║██╔══╝  ║
║   ██║     ███████╗    ██║     ██║  ██║██║  ██║███████║███████╗║
║   ╚═╝     ╚══════╝    ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝║
║              Portable Executable Analyzer v0.1                 ║
╚═══════════════════════════════════════════════════════════════╝
"#
    );
}

fn print_usage(prog: &str) {
    println!("Usage: {} <PE_FILE>", prog);
    println!();
    println!("Examples:");
    println!("  {} notepad.exe", prog);
    println!("  {} malware.dll", prog);
}

fn analyze_pe(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] Analyzing: {}\n", path);

    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    // Read DOS header
    let mut dos_header = [0u8; 64];
    file.read_exact(&mut dos_header)?;

    // Check MZ signature
    if dos_header[0] != 0x4D || dos_header[1] != 0x5A {
        return Err("Not a valid PE file (missing MZ signature)".into());
    }

    // Get PE header offset (e_lfanew at offset 0x3C)
    let pe_offset = u32::from_le_bytes([
        dos_header[60],
        dos_header[61],
        dos_header[62],
        dos_header[63],
    ]);

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  DOS HEADER                                                   ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Magic:           MZ (0x5A4D)                                 ║");
    println!(
        "║  PE Offset:       0x{:08X}                                   ║",
        pe_offset
    );
    println!(
        "║  File Size:       {} bytes ({:.2} KB)                     ║",
        file_size,
        file_size as f64 / 1024.0
    );

    // Read PE signature
    file.seek(SeekFrom::Start(pe_offset as u64))?;
    let mut pe_sig = [0u8; 4];
    file.read_exact(&mut pe_sig)?;

    if pe_sig != [0x50, 0x45, 0x00, 0x00] {
        return Err("Invalid PE signature".into());
    }

    // Read COFF header (20 bytes)
    let mut coff_header = [0u8; 20];
    file.read_exact(&mut coff_header)?;

    let machine = u16::from_le_bytes([coff_header[0], coff_header[1]]);
    let num_sections = u16::from_le_bytes([coff_header[2], coff_header[3]]);
    let timestamp = u32::from_le_bytes([
        coff_header[4],
        coff_header[5],
        coff_header[6],
        coff_header[7],
    ]);
    let characteristics = u16::from_le_bytes([coff_header[18], coff_header[19]]);
    let optional_header_size = u16::from_le_bytes([coff_header[16], coff_header[17]]);

    let arch = match machine {
        0x14c => "x86 (32-bit)",
        0x8664 => "x64 (64-bit)",
        0xAA64 => "ARM64",
        _ => "Unknown",
    };

    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  COFF HEADER                                                  ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Machine:         0x{:04X} ({})                         ║",
        machine, arch
    );
    println!(
        "║  Sections:        {}                                          ║",
        num_sections
    );
    println!(
        "║  Timestamp:       0x{:08X}                                   ║",
        timestamp
    );
    println!(
        "║  Characteristics: 0x{:04X}                                       ║",
        characteristics
    );

    // Parse characteristics flags
    print_characteristics(characteristics);

    // Read Optional Header
    if optional_header_size > 0 {
        let mut opt_header = vec![0u8; optional_header_size as usize];
        file.read_exact(&mut opt_header)?;

        let magic = u16::from_le_bytes([opt_header[0], opt_header[1]]);
        let is_pe32_plus = magic == 0x20b;

        let entry_point = u32::from_le_bytes([
            opt_header[16],
            opt_header[17],
            opt_header[18],
            opt_header[19],
        ]);
        let image_base: u64 = if is_pe32_plus {
            u64::from_le_bytes([
                opt_header[24],
                opt_header[25],
                opt_header[26],
                opt_header[27],
                opt_header[28],
                opt_header[29],
                opt_header[30],
                opt_header[31],
            ])
        } else {
            u32::from_le_bytes([
                opt_header[28],
                opt_header[29],
                opt_header[30],
                opt_header[31],
            ]) as u64
        };

        let subsystem_offset = if is_pe32_plus { 68 } else { 64 };
        let subsystem = u16::from_le_bytes([
            opt_header[subsystem_offset],
            opt_header[subsystem_offset + 1],
        ]);

        let subsystem_name = match subsystem {
            1 => "Native",
            2 => "Windows GUI",
            3 => "Windows Console",
            _ => "Other",
        };

        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║  OPTIONAL HEADER                                              ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!(
            "║  Magic:           0x{:04X} ({})                          ║",
            magic,
            if is_pe32_plus { "PE32+" } else { "PE32" }
        );
        println!(
            "║  Entry Point:     0x{:08X}                                   ║",
            entry_point
        );
        println!(
            "║  Image Base:      0x{:016X}                       ║",
            image_base
        );
        println!(
            "║  Subsystem:       {} ({})                              ║",
            subsystem, subsystem_name
        );
    }

    // Read section headers
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  SECTIONS                                                     ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Name       VirtAddr   VirtSize   RawSize    Entropy  Flags   ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");

    for _ in 0..num_sections {
        let mut section = [0u8; 40];
        file.read_exact(&mut section)?;

        let name: String = section[0..8]
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as char)
            .collect();

        let virtual_size = u32::from_le_bytes([section[8], section[9], section[10], section[11]]);
        let virtual_addr = u32::from_le_bytes([section[12], section[13], section[14], section[15]]);
        let raw_size = u32::from_le_bytes([section[16], section[17], section[18], section[19]]);
        let raw_offset = u32::from_le_bytes([section[20], section[21], section[22], section[23]]);
        let characteristics =
            u32::from_le_bytes([section[36], section[37], section[38], section[39]]);

        // Calculate entropy for this section
        let entropy = if raw_size > 0 {
            let current_pos = file.stream_position()?;
            file.seek(SeekFrom::Start(raw_offset as u64))?;
            let mut data = vec![0u8; std::cmp::min(raw_size as usize, 65536)];
            let _ = file.read(&mut data);
            file.seek(SeekFrom::Start(current_pos))?;
            calculate_entropy(&data)
        } else {
            0.0
        };

        let flags = format_section_flags(characteristics);
        let entropy_indicator = if entropy > 7.0 { "⚠" } else { " " };

        println!(
            "║  {:8} 0x{:08X} 0x{:08X} 0x{:08X} {:5.2}{} {:6} ║",
            name, virtual_addr, virtual_size, raw_size, entropy, entropy_indicator, flags
        );
    }

    println!("╚═══════════════════════════════════════════════════════════════╝");

    // Summary
    println!();
    println!("[*] Analysis complete");

    Ok(())
}

fn print_characteristics(chars: u16) {
    if chars & 0x0002 != 0 {
        println!("║    - EXECUTABLE_IMAGE                                         ║");
    }
    if chars & 0x0020 != 0 {
        println!("║    - LARGE_ADDRESS_AWARE                                      ║");
    }
    if chars & 0x0100 != 0 {
        println!("║    - 32BIT_MACHINE                                            ║");
    }
    if chars & 0x2000 != 0 {
        println!("║    - DLL                                                      ║");
    }
}

fn format_section_flags(chars: u32) -> String {
    let mut flags = String::new();
    if chars & 0x20000000 != 0 {
        flags.push('X');
    } else {
        flags.push('-');
    }
    if chars & 0x40000000 != 0 {
        flags.push('R');
    } else {
        flags.push('-');
    }
    if chars & 0x80000000 != 0 {
        flags.push('W');
    } else {
        flags.push('-');
    }
    flags
}

fn calculate_entropy(data: &[u8]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0u32; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f32;
    let mut entropy = 0.0f32;

    for &count in &counts {
        if count > 0 {
            let p = count as f32 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}
