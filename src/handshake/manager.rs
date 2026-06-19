use crate::handshake::answer::ConnectionAnswer;
use crate::handshake::negotiation::{HandshakeSession, load_handshakes, save_handshakes};
use crate::handshake::offer::ConnectionOffer;
use crate::network::candidate::IceCandidate;
use crate::peer::capabilities::load_local_capabilities;
use crate::session::manager::get_current_session;
use anyhow::{Result, anyhow};

pub struct HandshakeManager;

impl HandshakeManager {
    pub fn create_offer(recipient: &str, candidates: Vec<IceCandidate>) -> Result<ConnectionOffer> {
        let session = get_current_session()?;
        let capabilities = load_local_capabilities()?;

        let offer = ConnectionOffer {
            offer_id: uuid::Uuid::new_v4().to_string(),
            sender: session.username,
            recipient: recipient.to_string(),
            session_id: session.session_id,
            candidates,
            capabilities,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Self::validate_offer(&offer)?;

        let mut handshakes = load_handshakes()?;
        handshakes.push(HandshakeSession {
            offer: offer.clone(),
            answer: None,
            status: "PENDING".to_string(),
        });
        save_handshakes(&handshakes)?;

        Ok(offer)
    }

    pub fn validate_offer(offer: &ConnectionOffer) -> Result<()> {
        if offer.offer_id.trim().is_empty() {
            return Err(anyhow!("Invalid offer: offerId is empty."));
        }
        if offer.sender.trim().is_empty() {
            return Err(anyhow!("Invalid offer: sender is empty."));
        }
        if offer.recipient.trim().is_empty() {
            return Err(anyhow!("Invalid offer: recipient is empty."));
        }
        Ok(())
    }

    pub fn accept_offer(
        offer: ConnectionOffer,
        selected_candidate: IceCandidate,
    ) -> Result<ConnectionAnswer> {
        Self::validate_offer(&offer)?;
        let capabilities = load_local_capabilities()?;

        let answer = ConnectionAnswer {
            offer_id: offer.offer_id.clone(),
            accepted: true,
            selected_candidate,
            capabilities,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mut handshakes = load_handshakes()?;
        if let Some(session) = handshakes
            .iter_mut()
            .find(|s| s.offer.offer_id == offer.offer_id)
        {
            session.answer = Some(answer.clone());
            session.status = "ACCEPTED".to_string();
        } else {
            handshakes.push(HandshakeSession {
                offer,
                answer: Some(answer.clone()),
                status: "ACCEPTED".to_string(),
            });
        }
        save_handshakes(&handshakes)?;

        Ok(answer)
    }

    pub fn reject_offer(offer: ConnectionOffer) -> Result<ConnectionAnswer> {
        Self::validate_offer(&offer)?;
        let capabilities = load_local_capabilities()?;

        let answer = ConnectionAnswer {
            offer_id: offer.offer_id.clone(),
            accepted: false,
            selected_candidate: IceCandidate {
                foundation: "".to_string(),
                component: 0,
                transport: "".to_string(),
                priority: 0,
                address: "".to_string(),
                port: 0,
                candidate_type: crate::network::candidate::CandidateType::Host,
            },
            capabilities,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mut handshakes = load_handshakes()?;
        if let Some(session) = handshakes
            .iter_mut()
            .find(|s| s.offer.offer_id == offer.offer_id)
        {
            session.answer = Some(answer.clone());
            session.status = "REJECTED".to_string();
        } else {
            handshakes.push(HandshakeSession {
                offer,
                answer: Some(answer.clone()),
                status: "REJECTED".to_string(),
            });
        }
        save_handshakes(&handshakes)?;

        Ok(answer)
    }

    pub fn complete_handshake(offer_id: &str, answer: ConnectionAnswer) -> Result<()> {
        let mut handshakes = load_handshakes()?;
        let session = handshakes
            .iter_mut()
            .find(|s| s.offer.offer_id == offer_id)
            .ok_or_else(|| anyhow!("Handshake with offerId '{}' not found.", offer_id))?;

        session.answer = Some(answer.clone());
        session.status = if answer.accepted {
            "ACCEPTED".to_string()
        } else {
            "REJECTED".to_string()
        };

        save_handshakes(&handshakes)?;
        Ok(())
    }
}
