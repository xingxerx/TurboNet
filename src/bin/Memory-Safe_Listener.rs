// Design Choice: Using standard library networking for high-level logic, 
// but we will move to raw sockets if manual header manipulation is required.
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    // Bind to the virtual bridge interface
    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    println!("TurboNet node active on port 8080...");

    let mut buf = [0u8; 1024]; 

    loop {
        // Ownership logic: recv_from yields a result containing size and origin
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let data = &buf[..amt];
                println!("Received {} bytes from {}: {:?}", amt, src, data);
                
                // Philosophy: Simple echo for connection verification
                socket.send_to(data, &src)?;
            }
            Err(e) => eprintln!("Socket error: {}", e),
        }
    }
}