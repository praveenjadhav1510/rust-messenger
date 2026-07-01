use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::{append_message, load_messages};
use crate::messaging::delivery::DeliveryManager;
use crate::messaging::listener::get_secure_session_key;
use crate::protocol::packet::{Packet, PacketType};
use crate::punch::session::load_punch_sessions;
use crate::secure::session::load_secure_sessions;
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::Utc;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use uuid::Uuid;

pub struct MessageSender;

impl MessageSender {
    pub fn queue_message(peer: &str, content: &str) -> Result<Message> {
        let msg = Message {
            id: Uuid::new_v4(),
            direction: Direction::Outgoing,
            timestamp: Utc::now(),
            status: MessageStatus::Queued,
            message_type: MessageType::Text,
            content: content.to_string(),
            signature: None,
            reply_to: None,
        };
        append_message(peer, msg.clone())?;
        Ok(msg)
    }

    pub async fn send_message(peer: &str, content: &str) -> Result<Message> {
        let secure_sessions = load_secure_sessions()?;
        let secure_session = secure_sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .ok_or_else(|| anyhow!("No secure session found with peer '{}'.", peer))?;

        if !secure_session.established {
            return Err(anyhow!("Secure session with '{}' not established.", peer));
        }

        let punch_sessions = load_punch_sessions()?;
        let punch_session = punch_sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .ok_or_else(|| anyhow!("No punch session found with peer '{}'.", peer))?;

        if punch_session.state != crate::punch::state::PunchState::Established {
            return Err(anyhow!(
                "UDP punch connection with '{}' is not established.",
                peer
            ));
        }

        let mut msg = Self::queue_message(peer, content)?;

        let key = get_secure_session_key(&secure_session.session_id);
        let nonce = rand::random::<[u8; 12]>();
        let ciphertext = crate::secure::exchange::encrypt(&key, &nonce, content.as_bytes())?;

        let mut encrypted_bytes = vec![0u8; 12];
        encrypted_bytes.copy_from_slice(&nonce);
        encrypted_bytes.extend_from_slice(&ciphertext);

        let b64_payload = BASE64_STANDARD.encode(&encrypted_bytes);

        let current_session = crate::session::manager::get_current_session()?;
        let payload = serde_json::json!({
            "messageId": msg.id,
            "sender": current_session.username,
            "recipient": peer,
            "encryptedPayload": b64_payload,
            "timestamp": msg.timestamp.to_rfc3339(),
        });

        let packet = Packet {
            version: 1,
            packet_type: PacketType::Message,
            message_id: Uuid::new_v4(),
            sender: current_session.username.clone(),
            recipient: peer.to_string(),
            timestamp: Utc::now(),
            nonce: Uuid::new_v4().to_string(),
            encrypted_payload: payload.to_string(),
            signature: "message-signature".to_string(),
        };

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

        let remote_addr: SocketAddr = format!(
            "{}:{}",
            punch_session.selected_pair.remote.address, remote_port
        )
        .parse()?;

        let bind_addr = format!("0.0.0.0:{}", local_port);
        let socket = match UdpSocket::bind(&bind_addr).await {
            Ok(s) => s,
            Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
        };

        let ack_received =
            DeliveryManager::send_with_retry(&socket, remote_addr, &packet, msg.id).await?;

        if ack_received {
            msg.status = MessageStatus::Delivered;
            let _ = crate::messaging::receipts::update_message_status(
                peer,
                msg.id,
                MessageStatus::Delivered,
            );
        } else {
            msg.status = MessageStatus::Failed;
            let _ = crate::messaging::receipts::update_message_status(
                peer,
                msg.id,
                MessageStatus::Failed,
            );
        }

        Ok(msg)
    }

    pub async fn retry_failed(peer: &str) -> Result<Vec<Message>> {
        let messages = load_messages(peer)?;
        let mut retried = Vec::new();

        let failed_ids: Vec<Uuid> = messages
            .iter()
            .filter(|m| m.direction == Direction::Outgoing && m.status == MessageStatus::Failed)
            .map(|m| m.id)
            .collect();

        for id in failed_ids {
            let messages_now = load_messages(peer)?;
            if let Some(msg) = messages_now.iter().find(|m| m.id == id) {
                let _ = crate::messaging::receipts::update_message_status(
                    peer,
                    id,
                    MessageStatus::Queued,
                );

                let result = Self::resend_message(peer, id, &msg.content).await;
                if let Ok(m) = result {
                    retried.push(m);
                }
            }
        }
        Ok(retried)
    }

    async fn resend_message(peer: &str, message_id: Uuid, content: &str) -> Result<Message> {
        let secure_sessions = load_secure_sessions()?;
        let secure_session = secure_sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .ok_or_else(|| anyhow!("No secure session found with peer '{}'.", peer))?;

        let punch_sessions = load_punch_sessions()?;
        let punch_session = punch_sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .ok_or_else(|| anyhow!("No punch session found with peer '{}'.", peer))?;

        let current_session = crate::session::manager::get_current_session()?;

        let key = get_secure_session_key(&secure_session.session_id);
        let nonce = rand::random::<[u8; 12]>();
        let ciphertext = crate::secure::exchange::encrypt(&key, &nonce, content.as_bytes())?;

        let mut encrypted_bytes = vec![0u8; 12];
        encrypted_bytes.copy_from_slice(&nonce);
        encrypted_bytes.extend_from_slice(&ciphertext);

        let b64_payload = BASE64_STANDARD.encode(&encrypted_bytes);

        let payload = serde_json::json!({
            "messageId": message_id,
            "sender": current_session.username,
            "recipient": peer,
            "encryptedPayload": b64_payload,
            "timestamp": Utc::now().to_rfc3339(),
        });

        let packet = Packet {
            version: 1,
            packet_type: PacketType::Message,
            message_id: Uuid::new_v4(),
            sender: current_session.username.clone(),
            recipient: peer.to_string(),
            timestamp: Utc::now(),
            nonce: Uuid::new_v4().to_string(),
            encrypted_payload: payload.to_string(),
            signature: "message-signature".to_string(),
        };

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

        let remote_addr: SocketAddr = format!(
            "{}:{}",
            punch_session.selected_pair.remote.address, remote_port
        )
        .parse()?;

        let bind_addr = format!("0.0.0.0:{}", local_port);
        let socket = match UdpSocket::bind(&bind_addr).await {
            Ok(s) => s,
            Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
        };

        let ack_received =
            DeliveryManager::send_with_retry(&socket, remote_addr, &packet, message_id).await?;

        let status = if ack_received {
            MessageStatus::Delivered
        } else {
            MessageStatus::Failed
        };

        let _ = crate::messaging::receipts::update_message_status(peer, message_id, status);

        Ok(Message {
            id: message_id,
            direction: Direction::Outgoing,
            timestamp: Utc::now(),
            status,
            message_type: MessageType::Text,
            content: content.to_string(),
            signature: None,
            reply_to: None,
        })
    }
}
