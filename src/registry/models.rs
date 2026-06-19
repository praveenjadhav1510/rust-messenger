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
