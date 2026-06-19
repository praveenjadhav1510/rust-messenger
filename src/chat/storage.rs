use crate::chat::models::Message;
use crate::storage::filesystem::get_storage_dir;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn get_chat_dir(username: &str) -> Result<PathBuf> {
    let chats_dir = get_storage_dir()?.join("chats");
    Ok(chats_dir.join(username.to_lowercase()))
}

pub fn create_chat_if_missing(username: &str) -> Result<PathBuf> {
    let chat_dir = get_chat_dir(username)?;
    if !chat_dir.exists() {
        fs::create_dir_all(&chat_dir)?;
    }
    let messages_file = chat_dir.join("messages.json");
    if !messages_file.exists() {
        fs::write(&messages_file, "[]")?;
    }
    Ok(chat_dir)
}

pub fn load_messages(username: &str) -> Result<Vec<Message>> {
    create_chat_if_missing(username)?;
    let file_path = get_chat_dir(username)?.join("messages.json");
    let content = fs::read_to_string(file_path)?;
    let messages: Vec<Message> = serde_json::from_str(&content)?;
    Ok(messages)
}

pub fn save_messages(username: &str, messages: &[Message]) -> Result<()> {
    create_chat_if_missing(username)?;
    let file_path = get_chat_dir(username)?.join("messages.json");
    let content = serde_json::to_string_pretty(messages)?;
    fs::write(file_path, content)?;
    Ok(())
}

pub fn save_message(username: &str, message: &Message) -> Result<()> {
    let mut messages = load_messages(username)?;
    if let Some(index) = messages.iter().position(|m| m.id == message.id) {
        messages[index] = message.clone();
    } else {
        messages.push(message.clone());
    }
    save_messages(username, &messages)?;
    Ok(())
}

pub fn append_message(username: &str, message: Message) -> Result<()> {
    let mut messages = load_messages(username)?;
    messages.push(message);
    save_messages(username, &messages)?;
    Ok(())
}
