use crate::connection::manager::ConnectionManager;
use crate::ice::connectivity::{
    ConnectivityManager, IceSession, load_ice_sessions, save_ice_sessions,
};
use crate::ice::state::IceConnectionState;
use crate::network::candidate::CandidateType;
use crate::network::discovery::DiscoveryManager;
use crate::peer::discovery::PeerDiscoveryManager;
use crate::session::manager::get_current_session;
use anyhow::{Result, anyhow};

pub async fn exec(username: &str) -> Result<()> {
    let peer_session = ConnectionManager::get_peer(username)?;

    let current_session = get_current_session()?;
    let sender = current_session.username;

    let remote_candidates = PeerDiscoveryManager::fetch_remote_candidates(username).await?;
    let local_candidates = DiscoveryManager::load_candidates()?;

    let mut pairs =
        ConnectivityManager::build_candidate_pairs(&local_candidates, &remote_candidates);
    ConnectivityManager::prioritize_pairs(&mut pairs);

    ConnectivityManager::run_checks(&mut pairs, &sender, username).await?;

    let working_pair = ConnectivityManager::select_working_pair(&pairs)
        .ok_or_else(|| anyhow!("ICE connectivity check failed: no reachable candidate pairs."))?;

    let mut ice_sessions = load_ice_sessions()?;
    ice_sessions.retain(|s| !s.peer.eq_ignore_ascii_case(username));
    ice_sessions.push(IceSession {
        peer: peer_session.username.clone(),
        selected_pair: working_pair.clone(),
        established_at: chrono::Utc::now().to_rfc3339(),
    });
    save_ice_sessions(&ice_sessions)?;

    let local_type = match working_pair.local.candidate_type {
        CandidateType::Host => "HOST",
        CandidateType::ServerReflexive => "SRFLX",
        CandidateType::Relay => "RELAY",
    };
    let remote_type = match working_pair.remote.candidate_type {
        CandidateType::Host => "HOST",
        CandidateType::ServerReflexive => "SRFLX",
        CandidateType::Relay => "RELAY",
    };

    println!("ICE Connectivity Successful");
    println!();
    println!("Selected Pair:");
    println!();
    println!("{} ↔ {}", local_type, remote_type);
    println!();
    println!("State:");
    println!();
    println!("{}", IceConnectionState::Connected.to_string());

    Ok(())
}
