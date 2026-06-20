use crate::connection::manager::{load_sessions, save_sessions};
use crate::connection::models::PeerSession;
use crate::handshake::manager::HandshakeManager;
use crate::handshake::selector::CandidateSelector;
use crate::network::candidate::CandidateType;
use crate::network::discovery::DiscoveryManager;
use crate::peer::candidate_exchange::fetch_remote_candidates;
use anyhow::{Result, anyhow};

pub async fn exec(username: &str) -> Result<()> {
    // 1. Fetch remote candidates
    let remote_candidates = fetch_remote_candidates(username).await?;
    if remote_candidates.is_empty() {
        return Err(anyhow!(
            "No remote candidates found for peer '{}'.",
            username
        ));
    }

    // 2. Load local candidates
    let local_candidates = DiscoveryManager::load_candidates()?;
    if local_candidates.is_empty() {
        return Err(anyhow!(
            "No local candidates found. Run 'rust-messenger netinfo' first."
        ));
    }

    // 3. Require successful ICE check results
    let ice_sessions = crate::ice::connectivity::load_ice_sessions()?;
    let ice_session = ice_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(username))
        .ok_or_else(|| anyhow!("No successful ICE check found for peer '{}'. Run 'rust-messenger ice-check {}' first.", username, username))?;
    
    let best_pair = ice_session.selected_pair.clone();

    // 4. Resolve presence session IDs to derive a deterministic, shared session ID
    let local_presence = crate::session::manager::get_current_session()?;
    let remote_presence = crate::presence::manager::get_user_status(username).await?;
    let remote_session_id = remote_presence.session_id.ok_or_else(|| {
        anyhow!("Peer '{}' is offline or has no active session.", username)
    })?;

    let mut presence_ids = vec![local_presence.session_id.clone(), remote_session_id];
    presence_ids.sort();
    let combined = presence_ids.join("_");
    
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let negotiated_session_id = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // 5. Create peer session using the ICE transport and the deterministic session ID
    let peer_session = PeerSession::new(
        username.to_string(),
        negotiated_session_id,
        crate::connection::models::TransportType::Ice,
    );
    let mut sessions = load_sessions()?;
    sessions.retain(|s| !s.username.eq_ignore_ascii_case(username));
    sessions.push(peer_session);
    save_sessions(&sessions)?;

    // Display result
    let local_type = match best_pair.local.candidate_type {
        CandidateType::Host => "HOST",
        CandidateType::ServerReflexive => "SRFLX",
        CandidateType::Relay => "RELAY",
    };
    let remote_type = match best_pair.remote.candidate_type {
        CandidateType::Host => "HOST",
        CandidateType::ServerReflexive => "SRFLX",
        CandidateType::Relay => "RELAY",
    };

    println!("Negotiation successful.");
    println!();
    println!("Selected Pair:");
    println!();
    println!("{} ↔ {}", local_type, remote_type);

    Ok(())
}
