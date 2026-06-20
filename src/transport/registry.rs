use crate::connection::models::TransportType;
use crate::transport::mock::MockTransport;
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};

pub fn create_transport(transport_type: TransportType, peer_username: &str) -> Result<Box<dyn Transport + Send + Sync>> {
    match transport_type {
        TransportType::Mock => {
            let mut mock = MockTransport::new();
            // Connect the mock transport to simulate being active
            let _ = mock.connect();
            Ok(Box::new(mock))
        }
        TransportType::Tcp => Err(anyhow!("TCP transport not implemented yet.")),
        TransportType::Ice => {
            let punch_sessions = crate::punch::session::load_punch_sessions()?;
            let punch_session = punch_sessions
                .iter()
                .find(|s| s.peer.eq_ignore_ascii_case(peer_username))
                .ok_or_else(|| anyhow!("No established punch session found for peer '{}'. Run 'rust-messenger punch {}' first.", peer_username, peer_username))?;

            if punch_session.state != crate::punch::state::PunchState::Established {
                return Err(anyhow!("Punch session for peer '{}' is not established.", peer_username));
            }

            let local_session = crate::session::manager::get_current_session()?;
            let is_loopback = (punch_session.selected_pair.local.address == "127.0.0.1"
                || punch_session.selected_pair.local.address == "localhost")
                && (punch_session.selected_pair.remote.address == "127.0.0.1"
                    || punch_session.selected_pair.remote.address == "localhost");

            let (local_port, remote_port) = if is_loopback {
                if local_session.username.to_lowercase() < peer_username.to_lowercase() {
                    (5001, 5002)
                } else {
                    (5002, 5001)
                }
            } else {
                (
                    5000,
                    punch_session.selected_pair.remote.port,
                )
            };

            let remote_addr: std::net::SocketAddr = format!(
                "{}:{}",
                punch_session.selected_pair.remote.address, remote_port
            )
            .parse()?;

            let mut transport = crate::network::udp::UdpTransport::new(local_port, Some(remote_addr));
            transport.connect()?;
            Ok(Box::new(transport))
        }
        TransportType::Turn => Err(anyhow!("TURN transport not implemented yet.")),
    }
}
