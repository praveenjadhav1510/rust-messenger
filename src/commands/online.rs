use crate::presence::manager::set_user_online;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    let session_id = set_user_online().await?;
    println!("You are now online.");
    println!("Session ID: {}", session_id);
    Ok(())
}
