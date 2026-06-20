use crate::network::candidate::{CandidateType, IceCandidate};
use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub async fn fetch_remote_candidates(username: &str) -> Result<Vec<IceCandidate>> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url);

    let res = client.get_candidates(username).await?;
    Ok(res.candidates)
}
