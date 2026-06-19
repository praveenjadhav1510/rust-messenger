use crate::connection::manager::ConnectionManager;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let session = ConnectionManager::connect_peer(username).await?;
    println!("Connected to {}", session.username);
    println!("Transport: {}", session.transport);
    Ok(())
}
