use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresenceInfo {
    pub username: String,
    pub online: bool,
    #[serde(rename = "lastSeen")]
    pub last_seen: Option<String>,
    #[serde(rename = "clientVersion")]
    pub client_version: Option<String>,
}
