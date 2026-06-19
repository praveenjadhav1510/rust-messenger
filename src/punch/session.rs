use crate::ice::pair::CandidatePair;
use crate::punch::state::PunchState;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PunchSession {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub peer: String,
    #[serde(rename = "selectedPair")]
    pub selected_pair: CandidatePair,
    pub state: PunchState,
    pub attempts: usize,
    #[serde(rename = "startedAt")]
    pub started_at: String,
}

fn get_punch_sessions_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("punch_sessions.json"))
}

pub fn load_punch_sessions() -> Result<Vec<PunchSession>> {
    let path = get_punch_sessions_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let sessions: Vec<PunchSession> = serde_json::from_str(&content)?;
    Ok(sessions)
}

pub fn save_punch_sessions(sessions: &[PunchSession]) -> Result<()> {
    let path = get_punch_sessions_path()?;
    let content = serde_json::to_string_pretty(sessions)?;
    fs::write(path, content)?;
    Ok(())
}
