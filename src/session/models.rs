use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub username: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    pub online: bool,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
}
