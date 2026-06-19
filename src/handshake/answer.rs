use crate::network::candidate::IceCandidate;
use crate::peer::capabilities::PeerCapabilities;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionAnswer {
    #[serde(rename = "offerId")]
    pub offer_id: String,
    pub accepted: bool,
    #[serde(rename = "selectedCandidate")]
    pub selected_candidate: IceCandidate,
    pub capabilities: PeerCapabilities,
    pub timestamp: String,
}
