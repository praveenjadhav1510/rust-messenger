use crate::punch::session::{PunchSession, load_punch_sessions};
use crate::punch::state::PunchState;
use anyhow::{Result, anyhow};

pub struct HolePunchManager;

impl HolePunchManager {
    pub fn resume_session(peer: &str) -> Result<PunchSession> {
        let sessions = load_punch_sessions()?;
        let session = sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .ok_or_else(|| anyhow!("No existing punch session found for peer: {}", peer))?;

        if session.state != PunchState::Established {
            return Err(anyhow!(
                "Session not established. Current state: {}",
                session.state
            ));
        }

        Ok(session.clone())
    }
}
