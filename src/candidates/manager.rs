use crate::candidates::api::{delete_candidates, fetch_candidates, publish_candidates};
use crate::candidates::publication::CandidatePublication;
use crate::network::candidate::IceCandidate;
use crate::network::discovery::DiscoveryManager;
use crate::registry::models::CandidatesResponse;
use crate::session::manager::load_session;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use chrono::{Duration, Utc};
use std::fs;
use std::path::PathBuf;

pub struct CandidateManager;

fn get_publication_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("candidate_publication.json"))
}

impl CandidateManager {
    pub fn save_publication(publ: &CandidatePublication) -> Result<()> {
        let path = get_publication_path()?;
        let content = serde_json::to_string_pretty(publ)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_publication() -> Result<CandidatePublication> {
        let path = get_publication_path()?;
        if !path.exists() {
            return Err(anyhow!("No candidate publication found."));
        }
        let content = fs::read_to_string(path)?;
        let publ: CandidatePublication = serde_json::from_str(&content)?;
        Ok(publ)
    }

    pub async fn publish() -> Result<CandidatePublication> {
        let session = load_session()?;
        if !session.online {
            return Err(anyhow!("Cannot publish candidates: session is offline."));
        }

        let candidates = Self::load_local()?;
        if candidates.is_empty() {
            return Err(anyhow!(
                "No local candidates found. Run 'rust-messenger netinfo' first."
            ));
        }

        publish_candidates(&session.username, &session.session_id, &candidates).await?;

        let now = Utc::now();
        let expires = now + Duration::seconds(60);

        let publ = CandidatePublication {
            username: session.username.clone(),
            session_id: session.session_id.clone(),
            candidates,
            published_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
        };

        Self::save_publication(&publ)?;
        Ok(publ)
    }

    pub async fn refresh() -> Result<CandidatePublication> {
        let session = load_session()?;
        if !session.online {
            return Err(anyhow!("Cannot refresh candidates: session is offline."));
        }

        // Regenerate candidates
        let _net_info = DiscoveryManager::discover().await?;
        let candidates = Self::load_local()?;

        publish_candidates(&session.username, &session.session_id, &candidates).await?;

        let now = Utc::now();
        let expires = now + Duration::seconds(60);

        let publ = CandidatePublication {
            username: session.username.clone(),
            session_id: session.session_id.clone(),
            candidates,
            published_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
        };

        Self::save_publication(&publ)?;
        Ok(publ)
    }

    pub async fn remove() -> Result<()> {
        let session = load_session()?;
        // Delete from registry
        delete_candidates(&session.username).await?;

        // Delete local publication metadata
        let path = get_publication_path()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub async fn fetch_remote(username: &str) -> Result<CandidatesResponse> {
        fetch_candidates(username).await
    }

    pub fn load_local() -> Result<Vec<IceCandidate>> {
        DiscoveryManager::load_candidates()
    }
}
