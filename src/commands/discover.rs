use crate::peer::discovery::PeerDiscoveryManager;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let peer = PeerDiscoveryManager::discover_peer(username).await?;

    println!("Username: {}", peer.username);
    println!();
    println!("Online: {}", peer.online);
    println!();
    println!("Capabilities:");
    println!();
    println!("ICE: {}", peer.capabilities.supports_ice);
    println!("TURN: {}", peer.capabilities.supports_turn);
    println!();
    println!("Candidates:");
    println!();
    for cand in peer.candidates {
        let type_str = match cand.candidate_type {
            crate::network::candidate::CandidateType::Host => "HOST",
            crate::network::candidate::CandidateType::ServerReflexive => "SRFLX",
            crate::network::candidate::CandidateType::Relay => "RELAY",
        };
        println!("{} {}:{}", type_str, cand.address, cand.port);
    }

    Ok(())
}
