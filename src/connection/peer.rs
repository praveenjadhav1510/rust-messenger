use crate::connection::models::{PeerSession, TransportType};
use crate::connection::state::ConnectionState;

impl PeerSession {
    pub fn new(username: String, session_id: String, transport: TransportType) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            username,
            session_id,
            state: ConnectionState::Connected,
            connected_at: now.clone(),
            last_seen: now,
            transport,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_session_serialization() {
        let session = PeerSession::new(
            "ankita".to_string(),
            "uuid-1234".to_string(),
            TransportType::Mock,
        );
        let serialized = serde_json::to_string(&session).unwrap();
        assert!(serialized.contains(r#""username":"ankita""#));
        assert!(serialized.contains(r#""state":"CONNECTED""#));
        assert!(serialized.contains(r#""transport":"MOCK""#));

        let deserialized: PeerSession = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.username, "ankita");
        assert_eq!(deserialized.state, ConnectionState::Connected);
        assert_eq!(deserialized.transport, TransportType::Mock);
    }
}
