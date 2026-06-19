use crate::connection::manager::ConnectionManager;
use anyhow::Result;

pub fn exec() -> Result<()> {
    let peers = ConnectionManager::list_peers()?;
    println!("{:<14}{:<13}{}", "USERNAME", "STATE", "TRANSPORT");
    println!();
    for peer in peers {
        println!(
            "{:<14}{:<13}{}",
            peer.username,
            peer.state.to_string(),
            peer.transport.to_string()
        );
    }
    Ok(())
}
