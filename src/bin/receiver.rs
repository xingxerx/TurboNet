use dotenvy::dotenv;
use std::env;
pub struct GhostReassembler {
    pub total_size: usize,
    pub weights: [u64; 3],
    pub salt: u64,
}

impl GhostReassembler {
    pub fn reassemble(
        &self, 
        b24: &[u8], 
        b5g1: &[u8], 
        b5g2: &[u8]
    ) -> Vec<u8> {
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
                let local_idx = (block_id * self.weights[1] + (pos_in_block - self.weights[0])) as usize;
                output.push(b5g1[local_idx]);
            } else {
                let local_idx = (block_id * self.weights[2] + (pos_in_block - self.weights[0] - self.weights[1])) as usize;
                output.push(b5g2[local_idx]);
            }
        }
        output
    }
}
use tokio::net::UdpSocket;
use pqc_kyber::keypair;
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
            name.contains("ethernet") || name.contains("wifi") || name.contains("wi-fi") || name.contains("lan")
        })
        .and_then(|iface| iface.addr.into_iter().find(|addr| addr.ip().is_ipv4() && !addr.ip().is_loopback()))
        .map(|addr| addr.ip().to_string())
        .or_else(|| local_ipaddress::get())
        .unwrap_or_else(|| "0.0.0.0".to_string());
    println!("📡 RECEIVER STACK ACTIVE: Listening on {}:{} {}:{} {}:{}", local_ip, lane1_port, local_ip, lane2_port, local_ip, lane3_port);
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
                eprintln!("Handshake PK_REQ mismatch: expected 6 bytes 'PK_REQ', got {} bytes", n);
            }
        }

        // 4. Level 11 Metadata Handshake
        let total_size = loop {
            let mut buf = [0u8; 512];
            let (n, addr) = sock_24.recv_from(&mut buf).await?;
            if n > 0 && buf[0] == b'M' {
                let fname_len = u32::from_be_bytes(buf[1..5].try_into().unwrap()) as usize;
                let fname = String::from_utf8_lossy(&buf[5..5 + fname_len]);
                let incoming_size = u64::from_be_bytes(buf[5 + fname_len..5 + fname_len + 8].try_into().unwrap()) as usize;
                println!("📦 METADATA: Filename: {}, Size: {} bytes", fname, incoming_size);
                
                let _ = sock_24.send_to(b"META_ACK", addr).await;
                break incoming_size;
            } else if n == 6 && &buf[..6] == b"PK_REQ" {
                let _ = sock_24.send_to(&public_key, addr).await;
            }
        };

        // 5. Receive Fragments with Header Stripping
        let mut data_buf = vec![0u8; total_size];
        let mut received = 0;
        println!("🚀 BLAST START: Expecting {} bytes...", total_size);

        while received < total_size {
            let mut packet = [0u8; 65536];
            let (dn, addr) = sock_24.recv_from(&mut packet).await?;

            // Handle probe packets (16 bytes starting with 0xFFFFFFFFFFFFFFFF)
            if dn == 16 && packet[0..8] == 0xFFFFFFFFFFFFFFFFu64.to_be_bytes() {
                let _ = sock_24.send_to(&packet[..dn], addr).await;
                continue;
            }

            // Skip 28-byte header packets (Salt, BlockID, Weights)
            if dn == 28 {
                continue;
            }

            // Accept raw data packets
            if dn > 0 {
                let to_copy = std::cmp::min(dn, total_size - received);
                data_buf[received..received + to_copy].copy_from_slice(&packet[..to_copy]);
                received += to_copy;
                
                if received % 10240 == 0 || received == total_size {
                    println!("🚀 Progress: {}/{} bytes", received, total_size);
                }
            }
        }

        // 6. Finalize Payload
        std::fs::write("output.jpg", &data_buf)?;
        println!("⚡ MISSION SUCCESS: Reassembled payload saved to output.jpg");
    }
}