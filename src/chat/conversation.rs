use crate::chat::storage::{get_chat_dir, load_messages};
use crate::contacts::manager::get_contact;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;

pub struct ConversationSummary {
    pub username: String,
    pub fingerprint: String,
    pub trust_level: String,
    pub total_messages: usize,
    pub last_activity: Option<DateTime<Utc>>,
}

pub fn get_total_messages(username: &str) -> Result<usize> {
    let messages = load_messages(username)?;
    Ok(messages.len())
}

pub fn get_last_activity(username: &str) -> Result<Option<DateTime<Utc>>> {
    let messages = load_messages(username)?;
    Ok(messages.last().map(|m| m.timestamp))
}

pub fn get_conversation_summary(username: &str) -> Result<ConversationSummary> {
    let contact = get_contact(username)?;
    let total_messages = get_total_messages(&contact.username)?;
    let last_activity = get_last_activity(&contact.username)?;
    Ok(ConversationSummary {
        username: contact.username,
        fingerprint: contact.fingerprint,
        trust_level: contact.trust_level.to_string(),
        total_messages,
        last_activity,
    })
}

pub fn list_conversations() -> Result<Vec<ConversationSummary>> {
    let contacts = crate::contacts::manager::load_contacts()?;
    let mut summaries = Vec::new();

    for contact in contacts {
        let total_messages = get_total_messages(&contact.username).unwrap_or(0);
        let last_activity = get_last_activity(&contact.username).unwrap_or(None);
        summaries.push(ConversationSummary {
            username: contact.username,
            fingerprint: contact.fingerprint,
            trust_level: contact.trust_level.to_string(),
            total_messages,
            last_activity,
        });
    }

    summaries.sort_by(|a, b| match (a.last_activity, b.last_activity) {
        (Some(ta), Some(tb)) => tb.cmp(&ta),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.username.cmp(&b.username),
    });

    Ok(summaries)
}

pub fn clear_conversation(username: &str) -> Result<()> {
    let chat_dir = get_chat_dir(username)?;
    if chat_dir.exists() {
        fs::remove_dir_all(chat_dir)?;
    }
    Ok(())
}
