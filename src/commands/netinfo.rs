use crate::network::discovery::DiscoveryManager;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    let info = DiscoveryManager::discover().await?;
    let candidates = DiscoveryManager::get_candidates()?;

    println!("{:<14}{}", "Local IP:", info.local_ip);
    println!("{:<14}{}", "Public IP:", info.public_ip);
    println!("{:<14}{}", "Public Port:", info.public_port);
    println!();
    println!("{:<14}{}", "NAT Type:", info.nat_type.to_string());
    println!();
    println!("ICE Candidates:");
    println!();
    for cand in candidates {
        println!("{}", cand.to_display_string());
    }

    Ok(())
}
