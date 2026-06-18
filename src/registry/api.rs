use crate::registry::models::{
    ApiError, CheckUsernameResponse, RegisterRequest, RegisterResponse, SearchResult,
    UserLookupResponse,
};
use anyhow::{Result, anyhow};
use reqwest::Client;

pub struct RegistryClient {
    client: Client,
    base_url: String,
}

impl RegistryClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn check_username(&self, username: &str) -> Result<bool> {
        let url = format!("{}/api/check/{}", self.base_url, username);
        let res = self.client.get(&url).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!(
                "Registry unavailable or returned error code: {}",
                res.status()
            ));
        }

        let body: CheckUsernameResponse = res.json().await?;
        Ok(body.available)
    }

    pub async fn register_user(
        &self,
        username: &str,
        public_key: &str,
    ) -> Result<RegisterResponse> {
        let url = format!("{}/api/register", self.base_url);
        let req_body = RegisterRequest {
            username: username.to_string(),
            public_key: public_key.to_string(),
        };

        let res = self.client.post(&url).json(&req_body).send().await?;
        let status = res.status();

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("Registration failed: {}", api_err.error));
            }
            return Err(anyhow!("Registration failed with status code: {}", status));
        }

        let resp_body: RegisterResponse = res.json().await?;
        Ok(resp_body)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = format!("{}/api/search?q={}", self.base_url, query);
        let res = self.client.get(&url).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!("Search failed with status code: {}", res.status()));
        }

        let results: Vec<SearchResult> = res.json().await?;
        Ok(results)
    }

    pub async fn lookup_user(&self, username: &str) -> Result<UserLookupResponse> {
        let url = format!("{}/api/user/{}", self.base_url, username);
        let res = self.client.get(&url).send().await?;
        let status = res.status();

        if status.as_u16() == 404 {
            return Err(anyhow!("User not found."));
        }

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("Lookup failed: {}", api_err.error));
            }
            return Err(anyhow!("Lookup failed with status code: {}", status));
        }

        let user: UserLookupResponse = res.json().await?;
        Ok(user)
    }
}
