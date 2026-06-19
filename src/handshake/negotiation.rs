use crate::handshake::answer::ConnectionAnswer;
use crate::handshake::offer::ConnectionOffer;
use crate::storage::filesystem::get_storage_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HandshakeSession {
    pub offer: ConnectionOffer,
    pub answer: Option<ConnectionAnswer>,
    pub status: String, // "PENDING", "ACCEPTED", "REJECTED"
}

fn get_handshakes_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("handshakes.json"))
}

pub fn load_handshakes() -> Result<Vec<HandshakeSession>> {
    let path = get_handshakes_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let sessions: Vec<HandshakeSession> = serde_json::from_str(&content)?;
    Ok(sessions)
}

pub fn save_handshakes(sessions: &[HandshakeSession]) -> Result<()> {
    let path = get_handshakes_path()?;
    let content = serde_json::to_string_pretty(sessions)?;
    fs::write(path, content)?;
    Ok(())
}
