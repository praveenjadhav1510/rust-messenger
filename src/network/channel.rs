use crate::network::reliability::ReliabilityLayer;
use crate::network::udp::UdpTransport;
use std::net::SocketAddr;

pub struct SecureChannel {
    pub transport: UdpTransport,
    pub reliability: ReliabilityLayer,
    pub shared_secret: Option<[u8; 32]>,
}

impl SecureChannel {
    pub fn new(local_port: u16, remote_addr: SocketAddr) -> Self {
        Self {
            transport: UdpTransport::new(local_port, Some(remote_addr)),
            reliability: ReliabilityLayer::new(),
            shared_secret: None,
        }
    }
}
