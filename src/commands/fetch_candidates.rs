use crate::candidates::manager::CandidateManager;
use crate::network::candidate::CandidateType;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let resp = CandidateManager::fetch_remote(username).await?;
    println!("Candidate List for {}", username);
    println!();
    for candidate in resp.candidates {
        let type_str = match candidate.candidate_type {
            CandidateType::Host => "HOST",
            CandidateType::ServerReflexive => "SRFLX",
            CandidateType::Relay => "RELAY",
        };
        println!("{} {}:{}", type_str, candidate.address, candidate.port);
        println!();
    }
    Ok(())
}
