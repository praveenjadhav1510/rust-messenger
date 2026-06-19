use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::{append_message, load_messages};
use crate::storage::filesystem::get_storage_dir;
use anyhow::Result;
use chrono::Utc;
use std::fs;
use uuid::Uuid;

pub fn list_chats() -> Result<Vec<String>> {
    let chats_dir = get_storage_dir()?.join("chats");
    if !chats_dir.exists() {
        return Ok(Vec::new());
    }
    let mut chats = Vec::new();
    for entry in fs::read_dir(chats_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                chats.push(name.to_string());
            }
        }
    }
    Ok(chats)
}

pub fn get_chat_messages(username: &str) -> Result<Vec<Message>> {
    load_messages(username)
}

pub fn send_local_message(
    username: &str,
    content: &str,
    reply_to: Option<Uuid>,
) -> Result<Message> {
    let msg = Message {
        id: Uuid::new_v4(),
        direction: Direction::Outgoing,
        timestamp: Utc::now(),
        status: MessageStatus::Queued,
        message_type: MessageType::Text,
        content: content.to_string(),
        signature: None,
        reply_to,
    };
    append_message(username, msg.clone())?;
    Ok(msg)
}

pub fn receive_local_message(
    username: &str,
    content: &str,
    reply_to: Option<Uuid>,
) -> Result<Message> {
    let msg = Message {
        id: Uuid::new_v4(),
        direction: Direction::Incoming,
        timestamp: Utc::now(),
        status: MessageStatus::Read,
        message_type: MessageType::Text,
        content: content.to_string(),
        signature: None,
        reply_to,
    };
    append_message(username, msg.clone())?;
    Ok(msg)
}
