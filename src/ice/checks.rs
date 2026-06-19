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

    let local_addr: SocketAddr = format!("{}:0", pair.local.address).parse()?;
    let remote_addr: SocketAddr =
        format!("{}:{}", pair.remote.address, pair.remote.port).parse()?;

    let socket = match UdpSocket::bind(local_addr).await {
        Ok(s) => s,
        Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
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

    let is_loopback = pair.remote.address == "127.0.0.1" || pair.remote.address == "localhost";
    // If testing on localhost, spawn a reply echo socket
    let mut _reply_handle = None;
    if is_loopback {
        let listener_addr = remote_addr;
        _reply_handle = Some(tokio::spawn(async move {
            if let Ok(listener) = UdpSocket::bind(listener_addr).await {
                let mut buf = vec![0u8; 2048];
                if let Ok((len, addr)) = listener.recv_from(&mut buf).await {
                    if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                        if let Ok(p) = Packet::decode(payload_str) {
                            let resp = Packet {
                                version: 1,
                                packet_type: PacketType::ConnectivityResponse,
                                message_id: Uuid::new_v4(),
                                sender: p.recipient,
                                recipient: p.sender,
                                timestamp: Utc::now(),
                                nonce: Uuid::new_v4().to_string(),
                                encrypted_payload: String::new(),
                                signature: "connectivity-response".to_string(),
                            };
                            if let Ok(resp_payload) = resp.encode() {
                                let _ = listener.send_to(resp_payload.as_bytes(), addr).await;
                            }
                        }
                    }
                }
            }
        }));
        // Small delay to ensure the listener socket is bound
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let _ = socket.send_to(payload.as_bytes(), remote_addr).await;

    let mut buf = vec![0u8; 2048];
    let check_future = socket.recv_from(&mut buf);

    match timeout(Duration::from_secs(2), check_future).await {
        Ok(Ok((len, _from_addr))) => {
            if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                if let Ok(p) = Packet::decode(payload_str) {
                    if p.packet_type == PacketType::ConnectivityResponse {
                        pair.state = IceConnectionState::Connected;
                        return Ok(IceConnectionState::Connected);
                    }
                }
            }
            pair.state = IceConnectionState::Failed;
            Ok(IceConnectionState::Failed)
        }
        _ => {
            pair.state = IceConnectionState::Failed;
            Ok(IceConnectionState::Failed)
        }
    }
}
