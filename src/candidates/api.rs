use crate::network::candidate::IceCandidate;
use crate::registry::api::RegistryClient;
use crate::registry::models::CandidatesResponse;
use crate::storage::filesystem::read_profile;
use anyhow::{Result, anyhow};

pub async fn publish_candidates(
    username: &str,
    session_id: &str,
    candidates: &[IceCandidate],
) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let resp = client
        .publish_candidates(username, session_id, candidates)
        .await
        .map_err(|e| anyhow!("Registry unavailable: {}", e))?;
    if !resp.success {
        return Err(anyhow!("Registry rejected publish candidates."));
    }
    Ok(())
}

pub async fn delete_candidates(username: &str) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let success = client
        .delete_candidates(username)
        .await
        .map_err(|_| anyhow!("Registry unavailable."))?;
    if !success {
        return Err(anyhow!("Registry unavailable."));
    }
    Ok(())
}

pub async fn fetch_candidates(username: &str) -> Result<CandidatesResponse> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let resp = client.get_candidates(username).await.map_err(|e| {
        if e.to_string() == "User not found." {
            e
        } else {
            anyhow!("Registry unavailable.")
        }
    })?;
    Ok(resp)
}
