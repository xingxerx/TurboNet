use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("❌ Usage: receiver <SALT> <SIZE>");
        return Ok(());
    }
    let salt: u64 = args[1].parse().expect("Invalid Salt");
    let n: usize = args[2].parse().expect("Invalid Size");
    let expected_sig = salt ^ 0xDEADBEEFDEADBEEF;

    println!("👻 GHOST RECEIVER ONLINE | SALT: {} | TARGET: {} bytes", salt, n);

    // --- NETWORK CONFIG FROM .ENV ---
    dotenvy::dotenv().ok();
    let p1_str = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string());
    let p2_str = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string());
    let p3_str = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string());

    let l1 = UdpSocket::bind(format!("0.0.0.0:{}", p1_str)).await?;
    let l2 = UdpSocket::bind(format!("0.0.0.0:{}", p2_str)).await?;
    let l3 = UdpSocket::bind(format!("0.0.0.0:{}", p3_str)).await?;

    println!("📡 Listening on Ports {}, {}, {}. Waiting...", p1_str, p2_str, p3_str);

    let get_lane_len = |n: usize, salt: u64, lane: u64| -> usize {
        let offset = (lane + 6000000 - (salt % 3)) % 3;
        if (offset as usize) < n { (n - 1 - offset as usize) / 3 + 1 } else { 0 }
    };

    let len0 = get_lane_len(n, salt, 0);
    let len1 = get_lane_len(n, salt, 1);
    let len2 = get_lane_len(n, salt, 2);

    async fn collect_lane(socket: UdpSocket, target_len: usize, expected_sig: u64, port: u16) -> Vec<u8> {
        let mut buffer = vec![0u8; target_len];
        let mut temp_buf = [0u8; 65535];
        
        loop {
            let (len, _) = socket.recv_from(&mut temp_buf).await.unwrap();
            if len == 8 {
                let sig = u64::from_be_bytes(temp_buf[0..8].try_into().unwrap());
                if sig == expected_sig {
                    println!("🎯 Port {}: Magic Header Received.", port);
                    break;
                }
            }
        }

        let mut received = 0;
        let mut last_report = 0;
        while received < target_len {
            let (len, _) = socket.recv_from(&mut temp_buf).await.unwrap();
            let end = (received + len).min(target_len);
            let to_copy = end - received;
            buffer[received..end].copy_from_slice(&temp_buf[0..to_copy]);
            received += to_copy;
            
            if received - last_report >= 4096 || received == target_len {
                println!("📥 Port {}: Progress {}/{} [{}%]", port, received, target_len, (received*100)/target_len);
                last_report = received;
            }
        }
        println!("✅ Port {}: Lane Complete.", port);
        buffer
    }

    println!("📡 Listening... Waiting for Sequential Shred...");
    let (lane0, lane1, lane2) = tokio::join!(
        collect_lane(l1, len0, expected_sig, 8001),
        collect_lane(l2, len1, expected_sig, 8002),
        collect_lane(l3, len2, expected_sig, 8003),
    );

    println!("✅ REASSEMBLING...");
    let mut reconstructed = vec![0u8; n];
    for idx in 0..n {
         let lane = (idx as u64 + salt) % 3;
         let lane_idx = idx / 3;
         let byte = if lane == 0 { lane0[lane_idx] }
         else if lane == 1 { lane1[lane_idx] }
         else { lane2[lane_idx] };
         reconstructed[idx] = byte;
    }

    println!("🔓 DECRYPTING...");
    let key_material = salt.to_be_bytes(); 
    let mut full_key = [0u8; 32];
    for i in 0..4 { full_key[i*8..(i+1)*8].copy_from_slice(&key_material); }
    let key = Key::<Aes256Gcm>::from_slice(&full_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]);
    
    match cipher.decrypt(nonce, reconstructed.as_ref()) {
        Ok(plaintext) => {
            std::fs::write("reborn_image.jpg", plaintext).expect("Write fail");
            println!("🎉 SUCCESS: reborn_image.jpg is READY.");
        },
        Err(e) => println!("⛔ DECRYPTION FAILED: {}", e),
    }
    Ok(())
}
