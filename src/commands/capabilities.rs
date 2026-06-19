use crate::peer::capabilities::load_local_capabilities;
use anyhow::Result;

pub fn exec() -> Result<()> {
    let caps = load_local_capabilities()?;
    println!("Local Capabilities:");
    println!();
    println!("ICE: {}", caps.supports_ice);
    println!("TURN: {}", caps.supports_turn);
    println!("Files: {}", caps.supports_files);
    println!("Images: {}", caps.supports_images);
    println!("Max File Size: {} MB", caps.max_file_size_mb);
    println!("Client Version: {}", caps.client_version);
    Ok(())
}
