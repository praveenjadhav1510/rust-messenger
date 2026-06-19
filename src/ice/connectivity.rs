use crate::ice::checks::validate_pair;
use crate::ice::pair::CandidatePair;
use crate::ice::state::IceConnectionState;
use crate::network::candidate::{CandidateType, IceCandidate};
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IceSession {
    pub peer: String,
    pub selected_pair: CandidatePair,
    pub established_at: String,
}

fn get_ice_sessions_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("ice_sessions.json"))
}

pub fn load_ice_sessions() -> Result<Vec<IceSession>> {
    let path = get_ice_sessions_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let sessions: Vec<IceSession> = serde_json::from_str(&content)?;
    Ok(sessions)
}

pub fn save_ice_sessions(sessions: &[IceSession]) -> Result<()> {
    let path = get_ice_sessions_path()?;
    let content = serde_json::to_string_pretty(sessions)?;
    fs::write(path, content)?;
    Ok(())
}

pub struct ConnectivityManager;

impl ConnectivityManager {
    pub fn build_candidate_pairs(
        local: &[IceCandidate],
        remote: &[IceCandidate],
    ) -> Vec<CandidatePair> {
        let mut pairs = Vec::new();
        for l in local {
            for r in remote {
                let priority = match (l.candidate_type, r.candidate_type) {
                    (CandidateType::Host, CandidateType::Host) => 100,
                    (CandidateType::Host, CandidateType::ServerReflexive)
                    | (CandidateType::ServerReflexive, CandidateType::Host) => 90,
                    (CandidateType::ServerReflexive, CandidateType::ServerReflexive) => 80,
                    _ => 50,
                };
                pairs.push(CandidatePair::new(l.clone(), r.clone(), priority));
            }
        }
        pairs
    }

    pub fn prioritize_pairs(pairs: &mut [CandidatePair]) {
        pairs.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub async fn run_checks(
        pairs: &mut [CandidatePair],
        sender: &str,
        recipient: &str,
    ) -> Result<()> {
        for pair in pairs {
            let _state = Self::validate_pair(pair, sender, recipient).await?;
        }
        Ok(())
    }

    pub async fn validate_pair(
        pair: &mut CandidatePair,
        sender: &str,
        recipient: &str,
    ) -> Result<IceConnectionState> {
        validate_pair(pair, sender, recipient).await
    }

    pub fn select_working_pair(pairs: &[CandidatePair]) -> Option<CandidatePair> {
        pairs
            .iter()
            .find(|p| p.state == IceConnectionState::Connected)
            .cloned()
    }
}
