use crate::secure::bootstrap::bootstrap_secure_session;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let _session = bootstrap_secure_session(username)?;

    println!("Secure session established.");
    println!();
    println!("Encryption:");
    println!("ChaCha20-Poly1305");
    Ok(())
}
