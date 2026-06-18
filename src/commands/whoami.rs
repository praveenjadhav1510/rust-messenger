use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub fn exec() -> Result<()> {
    let profile = read_profile()?;

    let username = profile.username.as_deref().unwrap_or("Not Registered");
    let fingerprint = profile.fingerprint.as_deref().unwrap_or("N/A");

    println!("Username: {}", username);
    println!("Fingerprint: {}", fingerprint);
    println!("Registry: {}", profile.registry_url);

    Ok(())
}
