use crate::registry::api::RegistryClient;
use crate::storage::filesystem::{read_profile, write_profile};
use anyhow::{Result, anyhow};
use std::io::{self, Write};

pub async fn exec(new_username: &str) -> Result<()> {
    let mut profile = read_profile()?;
    let current_username = match &profile.username {
        Some(name) if !name.is_empty() => name.clone(),
        _ => return Err(anyhow!("No registered identity found.")),
    };

    let client = RegistryClient::new(profile.registry_url.clone());

    print!("Enter recovery code: ");
    io::stdout().flush()?;
    let mut recovery_code = String::new();
    io::stdin().read_line(&mut recovery_code)?;
    let recovery_code = recovery_code.trim().to_string();

    if recovery_code.is_empty() {
        return Err(anyhow!("Invalid recovery code."));
    }

    let response = client
        .rename_account(&current_username, new_username, &recovery_code)
        .await?;

    if !response.success {
        return Err(anyhow!("Rename failed."));
    }

    profile.username = Some(response.username.clone());
    write_profile(&profile)?;

    println!("\nUsername successfully changed.");
    println!("\nOld Username: {}", current_username);
    println!("New Username: {}", response.username);

    Ok(())
}
