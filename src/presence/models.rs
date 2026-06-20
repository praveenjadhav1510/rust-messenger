use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresenceInfo {
    pub username: String,
    pub online: bool,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "lastSeen")]
    pub last_seen: Option<String>,
    #[serde(rename = "clientVersion")]
    pub client_version: Option<String>,
}
