use crate::ice::pair::CandidatePair;
use crate::protocol::packet::{Packet, PacketType};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct KeepaliveService {
    stop_tx: Option<oneshot::Sender<()>>,
    join_handle: Option<JoinHandle<()>>,
}

impl Default for KeepaliveService {
    fn default() -> Self {
        Self::new()
    }
}

impl KeepaliveService {
    pub fn new() -> Self {
        Self {
            stop_tx: None,
            join_handle: None,
        }
    }

    pub fn start(
        &mut self,
        working_pair: CandidatePair,
        sender: String,
        recipient: String,
    ) -> Result<()> {
        if self.join_handle.is_some() {
            return Err(anyhow!("Keepalive service already running."));
        }

        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        self.stop_tx = Some(stop_tx);

        let remote_addr: SocketAddr = format!(
            "{}:{}",
            working_pair.remote.address, working_pair.remote.port
        )
        .parse()?;

        let join_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(_) => return,
            };

            // First tick finishes immediately in Tokio, we can skip it or let it send.
            // Standard behavior is to tick.
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let packet = Packet {
                            version: 1,
                            packet_type: PacketType::Keepalive,
                            message_id: Uuid::new_v4(),
                            sender: sender.clone(),
                            recipient: recipient.clone(),
                            timestamp: Utc::now(),
                            nonce: Uuid::new_v4().to_string(),
                            encrypted_payload: String::new(),
                            signature: "keepalive".to_string(),
                        };
                        if let Ok(payload) = packet.encode() {
                            let _ = socket.send_to(payload.as_bytes(), remote_addr).await;
                        }
                    }
                    _ = &mut stop_rx => {
                        break;
                    }
                }
            }
        });

        self.join_handle = Some(join_handle);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.await;
        }
        Ok(())
    }
}
