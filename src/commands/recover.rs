use crate::crypto::keys::generate_keypair;
use crate::registry::api::RegistryClient;
use crate::storage::filesystem::{read_profile, write_keys, write_profile};
use anyhow::{Result, anyhow};
use std::io::{self, Write};

pub async fn exec(username: &str) -> Result<()> {
    let mut profile = read_profile().unwrap_or_default();
    let client = RegistryClient::new(profile.registry_url.clone());

    print!("Enter recovery code: ");
    io::stdout().flush()?;
    let mut recovery_code = String::new();
    io::stdin().read_line(&mut recovery_code)?;
    let recovery_code = recovery_code.trim().to_string();

    if recovery_code.is_empty() {
        return Err(anyhow!("Invalid recovery code."));
    }

    let keys = generate_keypair();

    let response = client
        .recover_account(username, &recovery_code, &keys.public_key_hex)
        .await?;

    if !response.success {
        return Err(anyhow!("Recovery failed."));
    }

    // Overwrite local keys and update profile
    write_keys(&keys.private_key_hex, &keys.public_key_hex)?;
    profile.username = Some(username.to_string());
    profile.fingerprint = Some(response.fingerprint.clone());
    write_profile(&profile)?;

    println!("\nAccount recovered successfully.");
    println!("\nIMPORTANT:");
    println!("Your old private key is no longer valid.");
    println!("\nNew Fingerprint: {}", response.fingerprint);

    Ok(())
}
