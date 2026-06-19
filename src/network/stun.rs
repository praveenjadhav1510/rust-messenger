use anyhow::{Result, anyhow};
use std::net::{ToSocketAddrs, UdpSocket};
use stunclient::StunClient;

pub async fn discover_public_endpoint(stun_server: &str) -> Result<(String, u16)> {
    let stun_server_str = stun_server.to_string();
    tokio::task::spawn_blocking(move || {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| anyhow!("Failed to bind local UDP socket: {}", e))?;

        let server_addr = stun_server_str
            .to_socket_addrs()
            .map_err(|e| anyhow!("Failed to resolve STUN server address: {}", e))?
            .next()
            .ok_or_else(|| anyhow!("STUN server address resolved to empty list."))?;

        let client = StunClient::new(server_addr);
        let public_addr = client
            .query_external_address(&socket)
            .map_err(|e| anyhow!("STUN query failed: {}", e))?;

        Ok((public_addr.ip().to_string(), public_addr.port()))
    })
    .await?
}
