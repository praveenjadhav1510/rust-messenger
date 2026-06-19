use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateType {
    Host,
    ServerReflexive,
    Relay,
}

impl std::fmt::Display for CandidateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CandidateType::Host => write!(f, "HOST"),
            CandidateType::ServerReflexive => write!(f, "SERVER_REFLEXIVE"),
            CandidateType::Relay => write!(f, "RELAY"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IceCandidate {
    pub foundation: String,
    pub component: u16,
    pub transport: String,
    pub priority: u32,
    pub address: String,
    pub port: u16,
    #[serde(rename = "candidateType")]
    pub candidate_type: CandidateType,
}

impl IceCandidate {
    pub fn to_display_string(&self) -> String {
        let type_str = match self.candidate_type {
            CandidateType::Host => "HOST",
            CandidateType::ServerReflexive => "SRFLX",
            CandidateType::Relay => "RELAY",
        };
        format!(
            "{} {} {}:{}",
            self.foundation, type_str, self.address, self.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_display() {
        let cand = IceCandidate {
            foundation: "1".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 2130706431,
            address: "192.168.1.8".to_string(),
            port: 5000,
            candidate_type: CandidateType::Host,
        };
        assert_eq!(cand.to_display_string(), "1 HOST 192.168.1.8:5000");

        let cand_srflx = IceCandidate {
            foundation: "2".to_string(),
            component: 1,
            transport: "UDP".to_string(),
            priority: 1686052863,
            address: "49.1.2.3".to_string(),
            port: 51322,
            candidate_type: CandidateType::ServerReflexive,
        };
        assert_eq!(cand_srflx.to_display_string(), "2 SRFLX 49.1.2.3:51322");
    }
}
