use crate::protocol::packet::{Packet, PacketType};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PunchProbePayload {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub sender: String,
    pub timestamp: String,
}

pub fn build_probe_packet(session_id: &str, sender: &str, recipient: &str) -> Result<Packet> {
    let payload = PunchProbePayload {
        session_id: session_id.to_string(),
        sender: sender.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    let payload_str = serde_json::to_string(&payload)?;

    Ok(Packet {
        version: 1,
        packet_type: PacketType::PunchProbe,
        message_id: Uuid::new_v4(),
        sender: sender.to_string(),
        recipient: recipient.to_string(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: payload_str,
        signature: "punch-probe".to_string(),
    })
}

pub fn build_ack_packet(session_id: &str, sender: &str, recipient: &str) -> Result<Packet> {
    let payload = PunchProbePayload {
        session_id: session_id.to_string(),
        sender: sender.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    let payload_str = serde_json::to_string(&payload)?;

    Ok(Packet {
        version: 1,
        packet_type: PacketType::PunchAck,
        message_id: Uuid::new_v4(),
        sender: sender.to_string(),
        recipient: recipient.to_string(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: payload_str,
        signature: "punch-ack".to_string(),
    })
}
