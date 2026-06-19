use crate::network::udp::UdpTransport;
use crate::protocol::packet::{Packet, PacketType};
use crate::transport::r#trait::Transport;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::net::SocketAddr;
use uuid::Uuid;

pub fn exec() -> Result<()> {
    let port = 12345;
    let remote_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut transport = UdpTransport::new(port, Some(remote_addr));
    transport.bind(port)?;

    let packet = Packet {
        version: 1,
        packet_type: PacketType::Ping,
        message_id: Uuid::new_v4(),
        sender: "test-sender".to_string(),
        recipient: "test-recipient".to_string(),
        timestamp: Utc::now(),
        nonce: "test-nonce".to_string(),
        encrypted_payload: "test-payload".to_string(),
        signature: "test-signature".to_string(),
    };

    transport.send(packet.clone())?;

    let received = transport
        .receive()?
        .ok_or_else(|| anyhow!("No packet received."))?;

    if received.message_id != packet.message_id {
        return Err(anyhow!(
            "Mismatched message ID. Received packet does not match."
        ));
    }

    println!("UDP transport operational.");
    Ok(())
}
