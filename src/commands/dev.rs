use crate::chat::simulator::inject_incoming_message;
use anyhow::Result;

pub fn exec_inject(username: &str, text: &str) -> Result<()> {
    let msg = inject_incoming_message(username, text)?;
    println!("✓ Injected incoming message from '{}' locally.", username);
    println!("  Message: {}", msg.content);
    Ok(())
}
