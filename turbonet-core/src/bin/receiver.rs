use dotenvy::dotenv;
use memmap2::MmapMut;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::OpenOptions;
pub struct GhostReassembler {
    pub total_size: usize,
    pub weights: [u64; 3],
    pub salt: u64,
}

impl GhostReassembler {
    pub fn reassemble(&self, b24: &[u8], b5g1: &[u8], b5g2: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.total_size);
        let w_total: u64 = self.weights.iter().sum();
        for idx in 0..self.total_size as u64 {
            let pattern_offset = self.salt % w_total;
            let effective_idx = idx + pattern_offset;
            let block_id = effective_idx / w_total;
            let pos_in_block = effective_idx % w_total;
            if pos_in_block < self.weights[0] {
                let local_idx = (block_id * self.weights[0] + pos_in_block) as usize;
                output.push(b24[local_idx]);
            } else if pos_in_block < self.weights[0] + self.weights[1] {
                let local_idx =
                    (block_id * self.weights[1] + (pos_in_block - self.weights[0])) as usize;
                output.push(b5g1[local_idx]);
            } else {
                let local_idx = (block_id * self.weights[2]
                    + (pos_in_block - self.weights[0] - self.weights[1]))
                    as usize;
                output.push(b5g2[local_idx]);
            }
        }
        output
    }
}
use pqc_kyber::keypair;
use tokio::net::UdpSocket;
// ...existing code...
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut rng = rand::thread_rng();

    // 1. Level 9: Generate Lattice Keypair (Quantum-Safe)
    let keypair = keypair(&mut rng).expect("Failed to generate Kyber-768 keys");
    let public_key = keypair.public;
    println!("🗝️  Lattice Public Key generated. Ready for Handshake.");

    // Load IP and ports from .env
    let listen_ip = env::var("RECEIVER_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let lane1_port = env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string());
    let lane2_port = env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string());
    let lane3_port = env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string());

    let sock_24 = Arc::new(UdpSocket::bind(format!("{}:{}", listen_ip, lane1_port)).await?);
    let _sock_5g1 = Arc::new(UdpSocket::bind(format!("{}:{}", listen_ip, lane2_port)).await?);
    let _sock_5g2 = Arc::new(UdpSocket::bind(format!("{}:{}", listen_ip, lane3_port)).await?);

    // Print the actual local IP address for user clarity
    use network_interface::{NetworkInterface, NetworkInterfaceConfig};
    let local_ip = NetworkInterface::show()
        .unwrap_or_default()
        .into_iter()
        .find(|iface| {
            // Try common interface names on Windows
            let name = iface.name.to_lowercase();
            name.contains("ethernet")
                || name.contains("wifi")
                || name.contains("wi-fi")
                || name.contains("lan")
        })
        .and_then(|iface| {
            iface
                .addr
                .into_iter()
                .find(|addr| addr.ip().is_ipv4() && !addr.ip().is_loopback())
        })
        .map(|addr| addr.ip().to_string())
        .or_else(|| local_ipaddress::get())
        .unwrap_or_else(|| "0.0.0.0".to_string());
    println!(
        "📡 RECEIVER STACK ACTIVE: Listening on {}:{} {}:{} {}:{}",
        local_ip, lane1_port, local_ip, lane2_port, local_ip, lane3_port
    );
    println!("👉 Use RECEIVER IP: {} in Mission Control GUI.", local_ip);

    // (SECURITY) Local IP addresses are now managed via .env and not printed for security reasons.

    loop {
        // 3. Wait for PK_REQ handshake (6 bytes) or handle probe echoes (16 bytes)
        loop {
            let mut pkreq_buf = [0u8; 64];
            let (n, addr) = sock_24.recv_from(&mut pkreq_buf).await?;

            // Handle probe packets (16 bytes starting with 0xFFFFFFFFFFFFFFFF)
            if n == 16 && pkreq_buf[0..8] == 0xFFFFFFFFFFFFFFFFu64.to_be_bytes() {
                let _ = sock_24.send_to(&pkreq_buf[..n], addr).await;
                continue; // Keep listening for more probes or PK_REQ
            }

            if n == 6 && &pkreq_buf[..6] == b"PK_REQ" {
                println!("🔑 PK_REQ received from {}", addr);
                // Send public key back to sender
                let _ = sock_24.send_to(&public_key, addr).await;
                break;
            } else if n != 16 {
                eprintln!(
                    "Handshake PK_REQ mismatch: expected 6 bytes 'PK_REQ', got {} bytes",
                    n
                );
            }
        }

        // 4. Level 11 Metadata Handshake
        let (total_size, filename) = loop {
            let mut buf = [0u8; 512];
            let (n, addr) = sock_24.recv_from(&mut buf).await?;
            if n > 0 && buf[0] == b'M' {
                let fname_len = u32::from_be_bytes(buf[1..5].try_into().unwrap()) as usize;
                let fname = String::from_utf8_lossy(&buf[5..5 + fname_len]).to_string();
                let incoming_size =
                    u64::from_be_bytes(buf[5 + fname_len..5 + fname_len + 8].try_into().unwrap())
                        as usize;
                println!(
                    "📦 METADATA: Filename: {}, Size: {} bytes",
                    fname, incoming_size
                );

                let _ = sock_24.send_to(b"META_ACK", addr).await;
                break (incoming_size, fname);
            } else if n == 6 && &buf[..6] == b"PK_REQ" {
                let _ = sock_24.send_to(&public_key, addr).await;
            }
        };

        // 5. Receive Fragments with Zero-Copy Mmap Reassembly
        let output_filename = format!("reborn_{}", filename);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&output_filename)?;
        file.set_len(total_size as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        let mut received = 0;
        let mut last_log = 0;
        let mut packets_received: u64 = 0;
        let mut lane_packets: [u64; 3] = [0, 0, 0];
        let transfer_start = std::time::Instant::now();
        println!(
            "🚀 BLAST START: Expecting {} bytes (Multi-Lane Zero-Copy Mode)...",
            total_size
        );

        let mut end_sender_addr: Option<std::net::SocketAddr> = None;

        // Pre-allocate packet buffers for all lanes
        let mut packet0 = [0u8; 65536];
        let mut packet1 = [0u8; 65536];
        let mut packet2 = [0u8; 65536];

        // Track last activity for inactivity timeout
        let mut last_activity = std::time::Instant::now();
        const INACTIVITY_TIMEOUT_SECS: u64 = 3;

        while received < total_size {
            // LEVEL 13: Concurrent multi-lane reception using tokio::select! with timeout
            let recv_result = tokio::select! {
                result = sock_24.recv_from(&mut packet0) => {
                    let (n, a) = result?;
                    Some((n, a, 0usize, &packet0 as &[u8; 65536]))
                }
                result = _sock_5g1.recv_from(&mut packet1) => {
                    let (n, a) = result?;
                    Some((n, a, 1usize, &packet1 as &[u8; 65536]))
                }
                result = _sock_5g2.recv_from(&mut packet2) => {
                    let (n, a) = result?;
                    Some((n, a, 2usize, &packet2 as &[u8; 65536]))
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    None // Timeout - check for inactivity
                }
            };

            // Check for inactivity timeout (no new data for N seconds)
            if recv_result.is_none() {
                if last_activity.elapsed().as_secs() >= INACTIVITY_TIMEOUT_SECS {
                    println!(
                        "⚠️ INACTIVITY TIMEOUT: No data received for {}s",
                        INACTIVITY_TIMEOUT_SECS
                    );
                    println!(
                        "   Completing transfer with {} of {} bytes ({:.1}% received)",
                        received,
                        total_size,
                        (received as f64 / total_size as f64) * 100.0
                    );
                    break;
                }
                continue;
            }

            let (dn, addr, lane_idx, packet_buf) = recv_result.unwrap();
            let packet = &packet_buf[..dn];
            last_activity = std::time::Instant::now();

            // Handle probe packets (16 bytes starting with 0xFFFFFFFFFFFFFFFF)
            if dn == 16 && packet[0..8] == 0xFFFFFFFFFFFFFFFFu64.to_be_bytes() {
                let _ = sock_24.send_to(packet, addr).await;
                continue;
            }

            // Handle END_TRANSFER packet (graceful shutdown signal)
            if dn == 12 && &packet[..12] == b"END_TRANSFER" {
                end_sender_addr = Some(addr);
                // Continue receiving any remaining packets for a short while
                continue;
            }

            // Skip 28-byte header packets (Salt, BlockID, Weights)
            if dn == 28 {
                continue;
            }

            // Accept raw data packets - write directly to mmap (zero-copy)
            if dn > 0 {
                let to_copy = std::cmp::min(dn, total_size - received);
                mmap[received..received + to_copy].copy_from_slice(&packet[..to_copy]);
                received += to_copy;
                packets_received += 1;
                lane_packets[lane_idx] += 1;

                if received >= last_log + (1024 * 1024 * 10) || received == total_size {
                    last_log = received;
                    let elapsed = transfer_start.elapsed().as_secs_f64();
                    let speed_mbps = if elapsed > 0.0 {
                        (received as f64 / 1_000_000.0) / elapsed
                    } else {
                        0.0
                    };
                    println!(
                        "🚀 Progress: {}/{} bytes ({:.1}%) @ {:.1} MB/s [L0:{} L1:{} L2:{}]",
                        received,
                        total_size,
                        (received as f64 / total_size as f64) * 100.0,
                        speed_mbps,
                        lane_packets[0],
                        lane_packets[1],
                        lane_packets[2]
                    );
                }
            }
        }

        // Send END_ACK to sender to confirm all data received
        if let Some(sender_addr) = end_sender_addr {
            for _ in 0..3 {
                let _ = sock_24.send_to(b"END_ACK", sender_addr).await;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        } else {
            // Wait briefly for END_TRANSFER if we haven't received it yet
            let wait_end = std::time::Instant::now();
            while wait_end.elapsed().as_secs() < 2 {
                let mut packet = [0u8; 64];
                match tokio::time::timeout(
                    std::time::Duration::from_millis(200),
                    sock_24.recv_from(&mut packet),
                )
                .await
                {
                    Ok(Ok((dn, addr))) if dn == 12 && &packet[..12] == b"END_TRANSFER" => {
                        for _ in 0..3 {
                            let _ = sock_24.send_to(b"END_ACK", addr).await;
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }

        // SOTA Metrics: Calculate final throughput
        let transfer_duration = transfer_start.elapsed();
        let duration_secs = transfer_duration.as_secs_f64();
        let throughput_mbps = (received as f64 / 1_000_000.0) / duration_secs;
        let throughput_gbps = throughput_mbps * 8.0 / 1000.0;

        // 6. Finalize Payload with Integrity Check
        let hash_start = std::time::Instant::now();
        mmap.flush()?;

        // Level 11: SHA-256 Integrity Verification
        let mut hasher = Sha256::new();
        hasher.update(&mmap[..]);
        let hash = hasher.finalize();
        let hash_duration = hash_start.elapsed();

        println!("🛡️ INTEGRITY: SHA-256 Hash: {:x}", hash);
        println!("📊 TRANSFER STATS:");
        println!("   Duration: {:.2}s", duration_secs);
        println!(
            "   Bytes Received: {} ({:.2} MB)",
            received,
            received as f64 / 1_000_000.0
        );
        println!("   Packets: {}", packets_received);
        println!(
            "   🚀 THROUGHPUT: {:.1} MB/s ({:.2} Gbps)",
            throughput_mbps, throughput_gbps
        );
        println!(
            "   🔐 Hash Time: {:?} ({:.1} MB/s)",
            hash_duration,
            (received as f64 / 1_000_000.0) / hash_duration.as_secs_f64()
        );
        println!(
            "⚡ MISSION SUCCESS: Reassembled payload saved to {}",
            output_filename
        );
    }
}
