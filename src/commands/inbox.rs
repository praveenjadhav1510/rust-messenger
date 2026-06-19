use crate::chat::conversation::list_conversations;
use anyhow::Result;

pub fn exec() -> Result<()> {
    let conversations = list_conversations()?;
    if conversations.is_empty() {
        println!("No conversations found.");
        return Ok(());
    }

    for c in conversations {
        println!("{} ({} messages)", c.username, c.total_messages);
    }
    Ok(())
}
