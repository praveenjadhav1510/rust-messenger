use crate::registry::api::RegistryClient;
use crate::session::manager::get_current_session;
use crate::storage::filesystem::read_profile;
use anyhow::{Result, anyhow};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct HeartbeatService {
    stop_tx: Option<oneshot::Sender<()>>,
    join_handle: Option<JoinHandle<()>>,
}

impl Default for HeartbeatService {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatService {
    pub fn new() -> Self {
        Self {
            stop_tx: None,
            join_handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.join_handle.is_some() {
            return Err(anyhow!("Heartbeat service is already running."));
        }

        let profile = read_profile()?;
        let username = profile
            .username
            .clone()
            .ok_or_else(|| anyhow!("No registered identity found."))?;
        let registry_url = profile.registry_url.clone();

        let session = get_current_session()?;
        if !session.online {
            return Err(anyhow!("Session is offline. Start session first."));
        }

        let session_id = session.session_id.clone();
        let client_version = session.client_version.clone();

        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        self.stop_tx = Some(stop_tx);

        let client = RegistryClient::new(registry_url);

        let join_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::tick_client(&client, &username, &session_id, &client_version).await {
                            eprintln!("Heartbeat error: {}", e);
                        }
                    }
                    _ = &mut stop_rx => {
                        break;
                    }
                }
            }
        });

        self.join_handle = Some(join_handle);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.await;
        }
        Ok(())
    }

    pub async fn tick(&self) -> Result<()> {
        let profile = read_profile()?;
        let username = profile
            .username
            .clone()
            .ok_or_else(|| anyhow!("No registered identity found."))?;
        let registry_url = profile.registry_url.clone();

        let session = get_current_session()?;
        let session_id = session.session_id.clone();
        let client_version = session.client_version.clone();

        let client = RegistryClient::new(registry_url);
        Self::tick_client(&client, &username, &session_id, &client_version).await
    }

    async fn tick_client(
        client: &RegistryClient,
        username: &str,
        session_id: &str,
        client_version: &str,
    ) -> Result<()> {
        let resp = client
            .send_heartbeat(username, session_id, client_version)
            .await?;
        if !resp.success {
            return Err(anyhow!("Registry rejected heartbeat."));
        }
        Ok(())
    }
}
