use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IceConnectionState {
    New,
    Checking,
    Connected,
    Failed,
    Disconnected,
}

impl std::fmt::Display for IceConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IceConnectionState::New => write!(f, "NEW"),
            IceConnectionState::Checking => write!(f, "CHECKING"),
            IceConnectionState::Connected => write!(f, "CONNECTED"),
            IceConnectionState::Failed => write!(f, "FAILED"),
            IceConnectionState::Disconnected => write!(f, "DISCONNECTED"),
        }
    }
}
