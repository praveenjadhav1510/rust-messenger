use crate::chat::models::{Direction, Message, MessageStatus};
use crate::chat::storage::load_messages;
use crate::contacts::manager::load_contacts;
use anyhow::Result;

pub struct MessageManager;

impl MessageManager {
    pub fn get_unread_messages() -> Result<Vec<(String, Message)>> {
        let contacts = load_contacts()?;
        let mut unread = Vec::new();
        for contact in contacts {
            let messages = load_messages(&contact.username)?;
            for msg in messages {
                if msg.direction == Direction::Incoming && msg.status != MessageStatus::Read {
                    unread.push((contact.username.clone(), msg));
                }
            }
        }
        unread.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
        Ok(unread)
    }
}
