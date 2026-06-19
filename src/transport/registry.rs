use crate::connection::models::TransportType;
use crate::transport::mock::MockTransport;
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};

pub fn create_transport(transport_type: TransportType) -> Result<Box<dyn Transport + Send + Sync>> {
    match transport_type {
        TransportType::Mock => {
            let mut mock = MockTransport::new();
            // Connect the mock transport to simulate being active
            let _ = mock.connect();
            Ok(Box::new(mock))
        }
        TransportType::Tcp => Err(anyhow!("TCP transport not implemented yet.")),
        TransportType::Ice => Err(anyhow!("ICE transport not implemented yet.")),
        TransportType::Turn => Err(anyhow!("TURN transport not implemented yet.")),
    }
}
