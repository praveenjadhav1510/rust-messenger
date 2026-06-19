use crate::presence::manager::get_user_status;
use anyhow::Result;

pub async fn exec(username: &str) -> Result<()> {
    let presence = get_user_status(username).await?;

    if presence.online {
        let last_seen_raw = presence.last_seen.as_deref().unwrap_or("N/A");
        let last_seen = format_timestamp(last_seen_raw);
        let client_version = presence.client_version.as_deref().unwrap_or("N/A");

        println!("Username: {}", presence.username);
        println!("Status: ONLINE");
        println!();
        println!("Last Seen:");
        println!("{}", last_seen);
        println!();
        println!("Client Version:");
        println!("{}", client_version);
    } else {
        println!("Status: OFFLINE");
    }

    Ok(())
}

fn format_timestamp(ts: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        return dt
            .with_timezone(&chrono::Utc)
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return dt.format("%Y-%m-%d %H:%M UTC").to_string();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M") {
        return dt.format("%Y-%m-%d %H:%M UTC").to_string();
    }
    ts.to_string()
}
