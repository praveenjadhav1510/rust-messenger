use crate::protocol::packet::Packet;
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};
use std::collections::VecDeque;

pub struct MockTransport {
    connected: bool,
    pub incoming: VecDeque<Packet>,
    pub outgoing: Vec<Packet>,
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            connected: false,
            incoming: VecDeque::new(),
            outgoing: Vec::new(),
        }
    }

    /// Simulate receiving a packet (adds it to the incoming queue)
    pub fn simulate_incoming(&mut self, packet: Packet) {
        self.incoming.push_back(packet);
    }

    /// Check if transport is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Transport for MockTransport {
    fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn send(&mut self, packet: Packet) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Transport not connected."));
        }
        self.outgoing.push(packet);
        Ok(())
    }

    fn receive(&mut self) -> Result<Option<Packet>> {
        if !self.connected {
            return Err(anyhow!("Transport not connected."));
        }
        Ok(self.incoming.pop_front())
    }
}
