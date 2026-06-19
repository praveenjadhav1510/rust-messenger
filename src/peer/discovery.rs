use crate::network::candidate::IceCandidate;
use crate::peer::candidate_exchange::fetch_remote_candidates;
use crate::peer::capabilities::PeerCapabilities;
use crate::presence::manager::get_user_status;
use anyhow::Result;

pub struct DiscoveredPeer {
    pub username: String,
    pub online: bool,
    pub capabilities: PeerCapabilities,
    pub candidates: Vec<IceCandidate>,
}

pub struct PeerDiscoveryManager;

impl PeerDiscoveryManager {
    pub async fn discover_peer(username: &str) -> Result<DiscoveredPeer> {
        let presence = match get_user_status(username).await {
            Ok(p) => p,
            Err(_) => crate::presence::models::PresenceInfo {
                username: username.to_string(),
                online: true,
                last_seen: Some(chrono::Utc::now().to_rfc3339()),
                client_version: Some("0.5.0".to_string()),
            },
        };

        let caps = PeerCapabilities::default();
        let cands = fetch_remote_candidates(username).await?;

        Ok(DiscoveredPeer {
            username: username.to_string(),
            online: presence.online,
            capabilities: caps,
            candidates: cands,
        })
    }

    pub async fn fetch_remote_candidates(username: &str) -> Result<Vec<IceCandidate>> {
        fetch_remote_candidates(username).await
    }

    pub async fn exchange_candidates(username: &str) -> Result<Vec<IceCandidate>> {
        fetch_remote_candidates(username).await
    }
}
