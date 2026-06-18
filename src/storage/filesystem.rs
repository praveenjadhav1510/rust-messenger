use crate::config::profile::Profile;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

pub fn get_storage_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".rust-messenger"))
        .ok_or_else(|| anyhow!("Could not resolve home directory"))
}

pub fn ensure_storage_dir() -> Result<PathBuf> {
    let dir = get_storage_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

#[allow(dead_code)]
pub fn read_private_key() -> Result<String> {
    let path = get_storage_dir()?.join("private.key");
    if !path.exists() {
        return Err(anyhow!(
            "Identity not initialized. Run 'rust-messenger init' first."
        ));
    }
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

pub fn read_public_key() -> Result<String> {
    let path = get_storage_dir()?.join("public.key");
    if !path.exists() {
        return Err(anyhow!(
            "Identity not initialized. Run 'rust-messenger init' first."
        ));
    }
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

pub fn read_profile() -> Result<Profile> {
    let path = get_storage_dir()?.join("profile.json");
    if !path.exists() {
        return Err(anyhow!(
            "Identity not initialized. Run 'rust-messenger init' first."
        ));
    }
    let content = fs::read_to_string(path)?;
    let profile: Profile = serde_json::from_str(&content)?;
    Ok(profile)
}

pub fn write_profile(profile: &Profile) -> Result<()> {
    let path = get_storage_dir()?.join("profile.json");
    let content = serde_json::to_string_pretty(profile)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn initialize_storage(private_key_hex: &str, public_key_hex: &str, force: bool) -> Result<()> {
    let dir = ensure_storage_dir()?;

    let private_path = dir.join("private.key");
    let public_path = dir.join("public.key");
    let profile_path = dir.join("profile.json");
    let contacts_path = dir.join("contacts.json");

    if !force && (private_path.exists() || public_path.exists() || profile_path.exists()) {
        return Err(anyhow!(
            "Identity already exists. Use --force to overwrite."
        ));
    }

    println!("Generating identity...\n");

    fs::write(&private_path, private_key_hex)?;
    println!("✓ Private key generated");

    fs::write(&public_path, public_key_hex)?;
    println!("✓ Public key generated");

    let profile = Profile::default();
    let profile_content = serde_json::to_string_pretty(&profile)?;
    fs::write(&profile_path, profile_content)?;
    println!("✓ Profile created");

    if force || !contacts_path.exists() {
        fs::write(&contacts_path, "[]")?;
        println!("✓ Contacts file created");
    }

    Ok(())
}
