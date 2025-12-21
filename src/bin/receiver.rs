use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("❌ Error: Usage: receiver <SALT> <EXPECTED_SIZE>");
        return Ok(());
    }
    let salt: u64 = args[1].parse().expect("Salt must be a u64");
    let n: usize = args[2].parse().expect("Size must be a usize");
    
    let expected_sig = salt ^ 0xDEADBEEFDEADBEEF;

    println!("👻 GHOST RECEIVER ONLINE | SALT: {} | TARGET: {} bytes", salt, n);

    // Listen on 0.0.0.0 to catch the "Bounce" from the router
    let l1 = UdpSocket::bind("0.0.0.0:8001").await?;
    let l2 = UdpSocket::bind("0.0.0.0:8002").await?;
    let l3 = UdpSocket::bind("0.0.0.0:8003").await?;

    println!("📡 Listening on Ports 8001-8003. Waiting for Jumbo Shred...");

    let get_lane_len = |n: usize, salt: u64, lane: u64| -> usize {
        let offset = (lane + 6000000 - (salt % 3)) % 3;
        if (offset as usize) < n {
            (n - 1 - offset as usize) / 3 + 1
        } else {
            0
        }
    };

    let len0 = get_lane_len(n, salt, 0);
    let len1 = get_lane_len(n, salt, 1);
    let len2 = get_lane_len(n, salt, 2);

    // Helper to collect a lane
    async fn collect_lane(socket: UdpSocket, target_len: usize, expected_sig: u64) -> Vec<u8> {
        let mut buffer = vec![0u8; target_len];
        let mut temp_buf = [0u8; 65535];
        
        // 1. Wait for Header (Magic)
        loop {
            let (len, _) = socket.recv_from(&mut temp_buf).await.unwrap();
            // println!("DEBUG: Received {} bytes (Waiting for Magic)", len);
            if len == 8 {
                let sig = u64::from_be_bytes(temp_buf[0..8].try_into().unwrap());
                if sig == expected_sig {
                    println!("🎯 Magic Header Found!");
                    break;
                }
            }
        }

        // 2. Collect Data Chunks
        let mut received = 0;
        while received < target_len {
            let (len, _) = socket.recv_from(&mut temp_buf).await.unwrap();
            // println!("DEBUG: Received {} data bytes", len);
            let end = (received + len).min(target_len);
            let to_copy = end - received;
            buffer[received..end].copy_from_slice(&temp_buf[0..to_copy]);
            received += to_copy;
        }
        buffer
    }

    println!("📥 Receiving data fragments...");
    let (lane0, lane1, lane2) = tokio::join!(
        collect_lane(l1, len0, expected_sig),
        collect_lane(l2, len1, expected_sig),
        collect_lane(l3, len2, expected_sig),
    );

    println!("✅ ALL FRAGMENTS COLLECTED. Reassembling...");

    // --- REASSEMBLY LOGIC ---
    let mut reconstructed = vec![0u8; n];
    for idx in 0..n {
         let lane = (idx as u64 + salt) % 3;
         let lane_idx = idx / 3;
         let byte = if lane == 0 { lane0[lane_idx] }
         else if lane == 1 { lane1[lane_idx] }
         else { lane2[lane_idx] };
         reconstructed[idx] = byte;
    }

    // --- DECRYPTION LOGIC ---
    println!("🔓 DECRYPTING content...");
    let key_material = salt.to_be_bytes(); 
    let mut full_key = [0u8; 32];
    for i in 0..4 { full_key[i*8..(i+1)*8].copy_from_slice(&key_material); }
    let key = Key::<Aes256Gcm>::from_slice(&full_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]);
    
    match cipher.decrypt(nonce, reconstructed.as_ref()) {
        Ok(plaintext) => {
            std::fs::write("reborn_image.jpg", plaintext).expect("Failed to write reborn file");
            println!("🎉 SUCCESS: Image reassembled and decrypted!");
        },
        Err(e) => {
            println!("⛔ DECRYPTION FAILED: Auth Tag Mismatch! ({})", e);
        }
    }

    Ok(())
}
