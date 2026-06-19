use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PacketType {
    Message,
    MessageRequest,
    Ack,
    ReadReceipt,
    Ping,
    Presence,
    FileMetadata,
    FileChunk,
    ConnectivityCheck,
    ConnectivityResponse,
    Keepalive,
    PunchProbe,
    PunchAck,
    MessageAck,
    MessageRead,
}

impl std::fmt::Display for PacketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketType::Message => write!(f, "MESSAGE"),
            PacketType::MessageRequest => write!(f, "MESSAGE_REQUEST"),
            PacketType::Ack => write!(f, "ACK"),
            PacketType::ReadReceipt => write!(f, "READ_RECEIPT"),
            PacketType::Ping => write!(f, "PING"),
            PacketType::Presence => write!(f, "PRESENCE"),
            PacketType::FileMetadata => write!(f, "FILE_METADATA"),
            PacketType::FileChunk => write!(f, "FILE_CHUNK"),
            PacketType::ConnectivityCheck => write!(f, "CONNECTIVITY_CHECK"),
            PacketType::ConnectivityResponse => write!(f, "CONNECTIVITY_RESPONSE"),
            PacketType::Keepalive => write!(f, "KEEPALIVE"),
            PacketType::PunchProbe => write!(f, "PUNCH_PROBE"),
            PacketType::PunchAck => write!(f, "PUNCH_ACK"),
            PacketType::MessageAck => write!(f, "MESSAGE_ACK"),
            PacketType::MessageRead => write!(f, "MESSAGE_READ"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Packet {
    pub version: u32,
    pub packet_type: PacketType,
    pub message_id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,
    pub encrypted_payload: String,
    pub signature: String,
}

impl Packet {
    /// Encodes the packet as a JSON string
    pub fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }

    /// Decodes a packet from a JSON string
    pub fn decode(json: &str) -> Result<Self> {
        let packet: Self = serde_json::from_str(json)?;
        Ok(packet)
    }

    /// Validates the structure and core constraints of the packet
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(anyhow!("Unsupported packet version: {}", self.version));
        }
        if self.sender.trim().is_empty() {
            return Err(anyhow!("Sender username cannot be empty."));
        }
        if self.recipient.trim().is_empty() {
            return Err(anyhow!("Recipient username cannot be empty."));
        }
        if self.message_id.is_nil() {
            return Err(anyhow!("Message ID cannot be nil (all zeros)."));
        }
        if self.signature.trim().is_empty() {
            return Err(anyhow!("Packet signature cannot be empty."));
        }
        Ok(())
    }
}
