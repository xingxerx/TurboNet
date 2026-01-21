use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target = "127.0.0.1:9999";

    // Malicious payload
    let payload = b"AND 1=1; DROP TABLE users; --";

    println!("⚔️ ATTACKER: Casting malicious spell on {}...", target);

    for i in 0..5 {
        socket.send_to(payload, target)?;
        println!("   Sent packet {}/5", i + 1);
        thread::sleep(Duration::from_millis(500));
    }

    println!("✅ Attack complete.");
    Ok(())
}
