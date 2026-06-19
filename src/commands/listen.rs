use crate::messaging::listener::MessageListener;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    MessageListener::run().await?;
    Ok(())
}
