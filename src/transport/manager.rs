use crate::protocol::packet::Packet;
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};

pub struct TransportManager {
    transport: Option<Box<dyn Transport + Send + Sync>>,
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportManager {
    pub fn new() -> Self {
        Self { transport: None }
    }

    pub fn register_transport(&mut self, transport: Box<dyn Transport + Send + Sync>) {
        self.transport = Some(transport);
    }

    pub fn get_transport(&mut self) -> Result<&mut (dyn Transport + Send + Sync)> {
        match &mut self.transport {
            Some(t) => Ok(t.as_mut()),
            None => Err(anyhow!("No transport registered.")),
        }
    }

    pub fn send_packet(&mut self, packet: Packet) -> Result<()> {
        let t = self.get_transport()?;
        t.send(packet)
    }

    pub fn receive_packet(&mut self) -> Result<Option<Packet>> {
        let t = self.get_transport()?;
        t.receive()
    }
}
