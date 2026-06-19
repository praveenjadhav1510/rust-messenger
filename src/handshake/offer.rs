use crate::network::candidate::IceCandidate;
use crate::peer::capabilities::PeerCapabilities;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionOffer {
    #[serde(rename = "offerId")]
    pub offer_id: String,
    pub sender: String,
    pub recipient: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub candidates: Vec<IceCandidate>,
    pub capabilities: PeerCapabilities,
    pub timestamp: String,
}
