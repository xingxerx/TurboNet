// TurboNet Packets-Per-Second Benchmark
// Measures baseline PPS to validate optimization impact

#[cfg(feature = "benchmark")]
use criterion::{criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};

#[cfg(feature = "benchmark")]
fn pps_send_benchmark(c: &mut Criterion) {
    use std::net::UdpSocket;
    
    let mut group = c.benchmark_group("udp_send");
    
    // Test different packet sizes
    for size in [64, 512, 1500, 8192, 60000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
            let target = "127.0.0.1:9999";
            let buf = vec![0u8; size];
            
            b.iter(|| {
                let _ = socket.send_to(&buf, target);
            });
        });
    }
    group.finish();
}

#[cfg(feature = "benchmark")]
fn pps_batch_benchmark(c: &mut Criterion) {
    use std::net::UdpSocket;
    
    let mut group = c.benchmark_group("udp_batch");
    group.throughput(Throughput::Elements(100)); // 100 packets per iteration
    
    group.bench_function("batch_100_1500b", |b| {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let target: std::net::SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let buf = vec![0u8; 1500];
        
        b.iter(|| {
            for _ in 0..100 {
                let _ = socket.send_to(&buf, target);
            }
        });
    });
    group.finish();
}

#[cfg(feature = "benchmark")]
criterion_group!(benches, pps_send_benchmark, pps_batch_benchmark);

#[cfg(feature = "benchmark")]
criterion_main!(benches);

// Placeholder for non-benchmark builds
#[cfg(not(feature = "benchmark"))]
fn main() {
    eprintln!("Run with: cargo bench --features benchmark");
}
