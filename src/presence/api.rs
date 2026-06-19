use crate::presence::models::PresenceInfo;
use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::{Result, anyhow};

pub async fn fetch_presence(username: &str) -> Result<PresenceInfo> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client.get_presence(username).await.map_err(|e| {
        if e.to_string() == "User not found." {
            e
        } else {
            anyhow!("Registry unavailable.")
        }
    })?;

    Ok(PresenceInfo {
        username: response.username,
        online: response.online,
        last_seen: response.last_seen,
        client_version: response.client_version,
    })
}

pub async fn announce_online(username: &str, session_id: &str, client_version: &str) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client
        .send_heartbeat(username, session_id, client_version)
        .await
        .map_err(|_| anyhow!("Registry unavailable."))?;
    if !response.success {
        return Err(anyhow!("Registry unavailable."));
    }
    Ok(())
}

pub async fn announce_offline(username: &str, session_id: &str) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client
        .set_offline(username, session_id)
        .await
        .map_err(|_| anyhow!("Registry unavailable."))?;
    if !response.success {
        return Err(anyhow!("Registry unavailable."));
    }
    Ok(())
}
