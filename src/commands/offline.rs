use crate::presence::manager::set_user_offline;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    set_user_offline().await?;
    println!("You are now offline.");
    Ok(())
}
