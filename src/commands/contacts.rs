use crate::contacts::manager::{add_contact, get_contact, load_contacts, remove_contact};
use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub async fn exec_add(username: &str) -> Result<()> {
    let profile = read_profile()?;
    println!("Adding contact '{}'...", username);

    match add_contact(username, &profile.registry_url).await {
        Ok(contact) => {
            println!("✓ Contact '{}' added successfully.", contact.username);
            println!("  Fingerprint: {}", contact.fingerprint);
            println!("  Trust Level: {}", contact.trust_level);
        }
        Err(e) => {
            println!("✗ Failed to add contact: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

pub fn exec_remove(username: &str) -> Result<()> {
    remove_contact(username)?;
    println!("✓ Contact '{}' removed locally.", username);
    Ok(())
}

pub fn exec_list() -> Result<()> {
    let contacts = load_contacts()?;
    if contacts.is_empty() {
        println!("No contacts found.");
        return Ok(());
    }

    println!(
        "{:<15} | {:<12} | {:<12} | {:<10} | {:<25}",
        "Username", "Fingerprint", "Trust Level", "Status", "Added At"
    );
    println!("{}", "-".repeat(84));
    for c in contacts {
        let added_at_str = c.added_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        println!(
            "{:<15} | {:<12} | {:<12} | {:<10} | {:<25}",
            c.username,
            c.fingerprint,
            c.trust_level.to_string(),
            c.account_status,
            added_at_str
        );
    }
    Ok(())
}

pub fn exec_show(username: &str) -> Result<()> {
    let contact = get_contact(username)?;

    println!("Username:       {}", contact.username);
    println!("Public Key:     {}", contact.public_key);
    println!("Fingerprint:    {}", contact.fingerprint);
    println!("Trust Level:    {}", contact.trust_level);
    println!("Account Status: {}", contact.account_status);
    println!(
        "Added At:       {}",
        contact.added_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("Notes:          {}", contact.notes);

    Ok(())
}
