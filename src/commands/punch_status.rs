use crate::punch::session::load_punch_sessions;
use anyhow::{Result, anyhow};
use chrono::DateTime;

pub fn exec(username: &str) -> Result<()> {
    let sessions = load_punch_sessions()?;
    let session = sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(username))
        .ok_or_else(|| anyhow!("No punch session found for peer: {}", username))?;

    let formatted_time = if let Ok(dt) = DateTime::parse_from_rfc3339(&session.started_at) {
        dt.format("%Y-%m-%d %H:%M UTC").to_string()
    } else {
        session.started_at.clone()
    };

    println!("Peer: {}", session.peer);
    println!();
    println!("State: {}", session.state.to_string());
    println!();
    println!("Attempts: {}", session.attempts);
    println!();
    println!("Started:");
    println!("{}", formatted_time);

    Ok(())
}
