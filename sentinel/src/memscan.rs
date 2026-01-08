//! Sentinel-MemScan: Heuristic Memory Forensics Tool
//!
//! Scans process memory to detect fileless attack indicators:
//! - RWX (Read-Write-Execute) memory regions
//! - Injected PE headers (MZ magic bytes)
//!
//! Part of TurboNet Quantum-Hardened Security Toolkit

use std::env;

#[cfg(windows)]
mod scanner {
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    use windows::Win32::System::Memory::*;
    use windows::Win32::System::Threading::*;

    /// Represents a suspicious memory region found in a process
    #[allow(dead_code)]
    pub struct SuspiciousRegion {
        pub base_address: usize,
        pub size: usize,
        pub protection: PAGE_PROTECTION_FLAGS,
        pub reason: String,
    }

    /// Find process ID by name using Toolhelp32Snapshot
    pub fn get_process_id_by_name(name: &str) -> Option<u32> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    // Convert wide string to Rust String
                    let process_name: String = entry
                        .szExeFile
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                        .collect();

                    if process_name.to_lowercase() == name.to_lowercase() {
                        let _ = CloseHandle(snapshot);
                        return Some(entry.th32ProcessID);
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
            None
        }
    }

    /// Scan a specific process for suspicious memory regions
    pub fn scan_process(pid: u32) {
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            );

            let handle = match handle {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[!] Failed to open process PID {}: {:?}", pid, e);
                    eprintln!("[!] Try running as Administrator");
                    return;
                }
            };

            println!("[*] Scanning process PID: {}", pid);
            println!();
            println!(
                "{:<18} | {:<12} | {:<24} | {}",
                "Address", "Size", "Protection", "Suspicion"
            );
            println!("{}", "-".repeat(85));

            let mut address: usize = 0;
            let mut suspicious_count = 0;
            let mut total_regions = 0;

            loop {
                let mut mem_info = MEMORY_BASIC_INFORMATION::default();

                let result = VirtualQueryEx(
                    handle,
                    Some(address as *const std::ffi::c_void),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );

                if result == 0 {
                    break;
                }

                total_regions += 1;
                let protection = mem_info.Protect;
                let size = mem_info.RegionSize;
                let base = mem_info.BaseAddress as usize;

                let mut suspicion = String::new();
                let mut is_suspicious = false;

                // 1. Check for RWX Permissions (Read/Write/Execute)
                // Normal code is RX. Data is RW. RWX is highly suspicious for injected shellcode.
                if protection.contains(PAGE_EXECUTE_READWRITE) {
                    suspicion.push_str("[RWX DETECTED] ");
                    is_suspicious = true;
                }

                // 2. Scan for PE Headers in executable memory
                // If we find "MZ" (4D 5A) in a suspicious section, it's likely injected.
                if is_suspicious || protection.contains(PAGE_EXECUTE_READ) {
                    if let Ok(bytes) = read_process_memory(&handle, base, 2) {
                        if bytes.len() >= 2 && bytes[0] == 0x4D && bytes[1] == 0x5A {
                            suspicion.push_str("[MZ HEADER] ");
                            is_suspicious = true;
                        }
                    }
                }

                if is_suspicious {
                    suspicious_count += 1;
                    println!(
                        "{:#018x} | {:<12} | {:?} | {}",
                        base,
                        format_size(size),
                        protection,
                        suspicion.trim()
                    );
                }

                // Move to next region
                address = base.saturating_add(size);
                if address == 0 || size == 0 {
                    break;
                }
            }

            println!("{}", "-".repeat(85));
            println!(
                "[*] Scan complete: {} regions scanned, {} suspicious",
                total_regions, suspicious_count
            );

            if suspicious_count == 0 {
                println!("[+] No suspicious memory regions detected.");
            } else {
                println!("[!] Review flagged regions for potential threats.");
            }

            let _ = CloseHandle(handle);
        }
    }

    /// Helper to read raw bytes from process memory safely
    fn read_process_memory(
        handle: &HANDLE,
        address: usize,
        size: usize,
    ) -> Result<Vec<u8>, ()> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        unsafe {
            let result = ReadProcessMemory(
                *handle,
                address as *const std::ffi::c_void,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                size,
                Some(&mut bytes_read),
            );

            if result.is_ok() && bytes_read == size {
                return Ok(buffer);
            }
        }
        Err(())
    }

    /// Format byte size to human-readable string
    fn format_size(bytes: usize) -> String {
        if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    /// List all running processes
    pub fn list_processes() {
        unsafe {
            let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[!] Failed to create snapshot: {:?}", e);
                    return;
                }
            };

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            println!("{:<8} | {}", "PID", "Process Name");
            println!("{}", "-".repeat(50));

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let process_name: String = entry
                        .szExeFile
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                        .collect();

                    println!("{:<8} | {}", entry.th32ProcessID, process_name);

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }
    }
}

fn print_banner() {
    println!(r#"
  ╔═══════════════════════════════════════════════════════════════╗
  ║   ███████╗███████╗███╗   ██╗████████╗██╗███╗   ██╗███████╗██╗ ║
  ║   ██╔════╝██╔════╝████╗  ██║╚══██╔══╝██║████╗  ██║██╔════╝██║ ║
  ║   ███████╗█████╗  ██╔██╗ ██║   ██║   ██║██╔██╗ ██║█████╗  ██║ ║
  ║   ╚════██║██╔══╝  ██║╚██╗██║   ██║   ██║██║╚██╗██║██╔══╝  ██║ ║
  ║   ███████║███████╗██║ ╚████║   ██║   ██║██║ ╚████║███████╗███╗║
  ║   ╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝  ╚═══╝╚══════╝╚══╝║
  ║                  MEMSCAN - Heuristic Memory Forensics         ║
  ╚═══════════════════════════════════════════════════════════════╝
"#);
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} <process_name>     Scan a process by name", prog);
    println!("  {} --pid <PID>        Scan a process by PID", prog);
    println!("  {} --list             List all running processes", prog);
    println!();
    println!("Examples:");
    println!("  {} notepad.exe", prog);
    println!("  {} --pid 1234", prog);
    println!();
    println!("Detection Capabilities:");
    println!("  [RWX DETECTED]  Read-Write-Execute memory (shellcode indicator)");
    println!("  [MZ HEADER]     PE header in memory (injected module)");
    println!();
    println!("⚠️  Requires Administrator privileges for cross-process scanning");
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    #[cfg(windows)]
    {
        match args[1].as_str() {
            "--list" | "-l" => {
                println!("[*] Listing all running processes...\n");
                scanner::list_processes();
            }
            "--pid" | "-p" => {
                if args.len() < 3 {
                    eprintln!("[!] Please provide a PID");
                    return;
                }
                match args[2].parse::<u32>() {
                    Ok(pid) => {
                        println!("[*] Targeting PID: {}\n", pid);
                        scanner::scan_process(pid);
                    }
                    Err(_) => eprintln!("[!] Invalid PID: {}", args[2]),
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
            }
            target_name => {
                println!("[*] Targeting process: {}\n", target_name);
                match scanner::get_process_id_by_name(target_name) {
                    Some(pid) => {
                        println!("[+] Found process with PID: {}", pid);
                        scanner::scan_process(pid);
                    }
                    None => {
                        eprintln!("[!] Process '{}' not found.", target_name);
                        eprintln!("[*] Use --list to see running processes.");
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        eprintln!("[!] Sentinel-MemScan is Windows-only.");
        eprintln!("[!] This tool uses Windows APIs: VirtualQueryEx, ReadProcessMemory");
    }
}
