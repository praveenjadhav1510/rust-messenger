use crate::ice::state::IceConnectionState;
use crate::network::candidate::IceCandidate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePair {
    pub local: IceCandidate,
    pub remote: IceCandidate,
    pub priority: u32,
    pub state: IceConnectionState,
}

impl CandidatePair {
    pub fn new(local: IceCandidate, remote: IceCandidate, priority: u32) -> Self {
        Self {
            local,
            remote,
            priority,
            state: IceConnectionState::Checking,
        }
    }
}
