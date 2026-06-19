use crate::connection::manager::ConnectionManager;
use crate::ice::connectivity::load_ice_sessions;
use crate::peer::discovery::PeerDiscoveryManager;
use crate::punch::coordinator::HolePunchCoordinator;
use crate::punch::state::PunchState;
use crate::session::manager::get_current_session;
use anyhow::{Result, anyhow};

pub async fn exec(username: &str) -> Result<()> {
    // 1. Verify negotiated peer session exists
    let peer_session = ConnectionManager::get_peer(username)?;
    let current_session = get_current_session()?;
    let sender = current_session.username;

    // 2. Verify candidate pair exists
    let ice_sessions = load_ice_sessions()?;
    let ice_session = ice_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(username))
        .ok_or_else(|| {
            anyhow!(
                "No candidate pair selected. Run 'rust-messenger ice-check {}' first.",
                username
            )
        })?;
    let selected_pair = ice_session.selected_pair.clone();

    // 3. Load remote candidates (just verifying they exist on registry)
    let _remote_candidates = PeerDiscoveryManager::fetch_remote_candidates(username).await?;

    // 4. Start punching
    let coordinator = HolePunchCoordinator::new(
        &current_session.session_id,
        &sender,
        &peer_session.username,
        selected_pair,
    )
    .await?;

    let punch_session = coordinator.start_punch().await?;

    if punch_session.state == PunchState::Established {
        println!();
        println!("Connection established.");
        println!();
        println!("Peer:");
        println!("{}", punch_session.peer);
        println!();
        println!("State:");
        println!("{}", punch_session.state.to_string());
    } else {
        println!();
        println!("UDP hole punching failed.");
        println!();
        println!("Peer:");
        println!("{}", punch_session.peer);
        println!();
        println!("State:");
        println!("{}", punch_session.state.to_string());
    }

    Ok(())
}
