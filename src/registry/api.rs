use crate::registry::models::{
    ApiError, CandidatesResponse, CheckUsernameResponse, HeartbeatRequest, HeartbeatResponse,
    LockRequest, LockResponse, OfflineRequest, OfflineResponse, PresenceResponse,
    PublishCandidatesRequest, PublishCandidatesResponse, RecoverRequest, RecoverResponse,
    RegisterRequest, RegisterResponse, RemoveRequest, RemoveResponse, RenameRequest,
    RenameResponse, RestoreRequest, RestoreResponse, SearchResult, UnlockRequest, UnlockResponse,
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

    async fn post_request<Req, Resp>(
        &self,
        path: &str,
        body: &Req,
        action_name: &str,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let res = self.client.post(&url).json(body).send().await?;
        let status = res.status();

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("{}", api_err.error));
            }
            return Err(anyhow!(
                "{} failed with status code: {}",
                action_name,
                status
            ));
        }

        let resp_body: Resp = res.json().await?;
        Ok(resp_body)
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

    pub async fn recover_account(
        &self,
        username: &str,
        recovery_code: &str,
        new_public_key: &str,
    ) -> Result<RecoverResponse> {
        let req = RecoverRequest {
            username: username.to_string(),
            recovery_code: recovery_code.to_string(),
            new_public_key: new_public_key.to_string(),
        };
        self.post_request("/api/recover", &req, "Recover").await
    }

    pub async fn rename_account(
        &self,
        current_username: &str,
        new_username: &str,
        recovery_code: &str,
    ) -> Result<RenameResponse> {
        let req = RenameRequest {
            current_username: current_username.to_string(),
            new_username: new_username.to_string(),
            recovery_code: recovery_code.to_string(),
        };
        self.post_request("/api/rename", &req, "Rename").await
    }

    pub async fn remove_account(
        &self,
        username: &str,
        recovery_code: &str,
    ) -> Result<RemoveResponse> {
        let req = RemoveRequest {
            username: username.to_string(),
            recovery_code: recovery_code.to_string(),
        };
        self.post_request("/api/remove", &req, "Remove").await
    }

    pub async fn restore_account(
        &self,
        username: &str,
        recovery_code: &str,
        new_public_key: &str,
    ) -> Result<RestoreResponse> {
        let req = RestoreRequest {
            username: username.to_string(),
            recovery_code: recovery_code.to_string(),
            new_public_key: new_public_key.to_string(),
        };
        self.post_request("/api/restore", &req, "Restore").await
    }

    pub async fn lock_account(&self, username: &str, recovery_code: &str) -> Result<LockResponse> {
        let req = LockRequest {
            username: username.to_string(),
            recovery_code: recovery_code.to_string(),
        };
        self.post_request("/api/lock", &req, "Lock").await
    }

    pub async fn unlock_account(
        &self,
        username: &str,
        recovery_code: &str,
    ) -> Result<UnlockResponse> {
        let req = UnlockRequest {
            username: username.to_string(),
            recovery_code: recovery_code.to_string(),
        };
        self.post_request("/api/unlock", &req, "Unlock").await
    }

    pub async fn send_heartbeat(
        &self,
        username: &str,
        session_id: &str,
        client_version: &str,
    ) -> Result<HeartbeatResponse> {
        let req = HeartbeatRequest {
            username: username.to_string(),
            session_id: session_id.to_string(),
            client_version: client_version.to_string(),
        };
        self.post_request("/api/presence/heartbeat", &req, "Heartbeat")
            .await
    }

    pub async fn get_presence(&self, username: &str) -> Result<PresenceResponse> {
        let url = format!("{}/api/presence/{}", self.base_url, username);
        let res = self.client.get(&url).send().await?;
        let status = res.status();

        if status.as_u16() == 404 {
            return Err(anyhow!("User not found."));
        }

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("Get presence failed: {}", api_err.error));
            }
            return Err(anyhow!("Get presence failed with status code: {}", status));
        }

        let presence: PresenceResponse = res.json().await?;
        Ok(presence)
    }

    pub async fn set_offline(&self, username: &str, session_id: &str) -> Result<OfflineResponse> {
        let req = OfflineRequest {
            username: username.to_string(),
            session_id: session_id.to_string(),
        };
        self.post_request("/api/presence/offline", &req, "Offline")
            .await
    }

    pub async fn get_candidates(&self, username: &str) -> Result<CandidatesResponse> {
        let url = format!("{}/api/candidates/{}", self.base_url, username);
        let res = self.client.get(&url).send().await?;
        let status = res.status();

        if status.as_u16() == 404 {
            return Err(anyhow!("User not found."));
        }

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("Get candidates failed: {}", api_err.error));
            }
            return Err(anyhow!(
                "Get candidates failed with status code: {}",
                status
            ));
        }

        let candidates_resp: CandidatesResponse = res.json().await?;
        Ok(candidates_resp)
    }

    pub async fn publish_candidates(
        &self,
        username: &str,
        session_id: &str,
        candidates: &[crate::network::candidate::IceCandidate],
    ) -> Result<PublishCandidatesResponse> {
        let req = PublishCandidatesRequest {
            username: username.to_string(),
            session_id: session_id.to_string(),
            candidates: candidates.to_vec(),
        };
        self.post_request("/api/candidates/publish", &req, "PublishCandidates")
            .await
    }

    pub async fn delete_candidates(&self, username: &str) -> Result<bool> {
        let url = format!("{}/api/candidates/{}", self.base_url, username);
        let res = self.client.delete(&url).send().await?;
        let status = res.status();

        if !status.is_success() {
            if let Ok(api_err) = res.json::<ApiError>().await {
                return Err(anyhow!("Delete candidates failed: {}", api_err.error));
            }
            return Err(anyhow!(
                "Delete candidates failed with status code: {}",
                status
            ));
        }

        #[derive(serde::Deserialize)]
        struct DeleteResponse {
            success: bool,
        }
        let resp: DeleteResponse = res.json().await?;
        Ok(resp.success)
    }
}
