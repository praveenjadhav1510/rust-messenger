use crate::registry::api::RegistryClient;
use crate::storage::filesystem::{read_profile, read_public_key, write_profile};
use anyhow::{Result, anyhow};

pub async fn exec(username: &str) -> Result<()> {
    let public_key = read_public_key()?;
    let mut profile = read_profile()?;

    let client = RegistryClient::new(profile.registry_url.clone());

    // Check if the username is available first
    if !client.check_username(username).await? {
        return Err(anyhow!("Username already exists."));
    }

    let response = client.register_user(username, &public_key).await?;

    if !response.success {
        return Err(anyhow!("Registration failed."));
    }

    profile.username = Some(response.username.clone());
    profile.fingerprint = Some(response.fingerprint.clone());
    write_profile(&profile)?;

    println!("Username Registered Successfully\n");
    println!("Username: {}", response.username);
    println!("Fingerprint: {}\n", response.fingerprint);
    println!("IMPORTANT:");
    println!("Save this recovery code:\n");
    println!("{}", response.recovery_code);
    println!("\nThe recovery code must only be displayed.");
    println!("Never store recovery codes locally.");
    println!("Never write recovery codes to disk.");

    Ok(())
}
