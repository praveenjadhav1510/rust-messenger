use crate::candidates::manager::CandidateManager;
use anyhow::Result;

pub async fn exec() -> Result<()> {
    let publ = CandidateManager::refresh().await?;
    println!("Candidates refreshed successfully.");
    println!();
    println!("Published:");
    println!("{} candidates", publ.candidates.len());
    Ok(())
}
