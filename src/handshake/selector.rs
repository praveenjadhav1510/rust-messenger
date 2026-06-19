use crate::network::candidate::{CandidateType, IceCandidate};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedCandidatePair {
    pub local: IceCandidate,
    pub remote: IceCandidate,
}

pub struct CandidateSelector;

fn score_pair(local: &IceCandidate, remote: &IceCandidate) -> u64 {
    let type_score = match (local.candidate_type, remote.candidate_type) {
        (CandidateType::Host, CandidateType::Host) => 10000,
        (CandidateType::Host, CandidateType::ServerReflexive)
        | (CandidateType::ServerReflexive, CandidateType::Host) => 8000,
        (CandidateType::ServerReflexive, CandidateType::ServerReflexive) => 6000,
        _ => 4000,
    };

    // Add priority as a tie-breaker
    type_score + (local.priority as u64) + (remote.priority as u64)
}

impl CandidateSelector {
    pub fn select_best_pair(
        local: &[IceCandidate],
        remote: &[IceCandidate],
    ) -> Result<SelectedCandidatePair> {
        if local.is_empty() {
            return Err(anyhow!("Local candidates list is empty."));
        }
        if remote.is_empty() {
            return Err(anyhow!("Remote candidates list is empty."));
        }

        let mut best_pair = None;
        let mut best_score = 0;

        for l in local {
            for r in remote {
                let score = score_pair(l, r);
                if best_pair.is_none() || score > best_score {
                    best_pair = Some(SelectedCandidatePair {
                        local: l.clone(),
                        remote: r.clone(),
                    });
                    best_score = score;
                }
            }
        }

        best_pair.ok_or_else(|| anyhow!("Could not select a candidate pair."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_scoring() {
        let local_host = IceCandidate {
            foundation: "1".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 100,
            address: "192.168.1.8".to_string(),
            port: 5000,
            candidate_type: CandidateType::Host,
        };
        let local_srflx = IceCandidate {
            foundation: "2".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 50,
            address: "49.1.2.3".to_string(),
            port: 51322,
            candidate_type: CandidateType::ServerReflexive,
        };

        let remote_host = IceCandidate {
            foundation: "1".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 100,
            address: "192.168.1.20".to_string(),
            port: 5000,
            candidate_type: CandidateType::Host,
        };
        let remote_srflx = IceCandidate {
            foundation: "2".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 50,
            address: "49.5.6.7".to_string(),
            port: 51322,
            candidate_type: CandidateType::ServerReflexive,
        };

        let pair = CandidateSelector::select_best_pair(
            &[local_host.clone(), local_srflx.clone()],
            &[remote_host.clone(), remote_srflx.clone()],
        )
        .unwrap();
        assert_eq!(pair.local.candidate_type, CandidateType::Host);
        assert_eq!(pair.remote.candidate_type, CandidateType::Host);

        let pair2 = CandidateSelector::select_best_pair(
            &[local_srflx.clone()],
            &[remote_host.clone(), remote_srflx.clone()],
        )
        .unwrap();
        assert_eq!(pair2.local.candidate_type, CandidateType::ServerReflexive);
        assert_eq!(pair2.remote.candidate_type, CandidateType::Host);
    }
}
