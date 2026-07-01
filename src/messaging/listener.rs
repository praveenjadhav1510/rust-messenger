use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::append_message;
use crate::messaging::receipts::update_message_status;
use crate::protocol::packet::{Packet, PacketType};
use crate::punch::session::load_punch_sessions;
use crate::secure::session::load_secure_sessions;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use uuid::Uuid;

pub struct MessageListener;

pub fn get_listener_port() -> Result<u16> {
    let current_session = crate::session::manager::get_current_session()?;
    let punch_sessions = load_punch_sessions()?;

    let established = punch_sessions
        .iter()
        .find(|s| s.state == crate::punch::state::PunchState::Established);

    if let Some(session) = established {
        let is_loopback = session.selected_pair.local.address == session.selected_pair.remote.address
            || session.selected_pair.local.address == "127.0.0.1"
            || session.selected_pair.local.address == "localhost"
            || session.selected_pair.remote.address == "127.0.0.1"
            || session.selected_pair.remote.address == "localhost";

        if is_loopback {
            if current_session.username.to_lowercase() < session.peer.to_lowercase() {
                Ok(5001)
            } else {
                Ok(5002)
            }
        } else {
            Ok(session.selected_pair.local.port)
        }
    } else {
        Ok(5000)
    }
}

pub fn get_secure_session_key(session_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

impl MessageListener {
    pub async fn run() -> Result<()> {
        let port = get_listener_port()?;
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        println!("Listening for incoming messages...");
        println!();

        let mut buf = vec![0u8; 65535];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, from_addr)) => {
                    if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                        if let Ok(packet) = Packet::decode(payload_str) {
                            let _ = Self::handle_packet(packet, from_addr, &socket).await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving packet: {}", e);
                }
            }
        }
    }

    async fn handle_packet(
        packet: Packet,
        from_addr: SocketAddr,
        socket: &UdpSocket,
    ) -> Result<()> {
        match packet.packet_type {
            PacketType::Message => {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct MsgPayload {
                    #[serde(rename = "messageId")]
                    message_id: Uuid,
                    sender: String,
                    recipient: String,
                    #[serde(rename = "encryptedPayload")]
                    encrypted_payload: String,
                    timestamp: String,
                }
                let msg_payload: MsgPayload = serde_json::from_str(&packet.encrypted_payload)?;

                let secure_sessions = load_secure_sessions()?;
                let secure_session = secure_sessions
                    .iter()
                    .find(|s| s.peer.eq_ignore_ascii_case(&msg_payload.sender))
                    .ok_or_else(|| {
                        anyhow!("No secure session found for peer {}", msg_payload.sender)
                    })?;

                let key = get_secure_session_key(&secure_session.session_id);

                let encrypted_bytes = BASE64_STANDARD.decode(&msg_payload.encrypted_payload)?;
                if encrypted_bytes.len() < 12 {
                    return Err(anyhow!("Invalid encrypted payload length."));
                }
                let (nonce, ciphertext) = encrypted_bytes.split_at(12);
                let nonce_arr: &[u8; 12] = nonce.try_into()?;

                let decrypted_bytes =
                    crate::secure::exchange::decrypt(&key, nonce_arr, ciphertext)?;
                let plaintext = String::from_utf8(decrypted_bytes)?;

                let message = Message {
                    id: msg_payload.message_id,
                    direction: Direction::Incoming,
                    timestamp: Utc::now(),
                    status: MessageStatus::Delivered,
                    message_type: MessageType::Text,
                    content: plaintext,
                    signature: None,
                    reply_to: None,
                };
                append_message(&msg_payload.sender, message)?;

                // Reply with MESSAGE_ACK
                let ack_payload = serde_json::json!({
                    "messageId": msg_payload.message_id,
                });
                let ack_pkt = Packet {
                    version: 1,
                    packet_type: PacketType::MessageAck,
                    message_id: Uuid::new_v4(),
                    sender: msg_payload.recipient.clone(),
                    recipient: msg_payload.sender.clone(),
                    timestamp: Utc::now(),
                    nonce: Uuid::new_v4().to_string(),
                    encrypted_payload: ack_payload.to_string(),
                    signature: "message-ack".to_string(),
                };
                let encoded = ack_pkt.encode()?;
                let _ = socket.send_to(encoded.as_bytes(), from_addr).await;
            }
            PacketType::MessageAck => {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct AckPayload {
                    #[serde(rename = "messageId")]
                    message_id: Uuid,
                }
                if let Ok(ack) = serde_json::from_str::<AckPayload>(&packet.encrypted_payload) {
                    let _ = update_message_status(
                        &packet.sender,
                        ack.message_id,
                        MessageStatus::Delivered,
                    );
                }
            }
            PacketType::MessageRead => {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct ReadPayload {
                    #[serde(rename = "messageId")]
                    message_id: Uuid,
                }
                if let Ok(read_p) = serde_json::from_str::<ReadPayload>(&packet.encrypted_payload) {
                    let _ = update_message_status(
                        &packet.sender,
                        read_p.message_id,
                        MessageStatus::Read,
                    );
                }
            }
            PacketType::ConnectivityCheck => {
                let current_session = crate::session::manager::get_current_session()?;
                let resp = Packet {
                    version: 1,
                    packet_type: PacketType::ConnectivityResponse,
                    message_id: Uuid::new_v4(),
                    sender: current_session.username.clone(),
                    recipient: packet.sender.clone(),
                    timestamp: Utc::now(),
                    nonce: Uuid::new_v4().to_string(),
                    encrypted_payload: String::new(),
                    signature: "connectivity-response".to_string(),
                };
                if let Ok(resp_payload) = resp.encode() {
                    let _ = socket.send_to(resp_payload.as_bytes(), from_addr).await;
                }
            }
            PacketType::PunchProbe => {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct ProbePayload {
                    #[serde(rename = "sessionId")]
                    session_id: String,
                }
                if let Ok(p) = serde_json::from_str::<ProbePayload>(&packet.encrypted_payload) {
                    let current_session = crate::session::manager::get_current_session()?;
                    if let Ok(ack) = crate::punch::probe::build_ack_packet(
                        &p.session_id,
                        &current_session.username,
                        &packet.sender,
                    ) {
                        if let Ok(encoded) = ack.encode() {
                            let _ = socket.send_to(encoded.as_bytes(), from_addr).await;
                        }
                    }
                }
            }
            PacketType::PunchAck => {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct AckPayload {
                    #[serde(rename = "sessionId")]
                    session_id: String,
                }
                if let Ok(_p) = serde_json::from_str::<AckPayload>(&packet.encrypted_payload) {
                    if let Ok(mut sessions) = load_punch_sessions() {
                        if let Some(session) = sessions
                            .iter_mut()
                            .find(|s| s.peer.eq_ignore_ascii_case(&packet.sender))
                        {
                            session.state = crate::punch::state::PunchState::Established;
                            let _ = crate::punch::session::save_punch_sessions(&sessions);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
