use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::{Result, anyhow};
use std::io::{self, Write};

pub async fn exec() -> Result<()> {
    let profile = read_profile()?;
    let username = match &profile.username {
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

    let response = client.lock_account(&username, &recovery_code).await?;

    if !response.success {
        return Err(anyhow!("Lock failed."));
    }

    println!("\nAccount locked.");
    println!(
        "\nOther users should not trust or communicate with this identity until recovery is completed."
    );

    Ok(())
}
