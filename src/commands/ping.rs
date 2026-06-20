use crate::connection::manager::ConnectionManager;
use crate::protocol::packet::{Packet, PacketType};
use crate::storage::filesystem::read_profile;
use crate::transport::manager::TransportManager;
use crate::transport::registry::create_transport;
use anyhow::{Result, anyhow};
use chrono::Utc;
use uuid::Uuid;

pub async fn exec(username: &str) -> Result<()> {
    // 1. Get the peer session
    let peer = ConnectionManager::get_peer(username)?;

    // 2. Load our own profile for sender name
    let profile = read_profile()?;
    let sender = profile
        .username
        .ok_or_else(|| anyhow!("No registered identity found."))?;

    // 3. Initialize transport manager and transport
    let mut transport_manager = TransportManager::new();
    let transport = create_transport(peer.transport, username)?;
    transport_manager.register_transport(transport);

    // 4. Construct PING packet
    let packet = Packet {
        version: 1,
        packet_type: PacketType::Ping,
        message_id: Uuid::new_v4(),
        sender,
        recipient: username.to_string(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: String::new(),
        signature: "mock-signature".to_string(),
    };

    // 5. Validate packet
    packet.validate()?;

    // 6. Send the packet
    transport_manager.send_packet(packet)?;

    println!("Ping successful.");
    Ok(())
}
