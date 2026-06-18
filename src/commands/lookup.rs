use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url);

    let user = client.lookup_user(username).await?;

    println!("Username: {}", user.username);
    println!("Fingerprint: {}", user.fingerprint);
    println!("Status: {}", user.status);
    println!("Public Key: {}", user.public_key);
    Ok(())
}
