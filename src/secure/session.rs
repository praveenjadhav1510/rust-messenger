use crate::storage::filesystem::get_storage_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecureSession {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub peer: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub established: bool,
}

fn get_secure_sessions_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("secure_sessions.json"))
}

pub fn load_secure_sessions() -> Result<Vec<SecureSession>> {
    let path = get_secure_sessions_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let sessions: Vec<SecureSession> = serde_json::from_str(&content)?;
    Ok(sessions)
}

pub fn save_secure_sessions(sessions: &[SecureSession]) -> Result<()> {
    let path = get_secure_sessions_path()?;
    let content = serde_json::to_string_pretty(sessions)?;
    fs::write(path, content)?;
    Ok(())
}
