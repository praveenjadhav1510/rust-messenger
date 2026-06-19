use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PunchState {
    New,
    Probing,
    Waiting,
    Established,
    Failed,
}

impl std::fmt::Display for PunchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PunchState::New => write!(f, "NEW"),
            PunchState::Probing => write!(f, "PROBING"),
            PunchState::Waiting => write!(f, "WAITING"),
            PunchState::Established => write!(f, "ESTABLISHED"),
            PunchState::Failed => write!(f, "FAILED"),
        }
    }
}
