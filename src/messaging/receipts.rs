use crate::chat::models::{Direction, Message, MessageStatus};
use crate::chat::storage::{load_messages, save_messages};
use crate::protocol::packet::{Packet, PacketType};
use crate::punch::session::load_punch_sessions;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use uuid::Uuid;

pub fn mark_incoming_messages_read(username: &str) -> Result<()> {
    let mut messages = load_messages(username)?;
    let mut updated = false;
    for msg in &mut messages {
        if msg.direction == Direction::Incoming && msg.status != MessageStatus::Read {
            msg.status = MessageStatus::Read;
            updated = true;

            let msg_id = msg.id;
            let peer = username.to_string();
            tokio::spawn(async move {
                let _ = send_read_receipt_packet(&peer, msg_id).await;
            });
        }
    }
    if updated {
        save_messages(username, &messages)?;
    }
    Ok(())
}

pub fn update_message_status(
    username: &str,
    message_id: Uuid,
    status: MessageStatus,
) -> Result<()> {
    let mut messages = load_messages(username)?;
    if let Some(index) = messages.iter().position(|m| m.id == message_id) {
        messages[index].status = status;
        save_messages(username, &messages)?;
    }
    Ok(())
}

async fn send_read_receipt_packet(peer: &str, message_id: Uuid) -> Result<()> {
    let current_session = crate::session::manager::get_current_session()?;
    let punch_sessions = load_punch_sessions()?;
    let punch_session = punch_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(peer))
        .ok_or_else(|| anyhow!("No established punch session found for peer: {}", peer))?;

    let is_loopback = (punch_session.selected_pair.local.address == "127.0.0.1"
        || punch_session.selected_pair.local.address == "localhost")
        && (punch_session.selected_pair.remote.address == "127.0.0.1"
            || punch_session.selected_pair.remote.address == "localhost");

    let (local_port, remote_port) = if is_loopback {
        if current_session.username.to_lowercase() < peer.to_lowercase() {
            (5001, 5002)
        } else {
            (5002, 5001)
        }
    } else {
        (
            punch_session.selected_pair.local.port,
            punch_session.selected_pair.remote.port,
        )
    };

    let local_addr = format!(
        "{}:{}",
        punch_session.selected_pair.local.address, local_port
    );
    let remote_addr: SocketAddr = format!(
        "{}:{}",
        punch_session.selected_pair.remote.address, remote_port
    )
    .parse()?;

    let socket = match UdpSocket::bind(local_addr).await {
        Ok(s) => s,
        Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
    };

    let read_payload = serde_json::json!({
        "messageId": message_id,
    });
    let read_pkt = Packet {
        version: 1,
        packet_type: PacketType::MessageRead,
        message_id: Uuid::new_v4(),
        sender: current_session.username,
        recipient: peer.to_string(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: read_payload.to_string(),
        signature: "message-read".to_string(),
    };

    let encoded = read_pkt.encode()?;
    socket.send_to(encoded.as_bytes(), remote_addr).await?;
    Ok(())
}
