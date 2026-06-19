use crate::chat::conversation::{clear_conversation, list_conversations};
use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::append_message;
use crate::contacts::manager::get_contact;
use crate::contacts::models::TrustLevel;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::io::{self, Write};
use uuid::Uuid;

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn exec_send(username: &str, text: &str) -> Result<()> {
    // 1. Verify contact exists
    let contact = get_contact(username)?;

    // 2. Reject if contact trustLevel == BLOCKED
    if contact.trust_level == TrustLevel::Blocked {
        return Err(anyhow!(
            "Cannot send messages to blocked contact '{}'.",
            contact.username
        ));
    }

    // Message Rules: 1-4000 characters
    let length = text.chars().count();
    if length == 0 {
        return Err(anyhow!("Message content cannot be empty."));
    }
    if length > 4000 {
        return Err(anyhow!(
            "Message content exceeds maximum length of 4000 characters."
        ));
    }

    // Create local outgoing message
    let msg = Message {
        id: Uuid::new_v4(),
        direction: Direction::Outgoing,
        timestamp: Utc::now(),
        status: MessageStatus::Pending,
        message_type: MessageType::Text,
        content: text.to_string(),
        signature: None,
        reply_to: None,
    };

    // Save message
    append_message(&contact.username, msg)?;
    println!("Message queued locally.");
    Ok(())
}

pub fn exec_history(username: &str) -> Result<()> {
    // Verify contact exists
    let contact = get_contact(username)?;

    let messages = crate::chat::storage::load_messages(&contact.username)?;
    let sender_label = capitalize(&contact.username);

    for m in messages {
        let ts_str = m.timestamp.format("%Y-%m-%d %H:%M").to_string();
        let label = match m.direction {
            Direction::Outgoing => "You".to_string(),
            Direction::Incoming => sender_label.clone(),
        };
        println!("[{}] {}: {}", ts_str, label, m.content);
    }
    Ok(())
}

pub fn exec_list() -> Result<()> {
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

pub fn exec_clear(username: &str) -> Result<()> {
    // Verify contact exists
    let contact = get_contact(username)?;

    println!("Delete entire conversation?");
    print!("Type YES to continue: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim() == "YES" {
        clear_conversation(&contact.username)?;
        println!("✓ Conversation with '{}' cleared.", contact.username);
    } else {
        println!("Clear operation cancelled.");
    }
    Ok(())
}
