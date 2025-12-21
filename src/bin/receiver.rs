use tokio::net::UdpSocket;
use std::env;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::Write;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

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

    println!("👻 GHOST RECEIVER v2.1 | DASHBOARD ONLINE");
    println!("🔑 SALT: {} | 📦 SIZE: {} bytes", salt, total_expected);

    let p1_port = std::env::var("LANE1_PORT").unwrap_or_else(|_| "8001".to_string());
    let p2_port = std::env::var("LANE2_PORT").unwrap_or_else(|_| "8002".to_string());
    let p3_port = std::env::var("LANE3_PORT").unwrap_or_else(|_| "8003".to_string());
    let block_size: usize = std::env::var("BLOCK_SIZE").unwrap_or_else(|_| "5242880".to_string()).parse().unwrap();

    let l1 = UdpSocket::bind(format!("0.0.0.0:{}", p1_port)).await?;
    let l2 = UdpSocket::bind(format!("0.0.0.0:{}", p2_port)).await?;
    let l3 = UdpSocket::bind(format!("0.0.0.0:{}", p3_port)).await?;

    let total_blocks = (total_expected + block_size - 1) / block_size;
    let _ = std::fs::remove_file("reborn_image.jpg");
    let mut output_file = OpenOptions::new().create(true).write(true).open("reborn_image.jpg")?;

    let multiprogress = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("{prefix:.bold}▕{bar:40.cyan/blue}▏{pos}/{len} ({percent}%)")?
        .progress_chars("█▉▊▋▌▍▎▏  ");

    let pb1 = multiprogress.add(ProgressBar::new(0));
    pb1.set_style(sty.clone());
    pb1.set_prefix("📡 2.4GHz ");

    let pb2 = multiprogress.add(ProgressBar::new(0));
    pb2.set_style(sty.clone());
    pb2.set_prefix("⚡ 5GHz-1 ");

    let pb3 = multiprogress.add(ProgressBar::new(0));
    pb3.set_style(sty);
    pb3.set_prefix("⚡ 5GHz-2 ");

    async fn collect_lane_data(s: &UdpSocket, expected_salt: u64, expected_b: u32, pb: &ProgressBar) -> (Vec<u8>, usize, i32, i32, i32) {
        let mut buf = [0u8; 1024];
        let mut target_enc_n = 0;
        let (mut w0, mut w1, mut w2) = (0, 0, 0);

        loop {
            let (len, addr) = s.recv_from(&mut buf).await.unwrap();
            if len >= 16 {
                let s_val = u64::from_be_bytes(buf[0..8].try_into().unwrap());
                let b_val = u32::from_be_bytes(buf[8..12].try_into().unwrap());
                
                if s_val == 0xFFFFFFFFFFFFFFFF {
                    let _ = s.send_to(&buf[..len], addr).await;
                    continue; 
                }

                if s_val == expected_salt && b_val == expected_b {
                    if len >= 28 {
                        target_enc_n = u32::from_be_bytes(buf[12..16].try_into().unwrap()) as usize;
                        w0 = u32::from_be_bytes(buf[16..20].try_into().unwrap()) as i32;
                        w1 = u32::from_be_bytes(buf[20..24].try_into().unwrap()) as i32;
                        w2 = u32::from_be_bytes(buf[24..28].try_into().unwrap()) as i32;
                    }
                    break; 
                }
            }
        }
        
        let lane_id = match pb.prefix().contains("2.4") { true => 0, false => if pb.prefix().contains("-1") {1} else {2} };
        let target = get_lane_len_asymmetric(target_enc_n, expected_salt, w0, w1, w2, lane_id);
        
        pb.set_length(target as u64);
        pb.set_position(0);
        let mut data = vec![0u8; target];
        let mut rx = 0;
        let mut temp = [0u8; 2048];
        while rx < target {
            let (l, _) = s.recv_from(&mut temp).await.unwrap();
            let end = (rx + l).min(target);
            data[rx..end].copy_from_slice(&temp[0..end-rx]);
            rx = end;
            pb.set_position(rx as u64);
        }
        (data, target_enc_n, w0, w1, w2)
    }

    for block_idx in 0..total_blocks {
        let (r1, r2, r3) = tokio::join!(
            collect_lane_data(&l1, salt, block_idx as u32, &pb1),
            collect_lane_data(&l2, salt, block_idx as u32, &pb2),
            collect_lane_data(&l3, salt, block_idx as u32, &pb3),
        );
        
        let (d0, current_enc_n, w0, w1, w2) = r1;
        let (d1, _, _, _, _) = r2;
        let (d2, _, _, _, _) = r3;

        let mut re = vec![0u8; current_enc_n];
        let w_total = (w0 + w1 + w2) as u64;
        let pattern_offset = salt % w_total;
        let i0 = get_hits(pattern_offset, w_total, w0 as u64, 0);
        let i1 = get_hits(pattern_offset, w_total, w1 as u64, w0 as u64);
        let i2 = get_hits(pattern_offset, w_total, w2 as u64, (w0 + w1) as u64);

        for i in 0..current_enc_n {
            let eff = i as u64 + pattern_offset;
            let block_id = eff / w_total;
            let pos = eff % w_total;
            if pos < w0 as u64 {
                let local_idx = (block_id * w0 as u64 + pos) - i0;
                re[i] = d0[local_idx as usize];
            } else if pos < (w0 + w1) as u64 {
                let local_idx = (block_id * w1 as u64 + (pos - w0 as u64)) - i1;
                re[i] = d1[local_idx as usize];
            } else {
                let local_idx = (block_id * w2 as u64 + (pos - (w0 + w1) as u64)) - i2;
                re[i] = d2[local_idx as usize];
            }
        }

        let key_material = salt.to_be_bytes();
        let mut full_key = [0u8; 32];
        for k in 0..4 { full_key[k*8..(k+1)*8].copy_from_slice(&key_material); }
        let key = Key::<Aes256Gcm>::from_slice(&full_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&(block_idx as u32).to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, re.as_ref()).expect("Block decryption failure");
        output_file.write_all(&plaintext)?;
    }

    pb1.finish_and_clear();
    pb2.finish_and_clear();
    pb3.finish_and_clear();
    println!("🎉 SUCCESS: Deep-Sea stream reassembled and saved.");
    Ok(())
}
