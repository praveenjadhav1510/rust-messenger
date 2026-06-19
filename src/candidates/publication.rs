use crate::network::candidate::IceCandidate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePublication {
    pub username: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub candidates: Vec<IceCandidate>,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}
