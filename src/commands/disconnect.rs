use crate::connection::manager::ConnectionManager;
use anyhow::Result;

pub fn exec(username: &str) -> Result<()> {
    ConnectionManager::disconnect_peer(username)?;
    println!("Disconnected from {}", username);
    Ok(())
}
