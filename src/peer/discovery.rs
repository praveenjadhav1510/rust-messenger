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
        let presence = get_user_status(username).await?;

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
