use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::Write;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

fn get_hits(t: u64, w_total: u64, w_target: u64, offset: u64) -> u64 {
    if t == 0 { return 0; }
    let cycles = t / w_total;
    let rem = t % w_total;
    let mut hits = cycles * w_target;
    if rem > offset {
        hits += (rem - offset).min(w_target);
    }
    hits
}

fn get_lane_len_asymmetric(n: usize, salt: u64, w0: i32, w1: i32, w2: i32, lane: i32) -> usize {
    let w_total = (w0 + w1 + w2) as u64;
    let pattern_offset = salt % w_total;
    let t_start = pattern_offset;
    let t_end = pattern_offset + n as u64;
    
    let (w_target, offset) = match lane {
        0 => (w0 as u64, 0u64),
        1 => (w1 as u64, w0 as u64),
        2 => (w2 as u64, (w0 + w1) as u64),
        _ => unreachable!(),
    };

    let hits_end = get_hits(t_end, w_total, w_target, offset);
    let hits_start = get_hits(t_start, w_total, w_target, offset);
    (hits_end - hits_start) as usize
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("❌ Usage: receiver <SALT> <TOTAL_SIZE>");
        return Ok(());
    }
    let salt: u64 = args[1].parse().expect("Invalid Salt");
    let total_expected: usize = args[2].parse().expect("Invalid Total Size");

    println!("👻 GHOST RECEIVER v2.0 | SALT: {} | EXPECTING: {} bytes", salt, total_expected);

    let p1_port = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string());
    let p2_port = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string());
    let p3_port = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string());
    
    let w0: i32 = std::env::var("SHRED_W0").unwrap_or_else(|_| "10".to_string()).parse().unwrap();
    let w1: i32 = std::env::var("SHRED_W1").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let w2: i32 = std::env::var("SHRED_W2").unwrap_or_else(|_| "45".to_string()).parse().unwrap();
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let l1 = UdpSocket::bind(format!("0.0.0.0:{}", p1_port)).await?;
    let l2 = UdpSocket::bind(format!("0.0.0.0:{}", p2_port)).await?;
    let l3 = UdpSocket::bind(format!("0.0.0.0:{}", p3_port)).await?;

    let total_blocks = (total_expected + block_size - 1) / block_size;
    
    let _ = std::fs::remove_file("reborn_image.jpg");
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("reborn_image.jpg")?;

    async fn collect_block(l1: &UdpSocket, l2: &UdpSocket, l3: &UdpSocket, salt: u64, b_idx: u32, w0: i32, w1: i32, w2: i32, enc_n: usize) -> Vec<u8> {
        let len0 = get_lane_len_asymmetric(enc_n, salt, w0, w1, w2, 0);
        let len1 = get_lane_len_asymmetric(enc_n, salt, w0, w1, w2, 1);
        let len2 = get_lane_len_asymmetric(enc_n, salt, w0, w1, w2, 2);

        async fn collect_lane_data(s: &UdpSocket, expected_salt: u64, expected_b: u32, target: usize) -> Vec<u8> {
            let mut buf = [0u8; 16];
            loop {
                let (len, _) = s.recv_from(&mut buf).await.unwrap();
                if len == 16 {
                    let s_val = u64::from_be_bytes(buf[0..8].try_into().unwrap());
                    let b_val = u32::from_be_bytes(buf[8..12].try_into().unwrap());
                    if s_val == expected_salt && b_val == expected_b {
                        break; 
                    }
                }
            }
            let mut data = vec![0u8; target];
            let mut rx = 0;
            let mut temp = [0u8; 2048]; // Match 1KB chunks
            while rx < target {
                let (l, _) = s.recv_from(&mut temp).await.unwrap();
                let end = (rx + l).min(target);
                data[rx..end].copy_from_slice(&temp[0..end-rx]);
                rx = end;
            }
            data
        }

        let (d0, d1, d2) = tokio::join!(
            collect_lane_data(l1, salt, b_idx, len0),
            collect_lane_data(l2, salt, b_idx, len1),
            collect_lane_data(l3, salt, b_idx, len2),
        );

        // Reassemble block
        let mut re = vec![0u8; enc_n];
        let w_total = (w0 + w1 + w2) as u64;
        let pattern_offset = salt % w_total;
        
        // ... (math)
        let initial_idx0 = get_hits(pattern_offset, w_total, w0 as u64, 0);
        let initial_idx1 = get_hits(pattern_offset, w_total, w1 as u64, w0 as u64);
        let initial_idx2 = get_hits(pattern_offset, w_total, w2 as u64, (w0 + w1) as u64);

        for i in 0..enc_n {
            let eff = i as u64 + pattern_offset;
            let block_id = eff / w_total;
            let pos = eff % w_total;
            if pos < w0 as u64 {
                let local_idx = (block_id * w0 as u64 + pos) - initial_idx0;
                re[i] = d0[local_idx as usize];
            } else if pos < (w0 + w1) as u64 {
                let local_idx = (block_id * w1 as u64 + (pos - w0 as u64)) - initial_idx1;
                re[i] = d1[local_idx as usize];
            } else {
                let local_idx = (block_id * w2 as u64 + (pos - (w0 + w1) as u64)) - initial_idx2;
                re[i] = d2[local_idx as usize];
            }
        }
        
        let key_material = salt.to_be_bytes();
        let mut full_key = [0u8; 32];
        for k in 0..4 { full_key[k*8..(k+1)*8].copy_from_slice(&key_material); }
        let key = Key::<Aes256Gcm>::from_slice(&full_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&b_idx.to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        cipher.decrypt(nonce, re.as_ref()).expect("Block decryption failure")
    }

    for block_idx in 0..total_blocks {
        println!("📥 Collecting Block {}/{}...", block_idx + 1, total_blocks);
        let mut current_block_raw_size = if block_idx == total_blocks - 1 {
            total_expected % block_size
        } else {
            block_size
        };
        if current_block_raw_size == 0 { current_block_raw_size = block_size; }
        let current_enc_n = current_block_raw_size + 16;

        let plaintext = collect_block(&l1, &l2, &l3, salt, block_idx as u32, w0, w1, w2, current_enc_n).await;
        output_file.write_all(&plaintext)?;
    }

    println!("🎉 SUCCESS: Deep-Sea stream reassembled and saved.");
    Ok(())
}
