use crate::storage::filesystem::get_storage_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PeerCapabilities {
    #[serde(rename = "supportsIce")]
    pub supports_ice: bool,
    #[serde(rename = "supportsTurn")]
    pub supports_turn: bool,
    #[serde(rename = "supportsFiles")]
    pub supports_files: bool,
    #[serde(rename = "supportsImages")]
    pub supports_images: bool,
    #[serde(rename = "maxFileSizeMb")]
    pub max_file_size_mb: u32,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
}

impl Default for PeerCapabilities {
    fn default() -> Self {
        Self {
            supports_ice: true,
            supports_turn: false,
            supports_files: true,
            supports_images: true,
            max_file_size_mb: 50,
            client_version: "0.5.0".to_string(),
        }
    }
}

fn get_capabilities_path() -> Result<PathBuf> {
    Ok(get_storage_dir()?.join("capabilities.json"))
}

pub fn load_local_capabilities() -> Result<PeerCapabilities> {
    let path = get_capabilities_path()?;
    if !path.exists() {
        let default_caps = PeerCapabilities::default();
        save_local_capabilities(&default_caps)?;
        return Ok(default_caps);
    }
    let content = fs::read_to_string(path)?;
    let caps: PeerCapabilities = serde_json::from_str(&content)?;
    Ok(caps)
}

pub fn save_local_capabilities(caps: &PeerCapabilities) -> Result<()> {
    let path = get_capabilities_path()?;
    let content = serde_json::to_string_pretty(caps)?;
    fs::write(path, content)?;
    Ok(())
}
