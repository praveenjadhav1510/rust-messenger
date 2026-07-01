use crate::ice::pair::CandidatePair;
use crate::ice::state::IceConnectionState;
use crate::protocol::packet::{Packet, PacketType};
use anyhow::Result;
use chrono::Utc;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use uuid::Uuid;

pub async fn validate_pair(
    pair: &mut CandidatePair,
    sender_name: &str,
    recipient_name: &str,
) -> Result<IceConnectionState> {
    pair.state = IceConnectionState::Checking;

    let is_loopback = pair.remote.address == "127.0.0.1"
        || pair.remote.address == "localhost"
        || pair.local.address == "127.0.0.1"
        || pair.local.address == "localhost";

    let bind_port = if is_loopback {
        if sender_name.to_lowercase() < recipient_name.to_lowercase() {
            5001
        } else {
            5002
        }
    } else {
        5000
    };

    let local_addr = format!("0.0.0.0:{}", bind_port);
    let socket = match UdpSocket::bind(&local_addr).await {
        Ok(s) => s,
        Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
    };

    let remote_addr: SocketAddr = if is_loopback {
        let remote_port = if sender_name.to_lowercase() < recipient_name.to_lowercase() {
            5002
        } else {
            5001
        };
        format!("127.0.0.1:{}", remote_port).parse()?
    } else {
        format!("{}:{}", pair.remote.address, pair.remote.port).parse()?
    };

    let check_packet = Packet {
        version: 1,
        packet_type: PacketType::ConnectivityCheck,
        message_id: Uuid::new_v4(),
        sender: sender_name.to_string(),
        recipient: recipient_name.to_string(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: String::new(),
        signature: "connectivity-check".to_string(),
    };

    let payload = check_packet.encode()?;

    let start = std::time::Instant::now();
    let timeout_dur = Duration::from_secs(5);

    while start.elapsed() < timeout_dur {
        // Send our check
        let _ = socket.send_to(payload.as_bytes(), remote_addr).await;

        let mut buf = vec![0u8; 2048];
        let check_future = socket.recv_from(&mut buf);

        if let Ok(Ok((len, from_addr))) = timeout(Duration::from_millis(500), check_future).await {
            if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                if let Ok(p) = Packet::decode(payload_str) {
                    if p.packet_type == PacketType::ConnectivityResponse {
                        pair.state = IceConnectionState::Connected;
                        return Ok(IceConnectionState::Connected);
                    } else if p.packet_type == PacketType::ConnectivityCheck {
                        // Respond to peer check
                        let resp = Packet {
                            version: 1,
                            packet_type: PacketType::ConnectivityResponse,
                            message_id: Uuid::new_v4(),
                            sender: recipient_name.to_string(),
                            recipient: sender_name.to_string(),
                            timestamp: Utc::now(),
                            nonce: Uuid::new_v4().to_string(),
                            encrypted_payload: String::new(),
                            signature: "connectivity-response".to_string(),
                        };
                        if let Ok(resp_payload) = resp.encode() {
                            let _ = socket.send_to(resp_payload.as_bytes(), from_addr).await;
                        }
                        pair.state = IceConnectionState::Connected;
                        return Ok(IceConnectionState::Connected);
                    }
                }
            }
        }
    }

    pair.state = IceConnectionState::Failed;
    Ok(IceConnectionState::Failed)
}
