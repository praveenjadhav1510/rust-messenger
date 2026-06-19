use crate::protocol::requests::{accept_request, load_requests, reject_request};
use anyhow::Result;

pub fn exec_list() -> Result<()> {
    let requests = load_requests()?;
    if requests.is_empty() {
        println!("No message requests found.");
        return Ok(());
    }

    println!(
        "{:<36} | {:<15} | {:<10} | {:<25}",
        "Request ID", "Username", "Status", "Created At"
    );
    println!("{}", "-".repeat(92));
    for r in requests {
        let created_at_str = r.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        println!(
            "{:<36} | {:<15} | {:<10} | {:<25}",
            r.id.to_string(),
            r.username,
            r.status.to_string(),
            created_at_str
        );
    }
    Ok(())
}

pub fn exec_accept(username: &str) -> Result<()> {
    match accept_request(username) {
        Ok(_) => {
            println!("✓ Message request from '{}' accepted.", username);
        }
        Err(e) => {
            println!("✗ Failed to accept request: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

pub fn exec_reject(username: &str) -> Result<()> {
    match reject_request(username) {
        Ok(_) => {
            println!("✓ Message request from '{}' rejected.", username);
        }
        Err(e) => {
            println!("✗ Failed to reject request: {}", e);
            return Err(e);
        }
    }
    Ok(())
}
