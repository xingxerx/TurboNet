//! Hook Detector - Detect inline hooks in processes
//!
//! Part of TurboNet Security Toolkit

use std::env;

#[cfg(windows)]
mod detector {
    use std::ffi::c_void;
    use std::mem;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    use windows::Win32::System::Memory::*;
    use windows::Win32::System::Threading::*;

    pub fn list_processes() {
        unsafe {
            let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(s) => s,
                Err(_) => return,
            };

            let mut entry = PROCESSENTRY32W {
                dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            println!("{:<8} | {}", "PID", "Process Name");
            println!("{}", "-".repeat(50));

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let name: String = entry
                        .szExeFile
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                        .collect();
                    println!("{:<8} | {}", entry.th32ProcessID, name);
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
    }

    pub fn scan_process(pid: u32) {
        unsafe {
            let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
            {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[!] Failed to open process: {:?}", e);
                    eprintln!("[!] Try running as Administrator");
                    return;
                }
            };

            println!("[*] Scanning PID {} for hooks...\n", pid);
            println!("[*] Checking memory regions for hook signatures...\n");

            let mut address: usize = 0;
            let mut hooks_found = 0;
            let mut regions_scanned = 0;

            loop {
                let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();
                let result = VirtualQueryEx(
                    handle,
                    Some(address as *const c_void),
                    &mut mbi,
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );

                if result == 0 {
                    break;
                }

                // Check executable regions
                let protect = mbi.Protect.0;
                let is_executable = (protect & 0x10) != 0 || // PAGE_EXECUTE
                                    (protect & 0x20) != 0 || // PAGE_EXECUTE_READ
                                    (protect & 0x40) != 0 || // PAGE_EXECUTE_READWRITE
                                    (protect & 0x80) != 0; // PAGE_EXECUTE_WRITECOPY

                if mbi.State.0 == MEM_COMMIT.0 && is_executable && mbi.RegionSize > 0 {
                    regions_scanned += 1;

                    // Read first bytes of the region
                    let mut buffer = [0u8; 16];
                    let mut bytes_read = 0;

                    if ReadProcessMemory(
                        handle,
                        mbi.BaseAddress,
                        buffer.as_mut_ptr() as *mut c_void,
                        buffer.len(),
                        Some(&mut bytes_read),
                    )
                    .is_ok()
                        && bytes_read >= 8
                        && is_hook_signature(&buffer)
                    {
                        hooks_found += 1;
                        println!(
                            "[!] Potential hook at 0x{:016X}: {:02X?}",
                            mbi.BaseAddress as usize,
                            &buffer[..8]
                        );
                    }
                }

                address = mbi.BaseAddress as usize + mbi.RegionSize;
                if address == 0 {
                    break;
                }
            }

            println!("\n[*] Scanned {} executable regions", regions_scanned);
            println!("[*] Potential hooks found: {}", hooks_found);

            if hooks_found == 0 {
                println!("[+] No obvious hooks detected");
            } else {
                println!("[!] Review flagged addresses - may be JIT, hooks, or trampolines");
            }

            let _ = CloseHandle(handle);
        }
    }

    fn is_hook_signature(bytes: &[u8]) -> bool {
        if bytes.len() < 2 {
            return false;
        }

        // Common hook patterns at function entry:
        // E9 xx xx xx xx - JMP rel32 (5 bytes)
        // 68 xx xx xx xx C3 - PUSH addr + RET (6 bytes)
        // FF 25 xx xx xx xx - JMP [addr] (6 bytes)
        // Note: We're looking for these at region starts, which is suspicious

        // Don't flag normal function prologues (push ebp, etc)
        if bytes[0] == 0x55 {
            return false;
        } // push ebp
        if bytes[0] == 0x48 && bytes[1] == 0x89 {
            return false;
        } // mov [rsp+...], rbx etc

        // Flag suspicious patterns
        if bytes[0] == 0xE9 {
            return true;
        } // JMP rel32
        if bytes[0] == 0x68 && bytes.len() > 5 && bytes[5] == 0xC3 {
            return true;
        } // PUSH+RET
        if bytes[0] == 0xFF && bytes[1] == 0x25 {
            return true;
        } // JMP [addr]

        false
    }
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║   ██╗  ██╗ ██████╗  ██████╗ ██╗  ██╗                          ║
║   ██║  ██║██╔═══██╗██╔═══██╗██║ ██╔╝                          ║
║   ███████║██║   ██║██║   ██║█████╔╝                           ║
║   ██╔══██║██║   ██║██║   ██║██╔═██╗                           ║
║   ██║  ██║╚██████╔╝╚██████╔╝██║  ██╗                          ║
║   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝                          ║
║            DETECTOR - API Hook Scanner v0.1                    ║
╚═══════════════════════════════════════════════════════════════╝
"#
    );
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --list              List running processes", prog);
    println!("  {} --pid <PID>         Scan process for hooks", prog);
    println!();
    println!("Examples:");
    println!("  {} --pid 1234", prog);
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    #[cfg(windows)]
    match args[1].as_str() {
        "--list" | "-l" => detector::list_processes(),
        "--pid" | "-p" => {
            if args.len() > 2 {
                if let Ok(pid) = args[2].parse::<u32>() {
                    detector::scan_process(pid);
                } else {
                    eprintln!("[!] Invalid PID");
                }
            }
        }
        "--help" | "-h" => print_usage(&args[0]),
        _ => print_usage(&args[0]),
    }

    #[cfg(not(windows))]
    eprintln!("[!] Hook Detector is Windows-only");
}
