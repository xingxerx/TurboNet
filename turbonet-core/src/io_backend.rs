// TurboNet IO Backend Abstraction Layer
// Provides pluggable socket implementations for different performance tiers

use std::future::Future;
use std::io::Result;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::UdpSocket;
use socket2::{Socket, Domain, Type};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Trait for high-performance socket operations
/// Implementations can be swapped between standard tokio, io_uring, or DPDK
pub trait TurboSocket: Send + Sync {
    /// Send data to a target address
    fn send_to<'a>(
        &'a self,
        buf: &'a [u8],
        target: &'a str,
    ) -> BoxFuture<'a, usize>;

    /// Receive data and return source address
    fn recv_from<'a>(
        &'a self,
        buf: &'a mut [u8],
    ) -> BoxFuture<'a, (usize, SocketAddr)>;

    /// Batch send multiple packets (for high-throughput scenarios)
    fn batch_send<'a>(
        &'a self,
        packets: &'a [(&'a [u8], SocketAddr)],
    ) -> BoxFuture<'a, usize>;

    /// Get the local address this socket is bound to
    fn local_addr(&self) -> Result<SocketAddr>;
}

/// Standard Tokio UDP socket implementation
/// Uses socket2 for buffer tuning, tokio for async I/O
pub struct TokioUdpBackend {
    socket: Arc<UdpSocket>,
}

impl TokioUdpBackend {
    /// Create a new backend bound to the specified address
    /// Automatically tunes socket buffers for high throughput
    pub async fn bind(addr: &str) -> Result<Self> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        sock.set_reuse_address(true)?;
        
        // Tune for high throughput: 4MB buffers
        sock.set_recv_buffer_size(4 * 1024 * 1024)?;
        sock.set_send_buffer_size(4 * 1024 * 1024)?;
        sock.set_nonblocking(true)?;
        
        sock.bind(&addr.parse::<SocketAddr>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
            .into())?;
        
        let socket = Arc::new(UdpSocket::from_std(sock.into())?);
        Ok(Self { socket })
    }

    /// Create from an existing tokio UdpSocket
    pub fn from_socket(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }
}

impl TurboSocket for TokioUdpBackend {
    fn send_to<'a>(
        &'a self,
        buf: &'a [u8],
        target: &'a str,
    ) -> BoxFuture<'a, usize> {
        Box::pin(async move {
            self.socket.send_to(buf, target).await
        })
    }

    fn recv_from<'a>(
        &'a self,
        buf: &'a mut [u8],
    ) -> BoxFuture<'a, (usize, SocketAddr)> {
        Box::pin(async move {
            self.socket.recv_from(buf).await
        })
    }

    fn batch_send<'a>(
        &'a self,
        packets: &'a [(&'a [u8], SocketAddr)],
    ) -> BoxFuture<'a, usize> {
        Box::pin(async move {
            let mut total = 0;
            for (buf, addr) in packets {
                total += self.socket.send_to(buf, addr).await?;
            }
            Ok(total)
        })
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

/// Factory function to create the best available backend
/// In the future, this will auto-detect io_uring availability on Linux
pub async fn create_optimal_backend(bind_addr: &str) -> Result<Box<dyn TurboSocket>> {
    // TODO: Add io_uring detection for Linux
    // #[cfg(all(target_os = "linux", feature = "uring"))]
    // if io_uring_available() {
    //     return Ok(Box::new(IoUringBackend::bind(bind_addr).await?));
    // }
    
    Ok(Box::new(TokioUdpBackend::bind(bind_addr).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tokio_backend_bind() {
        let backend = TokioUdpBackend::bind("127.0.0.1:0").await.unwrap();
        let addr = backend.local_addr().unwrap();
        assert!(addr.port() > 0);
    }
}
