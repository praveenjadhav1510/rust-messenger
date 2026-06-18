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
