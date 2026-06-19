use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NatType {
    Unknown,
    OpenInternet,
    FullCone,
    Restricted,
    PortRestricted,
    Symmetric,
}

impl std::fmt::Display for NatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NatType::Unknown => write!(f, "UNKNOWN"),
            NatType::OpenInternet => write!(f, "OPEN_INTERNET"),
            NatType::FullCone => write!(f, "FULL_CONE"),
            NatType::Restricted => write!(f, "RESTRICTED"),
            NatType::PortRestricted => write!(f, "PORT_RESTRICTED"),
            NatType::Symmetric => write!(f, "SYMMETRIC"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    #[serde(rename = "localIp")]
    pub local_ip: String,
    #[serde(rename = "publicIp")]
    pub public_ip: String,
    #[serde(rename = "publicPort")]
    pub public_port: u16,
    #[serde(rename = "natType")]
    pub nat_type: NatType,
    #[serde(rename = "stunServer")]
    pub stun_server: String,
}
