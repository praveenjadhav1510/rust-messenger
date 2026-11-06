use crate::candidates::manager::CandidateManager;
use crate::chat::conversation::clear_conversation;
use crate::chat::models::{Direction, Message, MessageStatus, MessageType};
use crate::chat::storage::append_message;
use crate::config::profile::Profile;
use crate::connection::manager::{ConnectionManager, load_sessions, save_sessions};
use crate::connection::models::{PeerSession, TransportType};
use crate::contacts::manager::{
    add_contact, get_contact, load_contacts, remove_contact, update_trust_level,
};
use crate::contacts::models::TrustLevel;
use crate::crypto::keys::generate_keypair;
use crate::ice::connectivity::{
    ConnectivityManager, IceSession, load_ice_sessions, save_ice_sessions,
};
use crate::messaging::receipts::mark_incoming_messages_read;
use crate::messaging::sender::MessageSender;
use crate::network::candidate::CandidateType;
use crate::network::candidate::IceCandidate;
use crate::network::discovery::DiscoveryManager;
use crate::network::udp::UdpTransport;
use crate::peer::capabilities::load_local_capabilities;
use crate::peer::discovery::PeerDiscoveryManager;
use crate::presence::manager::{get_user_status, set_user_offline, set_user_online};
use crate::protocol::packet::{Packet, PacketType};
use crate::protocol::requests::{accept_request, reject_request};
use crate::punch::coordinator::HolePunchCoordinator;
use crate::punch::session::load_punch_sessions;
use crate::punch::state::PunchState;
use crate::registry::api::RegistryClient;
use crate::registry::models::{SearchResult, UserLookupResponse};
use crate::secure::bootstrap::bootstrap_secure_session;
use crate::storage::filesystem::{
    ensure_storage_dir, get_storage_dir, read_profile, read_public_key, write_keys, write_profile,
};
use crate::tui::actions::RunRequest;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::fs;
use std::net::SocketAddr;
use uuid::Uuid;

pub struct SecretPayload {
    pub title: String,
    pub code: String,
    pub warning: String,
}

pub enum ServiceData {
    None,
    Search(Vec<SearchResult>),
    Lookup(Box<UserLookupResponse>),
    Lines(Vec<(String, String)>),
}

pub struct ServiceOutput {
    pub summary: String,
    pub secret: Option<SecretPayload>,
    pub data: ServiceData,
}

impl ServiceOutput {
    fn message(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            secret: None,
            data: ServiceData::None,
        }
    }

    fn lines(summary: impl Into<String>, lines: Vec<(String, String)>) -> Self {
        Self {
            summary: summary.into(),
            secret: None,
            data: ServiceData::Lines(lines),
        }
    }
}

fn client() -> Result<RegistryClient> {
    let profile = read_profile().unwrap_or_default();
    Ok(RegistryClient::new(profile.registry_url))
}

fn registry_url() -> String {
    read_profile()
        .map(|p| p.registry_url)
        .unwrap_or_else(|_| Profile::default().registry_url)
}

fn candidate_label(candidate_type: CandidateType) -> &'static str {
    match candidate_type {
        CandidateType::Host => "HOST",
        CandidateType::ServerReflexive => "SRFLX",
        CandidateType::Relay => "RELAY",
    }
}

pub async fn execute(request: RunRequest) -> Result<ServiceOutput> {
    match request {
        RunRequest::Init { force } => init_identity(force),
        RunRequest::Register(username) => register(username).await,
        RunRequest::Lookup(username) => lookup(username).await,
        RunRequest::Recover { username, code } => recover(username, code).await,
        RunRequest::Rename { new_username, code } => rename(new_username, code).await,
        RunRequest::RemoveAccount { code } => remove_account(code).await,
        RunRequest::Restore { username, code } => restore(username, code).await,
        RunRequest::Lock { code } => lock(code).await,
        RunRequest::Unlock { code } => unlock(code).await,
        RunRequest::Status(username) => status(username).await,
        RunRequest::Search(query) => search(query).await,
        RunRequest::AddContact(username) => add_new_contact(username).await,
        RunRequest::RemoveContact(username) => {
            remove_contact(&username)?;
            Ok(ServiceOutput::message(format!(
                "Removed contact '{}'.",
                username
            )))
        }
        RunRequest::ShowContact(username) => show_contact(username),
        RunRequest::SetTrust { username, level } => {
            update_trust_level(&username, level)?;
            let word = match level {
                TrustLevel::Verified => "verified",
                TrustLevel::Blocked => "blocked",
                TrustLevel::Unverified => "unverified",
            };
            Ok(ServiceOutput::message(format!(
                "Contact '{}' marked {}.",
                username, word
            )))
        }
        RunRequest::AcceptRequest(username) => {
            accept_request(&username)?;
            Ok(ServiceOutput::message(format!(
                "Accepted request from '{}'.",
                username
            )))
        }
        RunRequest::RejectRequest(username) => {
            reject_request(&username)?;
            Ok(ServiceOutput::message(format!(
                "Rejected request from '{}'.",
                username
            )))
        }
        RunRequest::SendEncrypted { peer, text } => send_encrypted(peer, text).await,
        RunRequest::QueueMessage { peer, text } => queue_message(peer, text),
        RunRequest::ClearChat(peer) => {
            clear_conversation(&peer)?;
            Ok(ServiceOutput::message(format!(
                "Cleared conversation with '{}'.",
                peer
            )))
        }
        RunRequest::MarkRead(peer) => {
            mark_incoming_messages_read(&peer).await?;
            Ok(ServiceOutput::message("Marked messages as read."))
        }
        RunRequest::Inject { peer, text } => {
            crate::chat::simulator::inject_incoming_message(&peer, &text)?;
            Ok(ServiceOutput::message(format!(
                "Injected test message from '{}'.",
                peer
            )))
        }
        RunRequest::Connect(username) => connect(username).await,
        RunRequest::Disconnect(username) => {
            ConnectionManager::disconnect_peer(&username)?;
            Ok(ServiceOutput::message(format!(
                "Disconnected from '{}'.",
                username
            )))
        }
        RunRequest::Ping(username) => ping(username).await,
        RunRequest::Netinfo => netinfo().await,
        RunRequest::Capabilities => capabilities(),
        RunRequest::Discover(username) => discover(username).await,
        RunRequest::Negotiate(username) => negotiate(username).await,
        RunRequest::SecureSession(username) => {
            bootstrap_secure_session(&username)?;
            Ok(ServiceOutput::message(format!(
                "Secure ChaCha20-Poly1305 session established with '{}'.",
                username
            )))
        }
        RunRequest::CandidatePairs(username) => candidate_pairs(username).await,
        RunRequest::IceCheck(username) => ice_check(username).await,
        RunRequest::Punch(username) => punch(username).await,
        RunRequest::PunchStatus(username) => punch_status(username),
        RunRequest::PublishCandidates => {
            let publication = CandidateManager::publish().await?;
            Ok(ServiceOutput::message(format!(
                "Published {} ICE candidates (expires in 60s).",
                publication.candidates.len()
            )))
        }
        RunRequest::RefreshCandidates => {
            let publication = CandidateManager::refresh().await?;
            Ok(ServiceOutput::message(format!(
                "Refreshed and published {} ICE candidates.",
                publication.candidates.len()
            )))
        }
        RunRequest::FetchCandidates(username) => fetch_candidates(username).await,
        RunRequest::TestUdp => {
            tokio::task::spawn_blocking(test_udp).await??;
            Ok(ServiceOutput::message("UDP transport operational."))
        }
        RunRequest::Online => {
            let session_id = set_user_online().await?;
            Ok(ServiceOutput::message(format!(
                "Online. Session {}",
                session_id
            )))
        }
        RunRequest::Offline => {
            set_user_offline().await?;
            Ok(ServiceOutput::message("You are now offline."))
        }
    }
}

fn init_identity(force: bool) -> Result<ServiceOutput> {
    let dir = ensure_storage_dir()?;
    let private_path = dir.join("private.key");
    let public_path = dir.join("public.key");
    let profile_path = dir.join("profile.json");
    let contacts_path = dir.join("contacts.json");

    if !force && (private_path.exists() || public_path.exists() || profile_path.exists()) {
        return Err(anyhow!(
            "Identity already exists. Use force to overwrite it."
        ));
    }

    let keys = generate_keypair();
    write_keys(&keys.private_key_hex, &keys.public_key_hex)?;
    write_profile(&Profile::default())?;
    if force || !contacts_path.exists() {
        fs::write(&contacts_path, "[]")?;
    }

    Ok(ServiceOutput::message(
        "Identity initialized: Ed25519 keypair, profile and contacts store created.",
    ))
}

async fn register(username: String) -> Result<ServiceOutput> {
    let public_key = read_public_key()?;
    let mut profile = read_profile()?;
    let client = RegistryClient::new(profile.registry_url.clone());

    if !client.check_username(&username).await? {
        return Err(anyhow!("Username '{}' is already taken.", username));
    }

    let response = client.register_user(&username, &public_key).await?;
    if !response.success {
        return Err(anyhow!("Registration failed."));
    }

    profile.username = Some(response.username.clone());
    profile.fingerprint = Some(response.fingerprint.clone());
    write_profile(&profile)?;

    Ok(ServiceOutput {
        summary: format!(
            "Registered '{}' with fingerprint {}.",
            response.username, response.fingerprint
        ),
        secret: Some(SecretPayload {
            title: "RECOVERY CODE — SHOWN ONCE".to_string(),
            code: response.recovery_code,
            warning: "Write this down now. It is never stored on disk and cannot be recovered."
                .to_string(),
        }),
        data: ServiceData::None,
    })
}

async fn lookup(username: String) -> Result<ServiceOutput> {
    let user = client()?.lookup_user(&username).await?;
    Ok(ServiceOutput {
        summary: format!("Found '{}' in the registry.", user.username),
        secret: None,
        data: ServiceData::Lookup(Box::new(user)),
    })
}

async fn recover(username: String, code: String) -> Result<ServiceOutput> {
    if code.trim().is_empty() {
        return Err(anyhow!("Recovery code is required."));
    }
    let mut profile = read_profile().unwrap_or_default();
    let client = RegistryClient::new(profile.registry_url.clone());
    let keys = generate_keypair();

    let response = client
        .recover_account(&username, &code, &keys.public_key_hex)
        .await?;
    if !response.success {
        return Err(anyhow!("Recovery failed."));
    }

    write_keys(&keys.private_key_hex, &keys.public_key_hex)?;
    profile.username = Some(username.clone());
    profile.fingerprint = Some(response.fingerprint.clone());
    write_profile(&profile)?;

    Ok(ServiceOutput::message(format!(
        "Recovered '{}'. New fingerprint {}. Old private key is no longer valid.",
        username, response.fingerprint
    )))
}

async fn rename(new_username: String, code: String) -> Result<ServiceOutput> {
    let mut profile = read_profile()?;
    let current = profile
        .username
        .clone()
        .ok_or_else(|| anyhow!("No registered identity found."))?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client
        .rename_account(&current, &new_username, &code)
        .await?;
    if !response.success {
        return Err(anyhow!("Rename failed."));
    }
    profile.username = Some(response.username.clone());
    write_profile(&profile)?;
    Ok(ServiceOutput::message(format!(
        "Renamed '{}' to '{}'.",
        current, response.username
    )))
}

async fn remove_account(code: String) -> Result<ServiceOutput> {
    let mut profile = read_profile()?;
    let username = profile
        .username
        .clone()
        .ok_or_else(|| anyhow!("No registered identity found."))?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client.remove_account(&username, &code).await?;
    if !response.success {
        return Err(anyhow!("Account removal failed."));
    }
    profile.username = None;
    profile.fingerprint = None;
    write_profile(&profile)?;
    Ok(ServiceOutput::message(format!(
        "Account '{}' deactivated. The username is reserved for 30 days.",
        username
    )))
}

async fn restore(username: String, code: String) -> Result<ServiceOutput> {
    let mut profile = read_profile().unwrap_or_default();
    let client = RegistryClient::new(profile.registry_url.clone());
    let keys = generate_keypair();
    let response = client
        .restore_account(&username, &code, &keys.public_key_hex)
        .await?;
    if !response.success {
        return Err(anyhow!("Restore failed."));
    }
    write_keys(&keys.private_key_hex, &keys.public_key_hex)?;
    let info = client.lookup_user(&username).await?;
    profile.username = Some(username.clone());
    profile.fingerprint = Some(info.fingerprint);
    write_profile(&profile)?;
    Ok(ServiceOutput::message(format!(
        "Restored '{}' with a fresh keypair.",
        username
    )))
}

async fn lock(code: String) -> Result<ServiceOutput> {
    let profile = read_profile()?;
    let username = profile
        .username
        .clone()
        .ok_or_else(|| anyhow!("No registered identity found."))?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client.lock_account(&username, &code).await?;
    if !response.success {
        return Err(anyhow!("Lock failed."));
    }
    Ok(ServiceOutput::message(
        "Account locked. Others should not trust this identity until recovery.",
    ))
}

async fn unlock(code: String) -> Result<ServiceOutput> {
    let profile = read_profile()?;
    let username = profile
        .username
        .clone()
        .ok_or_else(|| anyhow!("No registered identity found."))?;
    let client = RegistryClient::new(profile.registry_url.clone());
    let response = client.unlock_account(&username, &code).await?;
    if !response.success {
        return Err(anyhow!("Unlock failed."));
    }
    Ok(ServiceOutput::message("Account unlocked successfully."))
}

async fn status(username: String) -> Result<ServiceOutput> {
    let presence = get_user_status(&username).await?;
    let mut lines = vec![
        ("Username".to_string(), presence.username.clone()),
        (
            "Status".to_string(),
            if presence.online {
                "ONLINE".to_string()
            } else {
                "OFFLINE".to_string()
            },
        ),
    ];
    if let Some(session) = presence.session_id {
        lines.push(("Session".to_string(), session));
    }
    if let Some(last_seen) = presence.last_seen {
        lines.push(("Last seen".to_string(), last_seen));
    }
    if let Some(version) = presence.client_version {
        lines.push(("Client version".to_string(), version));
    }
    Ok(ServiceOutput::lines(
        format!("Presence for '{}'.", username),
        lines,
    ))
}

async fn search(query: String) -> Result<ServiceOutput> {
    let results = client()?.search_users(&query).await?;
    let count = results.len();
    Ok(ServiceOutput {
        summary: format!("{} result(s) for '{}'.", count, query),
        secret: None,
        data: ServiceData::Search(results),
    })
}

async fn add_new_contact(username: String) -> Result<ServiceOutput> {
    let contact = add_contact(&username, &registry_url()).await?;
    Ok(ServiceOutput::message(format!(
        "Added '{}' ({}).",
        contact.username, contact.fingerprint
    )))
}

fn show_contact(username: String) -> Result<ServiceOutput> {
    let contact = get_contact(&username)?;
    let lines = vec![
        ("Username".to_string(), contact.username.clone()),
        ("Fingerprint".to_string(), contact.fingerprint.clone()),
        ("Trust level".to_string(), contact.trust_level.to_string()),
        ("Account status".to_string(), contact.account_status.clone()),
        (
            "Added".to_string(),
            contact.added_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        ),
        ("Public key".to_string(), contact.public_key.clone()),
        (
            "Notes".to_string(),
            if contact.notes.is_empty() {
                "—".to_string()
            } else {
                contact.notes.clone()
            },
        ),
    ];
    Ok(ServiceOutput::lines("CONTACT DETAILS", lines))
}

async fn send_encrypted(peer: String, text: String) -> Result<ServiceOutput> {
    let contact = get_contact(&peer)?;
    if contact.trust_level == TrustLevel::Blocked {
        return Err(anyhow!("Contact '{}' is blocked.", peer));
    }
    let length = text.chars().count();
    if length == 0 || length > 4000 {
        return Err(anyhow!("Message must be 1–4000 characters."));
    }
    let msg = MessageSender::send_message(&peer, &text).await?;
    if msg.status != MessageStatus::Delivered {
        return Err(anyhow!("Delivery failed: no ACK received from '{}'.", peer));
    }
    Ok(ServiceOutput::message(format!(
        "Message delivered to '{}'.",
        peer
    )))
}

fn queue_message(peer: String, text: String) -> Result<ServiceOutput> {
    let contact = get_contact(&peer)?;
    if contact.trust_level == TrustLevel::Blocked {
        return Err(anyhow!("Cannot message blocked contact '{}'.", peer));
    }
    let length = text.chars().count();
    if length == 0 || length > 4000 {
        return Err(anyhow!("Message must be 1–4000 characters."));
    }
    let msg = Message {
        id: Uuid::new_v4(),
        direction: Direction::Outgoing,
        timestamp: Utc::now(),
        status: MessageStatus::Queued,
        message_type: MessageType::Text,
        content: text,
        signature: None,
        reply_to: None,
    };
    append_message(&peer, msg)?;
    Ok(ServiceOutput::message(format!(
        "Message to '{}' queued locally.",
        peer
    )))
}

async fn connect(username: String) -> Result<ServiceOutput> {
    let session = ConnectionManager::connect_peer(&username).await?;
    Ok(ServiceOutput::message(format!(
        "Connected to '{}' via {}.",
        session.username, session.transport
    )))
}

async fn ping(username: String) -> Result<ServiceOutput> {
    let peer = ConnectionManager::get_peer(&username)?;
    let profile = read_profile()?;
    let sender = profile
        .username
        .ok_or_else(|| anyhow!("No registered identity found."))?;

    let mut manager = crate::transport::manager::TransportManager::new();
    let transport = crate::transport::registry::create_transport(peer.transport, &username)?;
    manager.register_transport(transport);

    let packet = Packet {
        version: 1,
        packet_type: PacketType::Ping,
        message_id: Uuid::new_v4(),
        sender,
        recipient: username.clone(),
        timestamp: Utc::now(),
        nonce: Uuid::new_v4().to_string(),
        encrypted_payload: String::new(),
        signature: "mock-signature".to_string(),
    };
    packet.validate()?;
    manager.send_packet(packet)?;
    Ok(ServiceOutput::message(format!(
        "Ping sent to '{}'.",
        username
    )))
}

async fn netinfo() -> Result<ServiceOutput> {
    let info = DiscoveryManager::discover().await?;
    let candidates = DiscoveryManager::get_candidates()?;
    let mut lines = vec![
        ("Local IP".to_string(), info.local_ip),
        ("Public IP".to_string(), info.public_ip),
        ("Public port".to_string(), info.public_port.to_string()),
        ("NAT type".to_string(), info.nat_type.to_string()),
        ("STUN server".to_string(), info.stun_server),
        (
            "Candidates".to_string(),
            format!("{} discovered", candidates.len()),
        ),
    ];
    for candidate in &candidates {
        lines.push((
            candidate.to_display_string(),
            format!("priority {}", candidate.priority),
        ));
    }
    Ok(ServiceOutput::lines("NETWORK DISCOVERY", lines))
}

fn capabilities() -> Result<ServiceOutput> {
    let caps = load_local_capabilities()?;
    let lines = vec![
        ("ICE".to_string(), caps.supports_ice.to_string()),
        ("TURN".to_string(), caps.supports_turn.to_string()),
        ("Files".to_string(), caps.supports_files.to_string()),
        ("Images".to_string(), caps.supports_images.to_string()),
        (
            "Max file size".to_string(),
            format!("{} MB", caps.max_file_size_mb),
        ),
        ("Client version".to_string(), caps.client_version),
    ];
    Ok(ServiceOutput::lines("LOCAL CAPABILITIES", lines))
}

async fn discover(username: String) -> Result<ServiceOutput> {
    let peer = PeerDiscoveryManager::discover_peer(&username).await?;
    let mut lines = vec![
        ("Username".to_string(), peer.username.clone()),
        (
            "Online".to_string(),
            if peer.online { "YES" } else { "NO" }.to_string(),
        ),
        (
            "ICE capable".to_string(),
            peer.capabilities.supports_ice.to_string(),
        ),
        (
            "TURN capable".to_string(),
            peer.capabilities.supports_turn.to_string(),
        ),
        (
            "Candidates".to_string(),
            format!("{} discovered", peer.candidates.len()),
        ),
    ];
    for candidate in &peer.candidates {
        lines.push((
            candidate.to_display_string(),
            format!("priority {}", candidate.priority),
        ));
    }
    Ok(ServiceOutput::lines(
        format!("PEER DISCOVERY · {}", username),
        lines,
    ))
}

async fn negotiate(username: String) -> Result<ServiceOutput> {
    let remote_candidates =
        crate::peer::candidate_exchange::fetch_remote_candidates(&username).await?;
    if remote_candidates.is_empty() {
        return Err(anyhow!("No remote candidates found for '{}'.", username));
    }
    let local_candidates = DiscoveryManager::load_candidates()?;
    if local_candidates.is_empty() {
        return Err(anyhow!("No local candidates. Run network discovery first."));
    }
    let ice_sessions = load_ice_sessions()?;
    let ice_session = ice_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(&username))
        .ok_or_else(|| anyhow!("No successful ICE check for '{}'.", username))?;
    let best_pair = ice_session.selected_pair.clone();

    let local_presence = crate::session::manager::get_current_session()?;
    let remote_presence = crate::presence::manager::get_user_status(&username).await?;
    let remote_session_id = remote_presence
        .session_id
        .ok_or_else(|| anyhow!("Peer '{}' is offline or has no active session.", username))?;

    let mut presence_ids = [local_presence.session_id.clone(), remote_session_id];
    presence_ids.sort();
    let combined = presence_ids.join("_");
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let negotiated = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let peer_session = PeerSession::new(username.clone(), negotiated.clone(), TransportType::Ice);
    let mut sessions = load_sessions()?;
    sessions.retain(|s| !s.username.eq_ignore_ascii_case(&username));
    sessions.push(peer_session);
    save_sessions(&sessions)?;

    Ok(ServiceOutput::lines(
        format!("Negotiation successful with '{}'.", username),
        vec![
            (
                "Selected pair".to_string(),
                format!(
                    "{} ↔ {}",
                    candidate_label(best_pair.local.candidate_type),
                    candidate_label(best_pair.remote.candidate_type)
                ),
            ),
            ("Transport".to_string(), "ICE / UDP".to_string()),
            ("Session".to_string(), negotiated),
        ],
    ))
}

async fn candidate_pairs(username: String) -> Result<ServiceOutput> {
    let remote = PeerDiscoveryManager::fetch_remote_candidates(&username).await?;
    let local = DiscoveryManager::load_candidates()?;
    let mut pairs = ConnectivityManager::build_candidate_pairs(&local, &remote);
    ConnectivityManager::prioritize_pairs(&mut pairs);

    let lines = if pairs.is_empty() {
        vec![("Pairs".to_string(), "No candidate pairs".to_string())]
    } else {
        pairs
            .iter()
            .map(|pair| {
                (
                    format!(
                        "{}:{}",
                        candidate_label(pair.local.candidate_type),
                        pair.local.address
                    ),
                    format!(
                        "{} → {}:{}  ·  priority {}",
                        pair.local.port,
                        candidate_label(pair.remote.candidate_type),
                        pair.remote.address,
                        pair.priority
                    ),
                )
            })
            .collect()
    };
    Ok(ServiceOutput::lines(
        format!("CANDIDATE PAIRS · {}", username),
        lines,
    ))
}

async fn ice_check(username: String) -> Result<ServiceOutput> {
    let peer_session = ConnectionManager::get_peer(&username)?;
    let current_session = crate::session::manager::get_current_session()?;
    let sender = current_session.username;
    let remote = PeerDiscoveryManager::fetch_remote_candidates(&username).await?;
    let local = DiscoveryManager::load_candidates()?;
    let mut pairs = ConnectivityManager::build_candidate_pairs(&local, &remote);
    ConnectivityManager::prioritize_pairs(&mut pairs);
    ConnectivityManager::run_checks(&mut pairs, &sender, &username).await?;
    let working = ConnectivityManager::select_working_pair(&pairs)
        .ok_or_else(|| anyhow!("ICE check failed: no reachable candidate pairs."))?;

    let mut sessions = load_ice_sessions()?;
    sessions.retain(|s| !s.peer.eq_ignore_ascii_case(&username));
    sessions.push(IceSession {
        peer: peer_session.username.clone(),
        selected_pair: working.clone(),
        established_at: Utc::now().to_rfc3339(),
    });
    save_ice_sessions(&sessions)?;

    Ok(ServiceOutput::lines(
        "ICE CONNECTIVITY SUCCESSFUL".to_string(),
        vec![
            (
                "Pair".to_string(),
                format!(
                    "{} ↔ {}",
                    candidate_label(working.local.candidate_type),
                    candidate_label(working.remote.candidate_type)
                ),
            ),
            (
                "Local".to_string(),
                format!("{}:{}", working.local.address, working.local.port),
            ),
            (
                "Remote".to_string(),
                format!("{}:{}", working.remote.address, working.remote.port),
            ),
            ("State".to_string(), "CONNECTED".to_string()),
        ],
    ))
}

async fn punch(username: String) -> Result<ServiceOutput> {
    let peer_session = ConnectionManager::get_peer(&username)?;
    let current_session = crate::session::manager::get_current_session()?;
    let sender = current_session.username;
    let ice_sessions = load_ice_sessions()?;
    let ice_session = ice_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(&username))
        .ok_or_else(|| anyhow!("No candidate pair selected. Run ICE check first."))?;
    let selected_pair = ice_session.selected_pair.clone();
    let _ = PeerDiscoveryManager::fetch_remote_candidates(&username).await?;

    let coordinator = HolePunchCoordinator::new(
        &current_session.session_id,
        &sender,
        &peer_session.username,
        selected_pair,
    )
    .await?;
    let punch_session = coordinator.start_punch_quiet().await?;

    if punch_session.state == PunchState::Established {
        Ok(ServiceOutput::message(format!(
            "UDP hole punch established with '{}' after {} attempt(s).",
            punch_session.peer, punch_session.attempts
        )))
    } else {
        Err(anyhow!(
            "UDP hole punching failed for '{}' after {} attempt(s).",
            punch_session.peer,
            punch_session.attempts
        ))
    }
}

fn punch_status(username: String) -> Result<ServiceOutput> {
    let sessions = load_punch_sessions()?;
    let session = sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(&username))
        .ok_or_else(|| anyhow!("No punch session found for '{}'.", username))?;
    Ok(ServiceOutput::lines(
        format!("PUNCH SESSION · {}", username),
        vec![
            ("State".to_string(), session.state.to_string()),
            ("Attempts".to_string(), session.attempts.to_string()),
            (
                "Local".to_string(),
                format!(
                    "{}:{}",
                    session.selected_pair.local.address, session.selected_pair.local.port
                ),
            ),
            (
                "Remote".to_string(),
                format!(
                    "{}:{}",
                    session.selected_pair.remote.address, session.selected_pair.remote.port
                ),
            ),
            ("Started".to_string(), session.started_at.clone()),
        ],
    ))
}

async fn fetch_candidates(username: String) -> Result<ServiceOutput> {
    let response = CandidateManager::fetch_remote(&username).await?;
    let mut lines: Vec<(String, String)> = response
        .candidates
        .iter()
        .map(|c| (c.to_display_string(), format!("priority {}", c.priority)))
        .collect();
    if lines.is_empty() {
        lines.push(("Candidates".to_string(), "None published".to_string()));
    }
    Ok(ServiceOutput::lines(
        format!("REMOTE CANDIDATES · {}", username),
        lines,
    ))
}

fn test_udp() -> Result<()> {
    let port = 12345;
    let remote_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut transport = UdpTransport::new(port, Some(remote_addr));
    transport.bind(port)?;

    let packet = Packet {
        version: 1,
        packet_type: PacketType::Ping,
        message_id: Uuid::new_v4(),
        sender: "test-sender".to_string(),
        recipient: "test-recipient".to_string(),
        timestamp: Utc::now(),
        nonce: "test-nonce".to_string(),
        encrypted_payload: "test-payload".to_string(),
        signature: "test-signature".to_string(),
    };
    use crate::transport::r#trait::Transport;
    transport.send(packet.clone())?;
    let received = transport
        .receive()?
        .ok_or_else(|| anyhow!("No packet received."))?;
    if received.message_id != packet.message_id {
        return Err(anyhow!("Received packet did not match."));
    }
    Ok(())
}

/// Used to guarantee an identity exists before network actions.
pub fn require_identity() -> Result<()> {
    let _ = get_storage_dir()?;
    let _ = load_contacts()?;
    read_profile()
        .map(|_| ())
        .map_err(|_| anyhow!("Initialize an identity first."))
}
