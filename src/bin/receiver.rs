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
    let mut rng = rand::thread_rng();

    // 1. Level 9: Generate Lattice Keypair (Quantum-Safe)
    let _keys = keypair(&mut rng).expect("Failed to generate Kyber-768 keys");
    println!("🗝️  Lattice Public Key generated. Ready for Handshake.");

    // 2. Open listeners for the 3 network bands
    let sock_24 = Arc::new(UdpSocket::bind("0.0.0.0:8001").await?);
    let _sock_5g1 = Arc::new(UdpSocket::bind("0.0.0.0:8002").await?);
    let _sock_5g2 = Arc::new(UdpSocket::bind("0.0.0.0:8003").await?);

    println!("📡 Ghost Receiver listening on ports 8001, 8002, 8003...");

    // 3. Wait for handshake header (file size)
    let mut header_buf = [0u8; 8];
    let (n, addr) = sock_24.recv_from(&mut header_buf).await?;
    if n != 8 {
        eprintln!("Handshake header size mismatch: expected 8, got {}", n);
        return Ok(());
    }
    let total_size = u64::from_be_bytes(header_buf) as usize;
    println!("🔔 Handshake Received! Incoming Payload: {} bytes", total_size);

    // 4. Receive the actual data (simulate with a single UDP packet for now)
    let mut data_buf = vec![0u8; total_size];
    let (data_n, _data_addr) = sock_24.recv_from(&mut data_buf).await?;
    println!("📥 Received {} bytes from {}", data_n, addr);

    // TODO: Pass data_buf to reassembly/decryption logic
    Ok(())
}