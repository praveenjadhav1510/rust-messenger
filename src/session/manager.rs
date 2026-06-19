use crate::session::models::Session;
use crate::storage::filesystem::{get_storage_dir, read_profile};
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

fn get_session_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("session.json"))
}

pub fn save_session(session: &Session) -> Result<()> {
    let path = get_session_path()?;
    let content = serde_json::to_string_pretty(session)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_session() -> Result<Session> {
    let path = get_session_path()?;
    if !path.exists() {
        return Err(anyhow!("Session not found."));
    }
    let content = fs::read_to_string(path)?;
    let session: Session = serde_json::from_str(&content)?;
    Ok(session)
}

pub fn start_session() -> Result<Session> {
    let profile = read_profile()?;
    let username = profile
        .username
        .as_ref()
        .ok_or_else(|| anyhow!("No registered identity found."))?;

    let session = match load_session() {
        Ok(s) if s.online => {
            // If session already online: reuse it.
            s
        }
        _ => {
            let s = Session {
                session_id: uuid::Uuid::new_v4().to_string(),
                username: username.clone(),
                started_at: chrono::Utc::now().to_rfc3339(),
                online: true,
                client_version: "0.2.0".to_string(),
            };
            save_session(&s)?;
            s
        }
    };
    Ok(session)
}

pub fn stop_session() -> Result<Session> {
    let mut session = load_session()?;
    session.online = false;
    save_session(&session)?;
    Ok(session)
}

pub fn is_online() -> bool {
    load_session().map(|s| s.online).unwrap_or(false)
}

pub fn get_current_session() -> Result<Session> {
    load_session()
}
