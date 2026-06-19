use crate::storage::filesystem::get_storage_dir;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    Pending,
    Accepted,
    Rejected,
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestStatus::Pending => write!(f, "PENDING"),
            RequestStatus::Accepted => write!(f, "ACCEPTED"),
            RequestStatus::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestType {
    MessageRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageRequest {
    pub id: Uuid,
    pub username: String,
    pub status: RequestStatus,
    pub created_at: DateTime<Utc>,
}

pub fn load_requests() -> Result<Vec<MessageRequest>> {
    let path = get_storage_dir()?.join("requests.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let requests: Vec<MessageRequest> = serde_json::from_str(&content)?;
    Ok(requests)
}

pub fn save_requests(requests: &[MessageRequest]) -> Result<()> {
    let path = get_storage_dir()?.join("requests.json");
    let content = serde_json::to_string_pretty(requests)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_request(username: &str) -> Result<MessageRequest> {
    let mut requests = load_requests()?;

    // Check if there is already a pending request for this user
    if requests
        .iter()
        .any(|r| r.username.eq_ignore_ascii_case(username) && r.status == RequestStatus::Pending)
    {
        return Err(anyhow!(
            "A pending request already exists for user '{}'.",
            username
        ));
    }

    let request = MessageRequest {
        id: Uuid::new_v4(),
        username: username.to_string(),
        status: RequestStatus::Pending,
        created_at: Utc::now(),
    };

    requests.push(request.clone());
    save_requests(&requests)?;
    Ok(request)
}

pub fn accept_request(username: &str) -> Result<MessageRequest> {
    let mut requests = load_requests()?;
    let mut target_req = None;

    for req in &mut requests {
        if req.username.eq_ignore_ascii_case(username) && req.status == RequestStatus::Pending {
            req.status = RequestStatus::Accepted;
            target_req = Some(req.clone());
            break;
        }
    }

    if let Some(req) = target_req {
        save_requests(&requests)?;
        Ok(req)
    } else {
        Err(anyhow!("No pending request found for user '{}'.", username))
    }
}

pub fn reject_request(username: &str) -> Result<MessageRequest> {
    let mut requests = load_requests()?;
    let mut target_req = None;

    for req in &mut requests {
        if req.username.eq_ignore_ascii_case(username) && req.status == RequestStatus::Pending {
            req.status = RequestStatus::Rejected;
            target_req = Some(req.clone());
            break;
        }
    }

    if let Some(req) = target_req {
        save_requests(&requests)?;
        Ok(req)
    } else {
        Err(anyhow!("No pending request found for user '{}'.", username))
    }
}
