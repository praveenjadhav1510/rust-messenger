use crate::contacts::manager::update_trust_level;
use crate::contacts::models::TrustLevel;
use anyhow::Result;

pub fn exec_verify(username: &str) -> Result<()> {
    update_trust_level(username, TrustLevel::Verified)?;
    println!("✓ Contact '{}' has been marked as VERIFIED.", username);
    Ok(())
}

pub fn exec_unverify(username: &str) -> Result<()> {
    update_trust_level(username, TrustLevel::Unverified)?;
    println!("✓ Contact '{}' has been marked as UNVERIFIED.", username);
    Ok(())
}
