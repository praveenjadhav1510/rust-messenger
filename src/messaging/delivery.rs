use crate::protocol::packet::{Packet, PacketType};
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::timeout;
use uuid::Uuid;

pub struct DeliveryManager;

impl DeliveryManager {
    pub async fn send_with_retry(
        socket: &UdpSocket,
        remote_addr: std::net::SocketAddr,
        packet: &Packet,
        message_id: Uuid,
    ) -> Result<bool> {
        let encoded = packet.encode()?;
        let backoffs = [
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
        ];

        let mut buf = vec![0u8; 65535];

        for &timeout_dur in &backoffs {
            socket.send_to(encoded.as_bytes(), remote_addr).await?;

            let start = Instant::now();
            while start.elapsed() < timeout_dur {
                let remaining = timeout_dur
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::from_secs(0));
                if remaining.as_millis() == 0 {
                    break;
                }

                let recv_fut = socket.recv_from(&mut buf);
                match timeout(remaining, recv_fut).await {
                    Ok(Ok((len, _from_addr))) => {
                        if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(pkt) = Packet::decode(payload_str) {
                                if pkt.packet_type == PacketType::MessageAck {
                                    #[derive(serde::Deserialize)]
                                    #[serde(rename_all = "camelCase")]
                                    struct AckPayload {
                                        #[serde(rename = "messageId")]
                                        message_id: Uuid,
                                    }
                                    if let Ok(ack) =
                                        serde_json::from_str::<AckPayload>(&pkt.encrypted_payload)
                                    {
                                        if ack.message_id == message_id {
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(false)
    }
}
