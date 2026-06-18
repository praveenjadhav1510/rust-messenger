use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub username: Option<String>,
    pub fingerprint: Option<String>,
    pub registry_url: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            username: None,
            fingerprint: None,
            registry_url: "https://user-registry-ten.vercel.app".to_string(),
        }
    }
}
