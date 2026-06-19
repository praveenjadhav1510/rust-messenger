use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "DISCONNECTED"),
            ConnectionState::Connecting => write!(f, "CONNECTING"),
            ConnectionState::Connected => write!(f, "CONNECTED"),
            ConnectionState::Reconnecting => write!(f, "RECONNECTING"),
            ConnectionState::Failed => write!(f, "FAILED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "CONNECTED");
        assert_eq!(ConnectionState::Disconnected.to_string(), "DISCONNECTED");
        assert_eq!(ConnectionState::Connecting.to_string(), "CONNECTING");
        assert_eq!(ConnectionState::Reconnecting.to_string(), "RECONNECTING");
        assert_eq!(ConnectionState::Failed.to_string(), "FAILED");
    }
}
