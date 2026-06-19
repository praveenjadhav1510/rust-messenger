use crate::connection::manager::load_sessions;
use crate::network::session_keys::{derive_shared_secret, generate_ephemeral_keypair};
use crate::secure::session::{SecureSession, load_secure_sessions, save_secure_sessions};
use anyhow::{Result, anyhow};

pub fn bootstrap_secure_session(username: &str) -> Result<SecureSession> {
    // 1. Verify negotiated connection exists
    let peers = load_sessions()?;
    let peer_session = peers
        .iter()
        .find(|p| p.username.eq_ignore_ascii_case(username))
        .ok_or_else(|| anyhow!("No negotiated connection found for '{}'.", username))?;

    // 2. Generate ephemeral keys (local and simulated peer)
    let (local_secret, _local_public) = generate_ephemeral_keypair();
    let (_peer_secret, peer_public) = generate_ephemeral_keypair();

    // 3. Derive shared secret (stored only in memory, never persisted)
    let _shared_secret = derive_shared_secret(local_secret, &peer_public);

    // 4. Create secure session
    let secure_session = SecureSession {
        session_id: uuid::Uuid::new_v4().to_string(),
        peer: peer_session.username.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        established: true,
    };

    // 5. Store metadata only
    let mut secure_sessions = load_secure_sessions()?;
    secure_sessions.retain(|s| !s.peer.eq_ignore_ascii_case(username));
    secure_sessions.push(secure_session.clone());
    save_secure_sessions(&secure_sessions)?;

    Ok(secure_session)
}
