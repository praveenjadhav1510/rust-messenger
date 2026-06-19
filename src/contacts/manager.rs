use crate::contacts::models::{Contact, TrustLevel};
use crate::registry::api::RegistryClient;
use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::fs;

pub fn load_contacts() -> Result<Vec<Contact>> {
    let path = get_storage_dir()?.join("contacts.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let contacts: Vec<Contact> = serde_json::from_str(&content)?;
    Ok(contacts)
}

pub fn save_contacts(contacts: &[Contact]) -> Result<()> {
    let path = get_storage_dir()?.join("contacts.json");
    let content = serde_json::to_string_pretty(contacts)?;
    fs::write(path, content)?;
    Ok(())
}

pub async fn add_contact(username: &str, registry_url: &str) -> Result<Contact> {
    let mut contacts = load_contacts()?;

    // Check duplicates
    if contacts
        .iter()
        .any(|c| c.username.eq_ignore_ascii_case(username))
    {
        return Err(anyhow!("Contact '{}' already exists.", username));
    }

    // Lookup user from registry
    let client = RegistryClient::new(registry_url.to_string());
    let lookup_res = match client.lookup_user(username).await {
        Ok(res) => res,
        Err(e) => return Err(anyhow!("Registry lookup failed: {}", e)),
    };

    let contact = Contact {
        username: lookup_res.username,
        public_key: lookup_res.public_key,
        fingerprint: lookup_res.fingerprint,
        trust_level: TrustLevel::Unverified,
        account_status: lookup_res.status,
        added_at: Utc::now(),
        notes: String::new(),
    };

    contacts.push(contact.clone());
    save_contacts(&contacts)?;

    Ok(contact)
}

pub fn remove_contact(username: &str) -> Result<()> {
    let mut contacts = load_contacts()?;
    let initial_len = contacts.len();
    contacts.retain(|c| !c.username.eq_ignore_ascii_case(username));

    if contacts.len() == initial_len {
        return Err(anyhow!("Contact '{}' not found.", username));
    }

    save_contacts(&contacts)?;
    Ok(())
}

pub fn get_contact(username: &str) -> Result<Contact> {
    let contacts = load_contacts()?;
    contacts
        .into_iter()
        .find(|c| c.username.eq_ignore_ascii_case(username))
        .ok_or_else(|| anyhow!("Contact '{}' not found.", username))
}

pub fn update_trust_level(username: &str, level: TrustLevel) -> Result<Contact> {
    let mut contacts = load_contacts()?;
    let mut updated_contact = None;

    for contact in &mut contacts {
        if contact.username.eq_ignore_ascii_case(username) {
            contact.trust_level = level;
            updated_contact = Some(contact.clone());
            break;
        }
    }

    if let Some(contact) = updated_contact {
        save_contacts(&contacts)?;
        Ok(contact)
    } else {
        Err(anyhow!("Contact '{}' not found.", username))
    }
}
