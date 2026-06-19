use crate::presence::api::{announce_offline, announce_online, fetch_presence};
use crate::presence::models::PresenceInfo;
use crate::session::manager::{get_current_session, start_session, stop_session};
use anyhow::Result;

pub async fn get_user_status(username: &str) -> Result<PresenceInfo> {
    fetch_presence(username).await
}

pub async fn set_user_online() -> Result<String> {
    let session = start_session()?;
    announce_online(
        &session.username,
        &session.session_id,
        &session.client_version,
    )
    .await?;
    Ok(session.session_id)
}

pub async fn set_user_offline() -> Result<()> {
    let session = get_current_session()?;
    announce_offline(&session.username, &session.session_id).await?;
    let _ = stop_session()?;
    Ok(())
}
