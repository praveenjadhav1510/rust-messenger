use crate::chat::conversation::get_conversation_summary;
use anyhow::Result;

pub fn exec_show(username: &str) -> Result<()> {
    let summary = get_conversation_summary(username)?;

    println!("Username:       {}", summary.username);
    println!("Fingerprint:    {}", summary.fingerprint);
    println!("Trust Level:    {}", summary.trust_level);
    println!("Total Messages: {}", summary.total_messages);

    let activity_str = match summary.last_activity {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "N/A".to_string(),
    };
    println!("Last Activity:  {}", activity_str);

    Ok(())
}
