use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::Write;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pqc_kyber::*;
use std::sync::Arc;

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
    let total_expected_arg: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    println!("👻 GHOST RECEIVER v4.2 | SOLID-STATE ACK ONLINE");

    let mut rng = rand::thread_rng();
    let keys = keypair(&mut rng).map_err(|e| format!("Kyber Error: {:?}", e))?;
    println!("⚛️ LATTICE: Post-Quantum Kyber-768 keypair generated.");

    let p1_port = env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string());
    let p2_port = env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string());
    let p3_port = env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string());
    let block_size: usize = env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let l1 = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", p1_port)).await?);
    let l2 = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", p2_port)).await?);
    let l3 = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", p3_port)).await?);

    println!("📡 HANDSHAKE: Waiting for Shredder to request Lattice Public Key...");
    let mut buf = [0u8; 1024];
    let shredder_addr;
    loop {
        let (len, addr) = l1.recv_from(&mut buf).await?;
        if &buf[..len] == b"PK_REQ" {
            println!("🤝 HANDSHAKE: Shredder found at {}. Sending Public Key...", addr);
            l1.send_to(&keys.public, addr).await?;
            shredder_addr = addr;
            break;
        }
    }

    println!("⚛️ LATTICE: Capturing Quantum-Mesh ciphertext fragments...");
    let ct_len = KYBER_CIPHERTEXTBYTES;
    let mut ct_full = vec![0u8; ct_len];

    let (tx1, mut rx1) = tokio::sync::mpsc::channel(1);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(1);
    let (tx3, mut rx3) = tokio::sync::mpsc::channel(1);

    let l1_h = l1.clone();
    let l2_h = l2.clone();
    let l3_h = l3.clone();

    tokio::spawn(async move {
        let mut b = [0u8; 1024];
        let (n, _) = l1_h.recv_from(&mut b).await.unwrap();
        tx1.send(b[..n].to_vec()).await.unwrap();
    });
    tokio::spawn(async move {
        let mut b = [0u8; 1024];
        let (n, _) = l2_h.recv_from(&mut b).await.unwrap();
        tx2.send(b[..n].to_vec()).await.unwrap();
    });
    tokio::spawn(async move {
        let mut b = [0u8; 1024];
        let (n, _) = l3_h.recv_from(&mut b).await.unwrap();
        tx3.send(b[..n].to_vec()).await.unwrap();
    });

    let f1 = rx1.recv().await.unwrap();
    let f2 = rx2.recv().await.unwrap();
    let f3 = rx3.recv().await.unwrap();

    ct_full[0..f1.len()].copy_from_slice(&f1);
    ct_full[f1.len()..f1.len()+f2.len()].copy_from_slice(&f2);
    ct_full[f1.len()+f2.len()..ct_len].copy_from_slice(&f3);

    let decoded_secret = decapsulate(&ct_full, &keys.secret).map_err(|_| "Decapsulation failed")?;
    let mut shared_secret = [0u8; 32];
    shared_secret.copy_from_slice(&decoded_secret);
    let quantum_salt = u64::from_be_bytes(shared_secret[0..8].try_into().unwrap());
    println!("✅ SUCCESS: Quantum Handshake complete. Shared Secret derived.");

    println!("📦 LATTICE: Waiting for Payload Metadata...");
    let mut m_buf = [0u8; 2048];
    let (_, _) = l1.recv_from(&mut m_buf).await?;
    let (filename, total_expected) = if m_buf[0] == b'M' {
        let f_len = u32::from_be_bytes(m_buf[1..5].try_into().unwrap()) as usize;
        let name = String::from_utf8_lossy(&m_buf[5..5+f_len]).to_string();
        let size = u64::from_be_bytes(m_buf[5+f_len..5+f_len+8].try_into().unwrap()) as usize;
        (name, size)
    } else {
        ("unknown_payload.bin".to_string(), total_expected_arg)
    };

    println!("🎯 TARGET IDENTIFIED: {} ({} bytes)", filename, total_expected);
    
    let total_blocks = (total_expected + block_size - 1) / block_size;
    let out_name = format!("reborn_{}", filename);
    let _ = std::fs::remove_file(&out_name);
    let mut output_file = OpenOptions::new().create(true).write(true).open(&out_name)?;

    let multiprogress = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("{prefix:.bold}▕{bar:40.cyan/blue}▏{pos}/{len} ({percent}%)")?
        .progress_chars("█▉▊▋▌▍▎▏  ");

    let pb1 = multiprogress.add(ProgressBar::new(0)); pb1.set_style(sty.clone()); pb1.set_prefix("📡 2.4GHz ");
    let pb2 = multiprogress.add(ProgressBar::new(0)); pb2.set_style(sty.clone()); pb2.set_prefix("⚡ 5GHz-1 ");
    let pb3 = multiprogress.add(ProgressBar::new(0)); pb3.set_style(sty); pb3.set_prefix("⚡ 5GHz-2 ");

    async fn collect_lane_data(s: &UdpSocket, expected_salt: u64, expected_b: u32, pb: &ProgressBar) -> Option<(Vec<u8>, usize, i32, i32, i32)> {
        let mut buf = [0u8; 2048];
        let mut target_enc_n = 0;
        let (mut w0, mut w1, mut w2) = (0, 0, 0);

        // Header Phase
        let header_res = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let (len, addr) = s.recv_from(&mut buf).await.unwrap();
                if len >= 16 {
                    let s_val = u64::from_be_bytes(buf[0..8].try_into().unwrap());
                    let b_val = u32::from_be_bytes(buf[8..12].try_into().unwrap());
                    if s_val == 0xFFFFFFFFFFFFFFFF { let _ = s.send_to(&buf[..len], addr).await; continue; }
                    if s_val == expected_salt && b_val == expected_b {
                        if len >= 28 {
                            target_enc_n = u32::from_be_bytes(buf[12..16].try_into().unwrap()) as usize;
                            w0 = u32::from_be_bytes(buf[16..20].try_into().unwrap()) as i32;
                            w1 = u32::from_be_bytes(buf[20..24].try_into().unwrap()) as i32;
                            w2 = u32::from_be_bytes(buf[24..28].try_into().unwrap()) as i32;
                        }
                        return Some((target_enc_n, w0, w1, w2));
                    }
                }
            }
        }).await;

        let (target_enc_n, w0, w1, w2) = match header_res {
            Ok(Some(h)) => h,
            _ => return None,
        };
        
        let lane_id = if pb.prefix().contains("2.4") { 0 } else if pb.prefix().contains("-1") { 1 } else { 2 };
        let target = get_lane_len_asymmetric(target_enc_n, expected_salt, w0, w1, w2, lane_id);
        
        pb.set_length(target as u64);
        pb.set_position(0);
        let mut data = vec![0u8; target];
        let mut rx = 0;
        let mut temp = [0u8; 2048];

        let data_res = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while rx < target {
                let (l, _) = s.recv_from(&mut temp).await.unwrap();
                let end = (rx + l).min(target);
                data[rx..end].copy_from_slice(&temp[0..end-rx]);
                rx = end;
                pb.set_position(rx as u64);
            }
        }).await;

        if data_res.is_err() { return None; }
        Some((data, target_enc_n, w0, w1, w2))
    }

    for block_idx in 0..total_blocks {
        loop {
            let res = tokio::join!(
                collect_lane_data(&l1, quantum_salt, block_idx as u32, &pb1),
                collect_lane_data(&l2, quantum_salt, block_idx as u32, &pb2),
                collect_lane_data(&l3, quantum_salt, block_idx as u32, &pb3),
            );

            if let (Some(r1), Some(r2), Some(r3)) = res {
                let (d0, current_enc_n, w0, w1, w2) = r1;
                let (d1, _, _, _, _) = r2;
                let (d2, _, _, _, _) = r3;

                let mut re = vec![0u8; current_enc_n];
                let w_total = (w0 + w1 + w2) as u64;
                let pattern_offset = quantum_salt % w_total;
                let i0 = get_hits(pattern_offset, w_total, w0 as u64, 0);
                let i1 = get_hits(pattern_offset, w_total, w1 as u64, w0 as u64);
                let i2 = get_hits(pattern_offset, w_total, w2 as u64, (w0 + w1) as u64);

                for i in 0..current_enc_n {
                    let eff = i as u64 + pattern_offset;
                    let block_id = eff / w_total;
                    let pos = eff % w_total;
                    if pos < w0 as u64 {
                        let local_idx = (block_id * w0 as u64 + pos).checked_sub(i0).unwrap_or(0);
                        if (local_idx as usize) < d0.len() { re[i] = d0[local_idx as usize]; }
                    } else if pos < (w0 + w1) as u64 {
                        let local_idx = (block_id * w1 as u64 + (pos - w0 as u64)).checked_sub(i1).unwrap_or(0);
                        if (local_idx as usize) < d1.len() { re[i] = d1[local_idx as usize]; }
                    } else {
                        let local_idx = (block_id * w2 as u64 + (pos - (w0 + w1) as u64)).checked_sub(i2).unwrap_or(0);
                        if (local_idx as usize) < d2.len() { re[i] = d2[local_idx as usize]; }
                    }
                }

                let key = Key::<Aes256Gcm>::from_slice(&shared_secret);
                let cipher = Aes256Gcm::new(key);
                let mut nonce_bytes = [0u8; 12];
                nonce_bytes[0..4].copy_from_slice(&(block_idx as u32).to_be_bytes());
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                if let Ok(plaintext) = cipher.decrypt(nonce, re.as_ref()) {
                    output_file.write_all(&plaintext)?;
                    let ack = format!("ACK:{}", block_idx);
                    for _ in 0..3 { let _ = l1.send_to(ack.as_bytes(), shredder_addr).await; }
                    break; // Move to next block
                }
            }
            // Failed block: Send NACK and loop back
            println!("🔄 BLOCK {} FAILED. Requesting re-transmission...", block_idx);
            let nack = format!("NACK:{}", block_idx);
            for _ in 0..3 { let _ = l1.send_to(nack.as_bytes(), shredder_addr).await; }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    pb1.finish_and_clear(); pb2.finish_and_clear(); pb3.finish_and_clear();
    println!("🎉 SUCCESS: Post-Quantum stream reassembled and saved.");
    Ok(())
}
