use crate::network::candidate::{CandidateType, IceCandidate};
use crate::network::interfaces::get_local_interfaces;
use crate::network::nat::{NatType, NetworkInfo};
use crate::network::stun::discover_public_endpoint;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

pub struct DiscoveryManager;

fn get_candidates_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("candidates.json"))
}

impl DiscoveryManager {
    pub fn save_candidates(candidates: &[IceCandidate]) -> Result<()> {
        let path = get_candidates_path()?;
        let content = serde_json::to_string_pretty(candidates)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_candidates() -> Result<Vec<IceCandidate>> {
        let path = get_candidates_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(path)?;
        let candidates: Vec<IceCandidate> = serde_json::from_str(&content)?;
        Ok(candidates)
    }

    pub fn get_candidates() -> Result<Vec<IceCandidate>> {
        Self::load_candidates()
    }

    pub async fn discover() -> Result<NetworkInfo> {
        let stun_server = "stun.l.google.com:19302";
        let mut candidates = Vec::new();

        let local_ips = match get_local_interfaces() {
            Ok(ips) => ips,
            Err(e) => {
                eprintln!("Failed to discover local interfaces: {}", e);
                Vec::new()
            }
        };

        let mut foundation_idx = 1;
        for ip in &local_ips {
            candidates.push(IceCandidate {
                foundation: foundation_idx.to_string(),
                component: 1,
                transport: "UDP".to_string(),
                priority: 2130706431,
                address: ip.clone(),
                port: 5000,
                candidate_type: CandidateType::Host,
            });
            foundation_idx += 1;
        }

        let primary_local_ip = local_ips
            .first()
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let mut public_ip = "0.0.0.0".to_string();
        let mut public_port = 0;

        match discover_public_endpoint(stun_server).await {
            Ok((ip, port)) => {
                public_ip = ip.clone();
                public_port = port;

                candidates.push(IceCandidate {
                    foundation: foundation_idx.to_string(),
                    component: 1,
                    transport: "UDP".to_string(),
                    priority: 1686052863,
                    address: ip,
                    port,
                    candidate_type: CandidateType::ServerReflexive,
                });
            }
            Err(_) => {
                // Graceful handling: fail silently on network issues and proceed with host candidates.
            }
        }

        Self::save_candidates(&candidates)?;

        Ok(NetworkInfo {
            local_ip: primary_local_ip,
            public_ip,
            public_port,
            nat_type: NatType::Unknown,
            stun_server: stun_server.to_string(),
        })
    }
}
