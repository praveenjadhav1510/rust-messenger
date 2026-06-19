use crate::ice::connectivity::ConnectivityManager;
use crate::network::discovery::DiscoveryManager;
use crate::peer::discovery::PeerDiscoveryManager;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let remote_candidates = PeerDiscoveryManager::fetch_remote_candidates(username).await?;
    let local_candidates = DiscoveryManager::load_candidates()?;

    let mut pairs =
        ConnectivityManager::build_candidate_pairs(&local_candidates, &remote_candidates);
    ConnectivityManager::prioritize_pairs(&mut pairs);

    println!("{:<25}{:<24}{}", "LOCAL", "REMOTE", "PRIORITY");
    println!();
    for pair in pairs {
        let local_type = match pair.local.candidate_type {
            crate::network::candidate::CandidateType::Host => "HOST",
            crate::network::candidate::CandidateType::ServerReflexive => "SRFLX",
            crate::network::candidate::CandidateType::Relay => "RELAY",
        };
        let remote_type = match pair.remote.candidate_type {
            crate::network::candidate::CandidateType::Host => "HOST",
            crate::network::candidate::CandidateType::ServerReflexive => "SRFLX",
            crate::network::candidate::CandidateType::Relay => "RELAY",
        };
        let local_str = format!("{}:{}:{}", local_type, pair.local.address, pair.local.port);
        let remote_str = format!(
            "{}:{}:{}",
            remote_type, pair.remote.address, pair.remote.port
        );

        println!("{:<25}{:<24}{}", local_str, remote_str, pair.priority);
    }

    Ok(())
}
