use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckUsernameResponse {
    pub available: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct RegisterRequest {
    pub username: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RegisterResponse {
    pub success: bool,
    pub username: String,
    pub fingerprint: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub username: String,
    pub fingerprint: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserLookupResponse {
    pub username: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub fingerprint: String,
    #[serde(alias = "accountStatus", alias = "status")]
    pub status: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    pub error: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RecoverRequest {
    pub username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
    #[serde(rename = "newPublicKey")]
    pub new_public_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RecoverResponse {
    pub success: bool,
    pub fingerprint: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RenameRequest {
    #[serde(rename = "currentUsername")]
    pub current_username: String,
    #[serde(rename = "newUsername")]
    pub new_username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RenameResponse {
    pub success: bool,
    pub username: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RemoveRequest {
    pub username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RemoveResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RestoreRequest {
    pub username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
    #[serde(rename = "newPublicKey")]
    pub new_public_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RestoreResponse {
    pub success: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct LockRequest {
    pub username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LockResponse {
    pub success: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct UnlockRequest {
    pub username: String,
    #[serde(rename = "recoveryCode")]
    pub recovery_code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UnlockResponse {
    pub success: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct HeartbeatRequest {
    pub username: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HeartbeatResponse {
    pub success: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PresenceResponse {
    pub username: String,
    pub online: bool,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "lastSeen")]
    pub last_seen: Option<String>,
    #[serde(rename = "clientVersion")]
    pub client_version: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct OfflineRequest {
    pub username: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OfflineResponse {
    pub success: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CandidatesResponse {
    pub username: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub candidates: Vec<crate::network::candidate::IceCandidate>,
}
