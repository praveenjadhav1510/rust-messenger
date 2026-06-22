use crate::connection::models::{PeerSession, TransportType};
use crate::connection::state::ConnectionState;
use crate::contacts::manager::get_contact;
use crate::contacts::models::TrustLevel;
use crate::presence::manager::get_user_status;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub struct ConnectionManager;

fn get_sessions_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("active_sessions.json"))
}

pub fn load_sessions() -> Result<Vec<PeerSession>> {
    let path = get_sessions_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let sessions: Vec<PeerSession> = serde_json::from_str(&content)?;
    Ok(sessions)
}

pub fn save_sessions(sessions: &[PeerSession]) -> Result<()> {
    let path = get_sessions_path()?;
    let content = serde_json::to_string_pretty(sessions)?;
    fs::write(path, content)?;
    Ok(())
}

impl ConnectionManager {
    pub async fn connect_peer(username: &str) -> Result<PeerSession> {
        // 1. Verify contact exists.
        let contact = get_contact(username)?;

        // 2. Verify contact not blocked.
        if contact.trust_level == TrustLevel::Blocked {
            return Err(anyhow!("Contact '{}' is blocked.", username));
        }

        // 3. Verify contact online using presence API.
        let presence = get_user_status(username).await?;
        if !presence.online {
            return Err(anyhow!("Contact '{}' is offline.", username));
        }

        // 4. Create peer session.
        let session_id = uuid::Uuid::new_v4().to_string();
        let peer_session =
            PeerSession::new(contact.username.clone(), session_id, TransportType::Ice);

        // Save peer session.
        let mut sessions = load_sessions()?;
        sessions.retain(|s| !s.username.eq_ignore_ascii_case(username));
        sessions.push(peer_session.clone());
        save_sessions(&sessions)?;

        Ok(peer_session)
    }

    pub fn disconnect_peer(username: &str) -> Result<()> {
        let mut sessions = load_sessions()?;
        let initial_len = sessions.len();
        sessions.retain(|s| !s.username.eq_ignore_ascii_case(username));

        if sessions.len() == initial_len {
            return Err(anyhow!("Peer session not found."));
        }

        save_sessions(&sessions)?;
        Ok(())
    }

    pub async fn reconnect_peer(username: &str) -> Result<()> {
        // Check if session exists first.
        let _ = Self::get_peer(username)?;

        Self::update_state(username, ConnectionState::Reconnecting)?;

        let mut success = false;
        let backoffs = [
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
        ];

        for i in 0..3 {
            match get_user_status(username).await {
                Ok(presence) if presence.online => {
                    success = true;
                    break;
                }
                _ => {
                    tokio::time::sleep(backoffs[i]).await;
                }
            }
        }

        if success {
            Self::update_state(username, ConnectionState::Connected)?;
            Ok(())
        } else {
            Self::update_state(username, ConnectionState::Failed)?;
            Err(anyhow!("Reconnection failed after 3 attempts."))
        }
    }

    pub fn get_peer(username: &str) -> Result<PeerSession> {
        let sessions = load_sessions()?;
        sessions
            .into_iter()
            .find(|s| s.username.eq_ignore_ascii_case(username))
            .ok_or_else(|| anyhow!("Peer session not found."))
    }

    pub fn list_peers() -> Result<Vec<PeerSession>> {
        load_sessions()
    }

    pub fn is_connected(username: &str) -> bool {
        Self::get_peer(username)
            .map(|s| s.state == ConnectionState::Connected)
            .unwrap_or(false)
    }

    pub fn update_state(username: &str, state: ConnectionState) -> Result<PeerSession> {
        let mut sessions = load_sessions()?;
        let mut updated = None;

        for session in &mut sessions {
            if session.username.eq_ignore_ascii_case(username) {
                session.state = state;
                updated = Some(session.clone());
                break;
            }
        }

        if let Some(session) = updated {
            save_sessions(&sessions)?;
            Ok(session)
        } else {
            Err(anyhow!("Peer session not found."))
        }
    }
}
