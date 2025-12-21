use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("❌ Error: No Salt provided.");
        return Ok(());
    }
    let salt: u64 = args[1].parse().expect("Salt must be a u64");
    
    // MATCHING SHREDDER LOGIC:
    // Shredder uses: let magic = salt ^ 0xDEADBEEFDEADBEEF;
    let expected_sig = salt ^ 0xDEADBEEFDEADBEEF;

    println!("👻 GHOST RECEIVER ONLINE | SALT: {}", salt);

    // Listen on 0.0.0.0 to catch the "Bounce" from the router
    let l1 = UdpSocket::bind("0.0.0.0:8001").await?;
    let l2 = UdpSocket::bind("0.0.0.0:8002").await?;
    let l3 = UdpSocket::bind("0.0.0.0:8003").await?;

    println!("📡 Listening on 0.0.0.0 Ports 8001, 8002, 8003 (Router Bounce Mode)...");

    // Buffer size: 65535 is safe max for UDP
    let mut b1 = [0u8; 65535];
    let mut b2 = [0u8; 65535];
    let mut b3 = [0u8; 65535];

    // Wait for the Shredder to blast
    let (r1, r2, r3) = tokio::join!(
        l1.recv_from(&mut b1),
        l2.recv_from(&mut b2),
        l3.recv_from(&mut b3),
    );

    let (len1, _) = r1?;
    let (len2, _) = r2?;
    let (len3, _) = r3?;

    // 🛡️ VERIFY THE WAX SEAL (First 8 bytes of each packet)
    // Shredder sends as Big Endian (to_be_bytes), so we read as Big Endian
    let s1 = u64::from_be_bytes(b1[0..8].try_into().unwrap());
    let s2 = u64::from_be_bytes(b2[0..8].try_into().unwrap());
    let s3 = u64::from_be_bytes(b3[0..8].try_into().unwrap());

    if s1 == expected_sig && s2 == expected_sig && s3 == expected_sig {
        println!("✅ SIGNATURES MATCH. Security clear.");
        
        // Data is at b1[8..len1], etc.
        println!("📦 REASSEMBLED: {} total bytes received.", (len1+len2+len3) - 24);

        // --- REASSEMBLY LOGIC ---
        let payload_len = (len1 - 8) + (len2 - 8) + (len3 - 8);
        let mut reconstructed = vec![0u8; payload_len];

        // Reverse the Shredder's "Quantum Loop"
        for idx in 0..payload_len {
             let lane = (idx as u64 + salt) % 3;
             let lane_idx = idx / 3;
             
             // +8 to skip the Magic Header
             let byte = if lane == 0 { b1[8 + lane_idx] }
             else if lane == 1 { b2[8 + lane_idx] }
             else { b3[8 + lane_idx] };
             
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
                println!("🎉 SUCCESS: Image decrypted and saved to 'reborn_image.jpg'");
            },
            Err(_) => {
                println!("⛔ DECRYPTION FAILED: Auth Tag Mismatch! Data tainted.");
            }
        }
    } else {
        println!("🚫 SECURITY ALERT: Invalid signature detected! DROP DATA.");
        println!("   Expected: {:X}", expected_sig);
        println!("   Received: {:X}, {:X}, {:X}", s1, s2, s3);
    }

    Ok(())
}
