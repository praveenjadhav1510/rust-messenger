use crate::crypto::keys::generate_keypair;
use crate::storage::filesystem::initialize_storage;
use anyhow::Result;

pub fn exec(force: bool) -> Result<()> {
    let keys = generate_keypair();
    initialize_storage(&keys.private_key_hex, &keys.public_key_hex, force)?;
    println!("\nIdentity initialized successfully");
    Ok(())
}
