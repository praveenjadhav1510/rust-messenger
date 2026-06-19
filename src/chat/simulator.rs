use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::append_message;
use crate::contacts::manager::get_contact;
use crate::contacts::models::TrustLevel;
use anyhow::{Result, anyhow};
use chrono::Utc;
use uuid::Uuid;

pub fn inject_incoming_message(username: &str, content: &str) -> Result<Message> {
    // Verify contact exists
    let contact = get_contact(username)?;

    // Reject if contact is blocked
    if contact.trust_level == TrustLevel::Blocked {
        return Err(anyhow!(
            "Cannot receive messages from blocked contact '{}'.",
            contact.username
        ));
    }

    // Message Rules: 1-4000 characters
    let length = content.chars().count();
    if length == 0 {
        return Err(anyhow!("Message content cannot be empty."));
    }
    if length > 4000 {
        return Err(anyhow!(
            "Message content exceeds maximum length of 4000 characters."
        ));
    }

    let msg = Message {
        id: Uuid::new_v4(),
        direction: Direction::Incoming,
        timestamp: Utc::now(),
        status: MessageStatus::Delivered,
        message_type: MessageType::Text,
        content: content.to_string(),
        signature: None,
        reply_to: None,
    };

    append_message(&contact.username, msg.clone())?;
    Ok(msg)
}
