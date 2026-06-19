use crate::messaging::manager::MessageManager;
use anyhow::Result;

pub fn exec() -> Result<()> {
    let unread = MessageManager::get_unread_messages()?;
    if unread.is_empty() {
        println!("No unread messages.");
        return Ok(());
    }

    for (username, msg) in unread {
        let ts_str = msg.timestamp.format("%Y-%m-%d %H:%M").to_string();
        println!("[{}] {}: {}", ts_str, username, msg.content);
    }
    Ok(())
}
