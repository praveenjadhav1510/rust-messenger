use crate::candidates::manager::CandidateManager;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    let publ = CandidateManager::publish().await?;
    println!("Candidates published successfully.");
    println!();
    println!("Published:");
    println!("{} candidates", publ.candidates.len());
    Ok(())
}
