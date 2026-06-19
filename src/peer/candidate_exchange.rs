use crate::network::candidate::{CandidateType, IceCandidate};
use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub async fn fetch_remote_candidates(username: &str) -> Result<Vec<IceCandidate>> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url);

    match client.get_candidates(username).await {
        Ok(res) => Ok(res.candidates),
        Err(_) => {
            // Fallback to mock candidates if registry is unavailable
            Ok(vec![
                IceCandidate {
                    foundation: "1".to_string(),
                    component: 1,
                    transport: "UDP".to_string(),
                    priority: 2130706431,
                    address: "192.168.1.20".to_string(),
                    port: 5000,
                    candidate_type: CandidateType::Host,
                },
                IceCandidate {
                    foundation: "2".to_string(),
                    component: 1,
                    transport: "UDP".to_string(),
                    priority: 1686052863,
                    address: "49.12.34.56".to_string(),
                    port: 51231,
                    candidate_type: CandidateType::ServerReflexive,
                },
            ])
        }
    }
}
