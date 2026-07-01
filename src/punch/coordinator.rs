use crate::ice::pair::CandidatePair;
use crate::punch::probe::{build_ack_packet, build_probe_packet};
use crate::punch::session::{PunchSession, load_punch_sessions, save_punch_sessions};
use crate::punch::state::PunchState;
use anyhow::{Result, anyhow};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;

pub struct HolePunchCoordinator {
    pub session_id: String,
    pub sender: String,
    pub peer: String,
    pub selected_pair: CandidatePair,
    pub socket: Arc<UdpSocket>,
}

impl HolePunchCoordinator {
    pub async fn new(
        session_id: &str,
        sender: &str,
        peer: &str,
        selected_pair: CandidatePair,
    ) -> Result<Self> {
        let is_loopback = (selected_pair.local.address == "127.0.0.1"
            || selected_pair.local.address == "localhost")
            && (selected_pair.remote.address == "127.0.0.1"
                || selected_pair.remote.address == "localhost");

        let local_port = if is_loopback {
            if sender.to_lowercase() < peer.to_lowercase() {
                5001
            } else {
                5002
            }
        } else {
            5000
        };

        let local_addr = format!("0.0.0.0:{}", local_port);
        let socket = match UdpSocket::bind(&local_addr).await {
            Ok(s) => s,
            Err(_) => UdpSocket::bind("0.0.0.0:0").await?,
        };

        Ok(Self {
            session_id: session_id.to_string(),
            sender: sender.to_string(),
            peer: peer.to_string(),
            selected_pair,
            socket: Arc::new(socket),
        })
    }

    pub async fn start_punch(&self) -> Result<PunchSession> {
        let is_loopback = (self.selected_pair.local.address == "127.0.0.1"
            || self.selected_pair.local.address == "localhost")
            && (self.selected_pair.remote.address == "127.0.0.1"
                || self.selected_pair.remote.address == "localhost");

        let remote_port = if is_loopback {
            if self.sender.to_lowercase() < self.peer.to_lowercase() {
                5002
            } else {
                5001
            }
        } else {
            self.selected_pair.remote.port
        };

        let remote_addr: SocketAddr =
            format!("{}:{}", self.selected_pair.remote.address, remote_port).parse()?;

        println!("Starting UDP hole punching...");
        println!();

        let mut attempts = 0;
        let mut final_state = PunchState::Failed;

        for attempt in 1..=10 {
            attempts = attempt;
            println!("Attempt {}/10", attempt);

            self.send_probe(remote_addr).await?;

            let start = Instant::now();
            let timeout_dur = Duration::from_millis(500);
            let mut ack_received = false;

            while start.elapsed() < timeout_dur {
                let remaining = timeout_dur
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::from_secs(0));
                if remaining.as_millis() == 0 {
                    break;
                }
                if self.await_ack(remaining).await? {
                    ack_received = true;
                    break;
                }
            }

            if ack_received {
                final_state = PunchState::Established;
                // Send confirmation ACK back
                let ack = build_ack_packet(&self.session_id, &self.sender, &self.peer)?;
                let encoded = ack.encode()?;
                let _ = self.socket.send_to(encoded.as_bytes(), remote_addr).await;
                break;
            }
        }

        let punch_session = self.complete(attempts, final_state)?;
        Ok(punch_session)
    }

    pub async fn send_probe(&self, remote_addr: SocketAddr) -> Result<()> {
        let probe = build_probe_packet(&self.session_id, &self.sender, &self.peer)?;
        let encoded = probe.encode()?;
        self.socket.send_to(encoded.as_bytes(), remote_addr).await?;
        Ok(())
    }

    pub async fn await_ack(&self, timeout_dur: Duration) -> Result<bool> {
        let mut buf = vec![0u8; 65535];
        let res = tokio::time::timeout(timeout_dur, self.socket.recv_from(&mut buf)).await;
        match res {
            Ok(Ok((len, from_addr))) => {
                if let Ok(payload_str) = std::str::from_utf8(&buf[..len]) {
                    if let Ok(packet) = crate::protocol::packet::Packet::decode(payload_str) {
                        if packet.packet_type == crate::protocol::packet::PacketType::PunchAck {
                            return Ok(true);
                        } else if packet.packet_type
                            == crate::protocol::packet::PacketType::PunchProbe
                        {
                            // Reply with ACK
                            let ack = build_ack_packet(&self.session_id, &self.sender, &self.peer)?;
                            let encoded = ack.encode()?;
                            let _ = self.socket.send_to(encoded.as_bytes(), from_addr).await;
                            return Ok(true);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    pub fn complete(&self, attempts: usize, state: PunchState) -> Result<PunchSession> {
        let punch_session = PunchSession {
            session_id: self.session_id.clone(),
            peer: self.peer.clone(),
            selected_pair: self.selected_pair.clone(),
            state,
            attempts,
            started_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut sessions = load_punch_sessions()?;
        sessions.retain(|s| !s.peer.eq_ignore_ascii_case(&self.peer));
        sessions.push(punch_session.clone());
        save_punch_sessions(&sessions)?;

        Ok(punch_session)
    }
}
