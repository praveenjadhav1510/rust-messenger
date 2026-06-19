use crate::connection::state::ConnectionState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransportType {
    Mock,
    Tcp,
    Ice,
    Turn,
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Mock => write!(f, "MOCK"),
            TransportType::Tcp => write!(f, "TCP"),
            TransportType::Ice => write!(f, "ICE"),
            TransportType::Turn => write!(f, "TURN"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PeerSession {
    pub username: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub state: ConnectionState,
    #[serde(rename = "connectedAt")]
    pub connected_at: String,
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
    pub transport: TransportType,
}
