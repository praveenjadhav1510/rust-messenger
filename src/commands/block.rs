use crate::contacts::manager::update_trust_level;
use crate::contacts::models::TrustLevel;
use anyhow::Result;

pub fn exec_block(username: &str) -> Result<()> {
    update_trust_level(username, TrustLevel::Blocked)?;
    println!("✓ Contact '{}' has been BLOCKED.", username);
    Ok(())
}

pub fn exec_unblock(username: &str) -> Result<()> {
    update_trust_level(username, TrustLevel::Unverified)?;
    println!(
        "✓ Contact '{}' has been UNBLOCKED (marked as UNVERIFIED).",
        username
    );
    Ok(())
}
