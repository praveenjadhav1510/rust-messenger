use crate::registry::api::RegistryClient;
use crate::storage::filesystem::{read_profile, write_profile};
use anyhow::{Result, anyhow};
use std::io::{self, Write};

pub async fn exec() -> Result<()> {
    let mut profile = read_profile()?;
    let username = match &profile.username {
        Some(name) if !name.is_empty() => name.clone(),
        _ => return Err(anyhow!("No registered identity found.")),
    };

    let client = RegistryClient::new(profile.registry_url.clone());

    println!("WARNING:");
    println!("This account will be deactivated.\n");

    print!("Type DELETE to continue: ");
    io::stdout().flush()?;
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    let confirmation = confirmation.trim();

    if confirmation != "DELETE" {
        return Err(anyhow!("Deletion cancelled."));
    }

    print!("Enter recovery code: ");
    io::stdout().flush()?;
    let mut recovery_code = String::new();
    io::stdin().read_line(&mut recovery_code)?;
    let recovery_code = recovery_code.trim().to_string();

    if recovery_code.is_empty() {
        return Err(anyhow!("Invalid recovery code."));
    }

    let response = client.remove_account(&username, &recovery_code).await?;

    if !response.success {
        return Err(anyhow!("Account removal failed."));
    }

    profile.username = None;
    profile.fingerprint = None;
    write_profile(&profile)?;

    println!("\nAccount scheduled for deletion.");
    println!("\nYour username will remain reserved for 30 days.");

    Ok(())
}
