use crate::chat::models::MessageStatus;
use crate::messaging::sender::MessageSender;
use anyhow::Result;

pub async fn exec(username: &str, message: &str) -> Result<()> {
    let msg = MessageSender::send_message(username, message).await?;
    if msg.status == MessageStatus::Delivered {
        println!("Message delivered.");
    } else {
        println!("Message delivery failed.");
    }
    Ok(())
}
