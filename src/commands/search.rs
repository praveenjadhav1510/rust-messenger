use crate::registry::api::RegistryClient;
use crate::storage::filesystem::read_profile;
use anyhow::Result;

pub async fn exec(query: &str) -> Result<()> {
    let profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url);

    let results = client.search_users(query).await?;

    if results.is_empty() {
        println!("No users found for query: '{}'", query);
    } else {
        for user in results {
            println!("{} ({})", user.username, user.fingerprint);
        }
    }

    Ok(())
}
