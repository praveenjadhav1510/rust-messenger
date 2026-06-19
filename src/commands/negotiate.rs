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

    // 3. Select best candidate pair
    let best_pair = CandidateSelector::select_best_pair(&local_candidates, &remote_candidates)?;

    // 4. Generate offer
    let offer = HandshakeManager::create_offer(username, local_candidates)?;

    // 5. Simulate acceptance
    let answer = HandshakeManager::accept_offer(offer.clone(), best_pair.remote.clone())?;
    HandshakeManager::complete_handshake(&offer.offer_id, answer)?;

    // 6. Create peer session
    let peer_session = PeerSession::new(
        username.to_string(),
        offer.session_id,
        crate::connection::models::TransportType::Mock,
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
