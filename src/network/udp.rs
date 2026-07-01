use crate::protocol::packet::Packet;
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub struct UdpTransport {
    socket: Option<UdpSocket>,
    local_port: u16,
    remote_addr: Option<SocketAddr>,
}

impl UdpTransport {
    pub fn new(local_port: u16, remote_addr: Option<SocketAddr>) -> Self {
        Self {
            socket: None,
            local_port,
            remote_addr,
        }
    }

    pub fn bind(&mut self, port: u16) -> Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        self.socket = Some(socket);
        self.local_port = port;
        Ok(())
    }

    pub fn send_to(&mut self, packet: Packet, addr: SocketAddr) -> Result<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| anyhow!("Socket not bound."))?;
        let payload = packet.encode()?;
        socket.send_to(payload.as_bytes(), addr)?;
        Ok(())
    }

    pub fn receive_from(&mut self) -> Result<(Packet, SocketAddr)> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| anyhow!("Socket not bound."))?;
        let mut buf = vec![0u8; 65535];
        let (len, addr) = socket.recv_from(&mut buf)?;
        let payload_str = std::str::from_utf8(&buf[..len])?;
        let packet = Packet::decode(payload_str)?;
        Ok((packet, addr))
    }

    pub fn close(&mut self) -> Result<()> {
        self.socket = None;
        Ok(())
    }
}

impl Transport for UdpTransport {
    fn connect(&mut self) -> Result<()> {
        if self.socket.is_none() {
            self.bind(self.local_port)?;
        }
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.close()
    }

    fn send(&mut self, packet: Packet) -> Result<()> {
        let remote_addr = self
            .remote_addr
            .ok_or_else(|| anyhow!("No remote address configured for transport."))?;
        self.send_to(packet, remote_addr)
    }

    fn receive(&mut self) -> Result<Option<Packet>> {
        match self.receive_from() {
            Ok((packet, _addr)) => Ok(Some(packet)),
            Err(e) => Err(e),
        }
    }
}
