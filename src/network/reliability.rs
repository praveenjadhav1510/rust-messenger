use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReliablePacket {
    pub sequence: u64,
    pub ack: u64,
    pub payload: serde_json::Value,
}

pub struct ReliabilityLayer {
    next_seq: u64,
    highest_ack: u64,
    pending: HashMap<u64, ReliablePacket>,
}

impl Default for ReliabilityLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReliabilityLayer {
    pub fn new() -> Self {
        Self {
            next_seq: 1,
            highest_ack: 0,
            pending: HashMap::new(),
        }
    }

    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        seq
    }

    pub fn ack_packet(&mut self, seq: u64) {
        self.pending.remove(&seq);
        if seq > self.highest_ack {
            self.highest_ack = seq;
        }
    }

    pub fn track_pending(&mut self, packet: ReliablePacket) {
        self.pending.insert(packet.sequence, packet);
    }
}
