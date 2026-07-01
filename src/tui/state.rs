use crate::chat::models::{Direction, Message, MessageStatus};
use crate::contacts::models::{Contact, TrustLevel};
use crate::punch::session::PunchSession;

// ── Color Palette ──────────────────────────────────────────────────────────
// Background:  #1e1e1e
// Cream:       #F2D2BD  (soft warm highlight)
// Burnt:       #C04000  (accent / active)
// Wheat:       #F5DEB3  (secondary text)
// Dark red:    #8B0000  (error / blocked)
// Linen:       #EADDCA  (primary text)
// Red accent:  #FF4433  (status dot / unread badge)

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Chat,
    SetupWizard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetupStep {
    Init,
    Register,
    Online,
    PublishCandidates,
    AddContact,
    VerifyContact,
    Discover,
    Connect,
    IceCheck,
    Negotiate,
    SecureSession,
    Punch,
    Ready,
}

impl SetupStep {
    pub fn all() -> Vec<SetupStep> {
        vec![
            SetupStep::Init,
            SetupStep::Register,
            SetupStep::Online,
            SetupStep::PublishCandidates,
            SetupStep::AddContact,
            SetupStep::VerifyContact,
            SetupStep::Discover,
            SetupStep::Connect,
            SetupStep::IceCheck,
            SetupStep::Negotiate,
            SetupStep::SecureSession,
            SetupStep::Punch,
            SetupStep::Ready,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SetupStep::Init => "init",
            SetupStep::Register => "register <username>",
            SetupStep::Online => "online",
            SetupStep::PublishCandidates => "publish-candidates",
            SetupStep::AddContact => "contacts add <username>",
            SetupStep::VerifyContact => "verify <username>",
            SetupStep::Discover => "discover <username>",
            SetupStep::Connect => "connect <username>",
            SetupStep::IceCheck => "ice-check <username>",
            SetupStep::Negotiate => "negotiate <username>",
            SetupStep::SecureSession => "secure-session <username>",
            SetupStep::Punch => "punch <username>",
            SetupStep::Ready => "listen  →  send <username> \"msg\"",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SetupStep::Init => "Generate Ed25519 keypair & local profile",
            SetupStep::Register => "Claim username on identity registry",
            SetupStep::Online => "Announce presence session to registry",
            SetupStep::PublishCandidates => "Upload ICE candidates (expires 60s)",
            SetupStep::AddContact => "Fetch & store peer public key locally",
            SetupStep::VerifyContact => "Mark peer fingerprint as trusted",
            SetupStep::Discover => "Resolve peer status & candidate list",
            SetupStep::Connect => "Initialise local connection profile",
            SetupStep::IceCheck => "Run concurrent connectivity checks",
            SetupStep::Negotiate => "Derive deterministic session ID",
            SetupStep::SecureSession => "Bootstrap ChaCha20-Poly1305 keys",
            SetupStep::Punch => "Concurrent UDP hole-punch handshake",
            SetupStep::Ready => "Start listener & exchange messages",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    Done,
    Active,
    Pending,
}

pub struct AppState {
    // Identity
    pub username: Option<String>,
    pub fingerprint: Option<String>,
    pub is_online: bool,
    pub registry_url: String,

    // Screen
    pub current_screen: Screen,
    pub show_help: bool,

    // Setup wizard
    pub setup_steps: Vec<(SetupStep, StepStatus)>,
    pub wizard_scroll: usize,

    // Contacts panel
    pub contacts: Vec<Contact>,
    pub selected_contact_idx: usize,

    // Chat
    pub active_peer: Option<String>,
    pub messages: Vec<Message>,
    pub chat_scroll: usize,
    pub input_buffer: String,
    pub input_cursor: usize,

    // Connection info
    pub punch_sessions: Vec<PunchSession>,

    // Status bar
    pub status_message: Option<String>,
    pub status_is_error: bool,

    // Tick counter for animations
    pub tick: u64,
}

impl AppState {
    pub fn new() -> Self {
        // Load profile from disk (best-effort)
        let (username, fingerprint, registry_url, is_online) =
            if let Ok(profile) = crate::storage::filesystem::read_profile() {
                let online = crate::session::manager::is_online();
                (
                    profile.username,
                    profile.fingerprint,
                    profile.registry_url,
                    online,
                )
            } else {
                (
                    None,
                    None,
                    "https://user-registry-ten.vercel.app".to_string(),
                    false,
                )
            };

        // Load contacts (best-effort)
        let contacts = crate::contacts::manager::load_contacts().unwrap_or_default();

        // Load punch sessions (best-effort)
        let punch_sessions = crate::punch::session::load_punch_sessions().unwrap_or_default();

        // Determine wizard step statuses
        let setup_steps = Self::compute_wizard_steps(
            &username,
            is_online,
            &contacts,
            &punch_sessions,
        );

        // Pick first contact as active peer
        let active_peer = contacts.first().map(|c| c.username.clone());
        let messages = if let Some(ref peer) = active_peer {
            crate::chat::storage::load_messages(peer).unwrap_or_default()
        } else {
            Vec::new()
        };
        let chat_scroll = messages.len().saturating_sub(1);

        AppState {
            username,
            fingerprint,
            is_online,
            registry_url,
            current_screen: Screen::Dashboard,
            show_help: false,
            setup_steps,
            wizard_scroll: 0,
            contacts,
            selected_contact_idx: 0,
            active_peer,
            messages,
            chat_scroll,
            input_buffer: String::new(),
            input_cursor: 0,
            punch_sessions,
            status_message: None,
            status_is_error: false,
            tick: 0,
        }
    }

    fn compute_wizard_steps(
        username: &Option<String>,
        is_online: bool,
        contacts: &[Contact],
        punch_sessions: &[PunchSession],
    ) -> Vec<(SetupStep, StepStatus)> {
        let initialized = username.is_some();
        let registered = username.is_some();
        let online = is_online;
        let has_candidates = {
            // Check if punch sessions exist as proxy for candidates published
            let storage = crate::storage::filesystem::get_storage_dir().ok();
            storage
                .map(|d| d.join("candidates.json").exists())
                .unwrap_or(false)
        };
        let has_contacts = !contacts.is_empty();
        let has_verified = contacts.iter().any(|c| c.trust_level == TrustLevel::Verified);
        let has_punch = !punch_sessions.is_empty();
        let punch_established = punch_sessions
            .iter()
            .any(|s| s.state == crate::punch::state::PunchState::Established);

        // Connection session file exists?
        let has_connection = {
            let storage = crate::storage::filesystem::get_storage_dir().ok();
            storage
                .map(|d| d.join("connections.json").exists())
                .unwrap_or(false)
        };
        let has_secure = {
            let storage = crate::storage::filesystem::get_storage_dir().ok();
            storage
                .map(|d| d.join("secure_sessions.json").exists())
                .unwrap_or(false)
        };
        let has_ice = {
            let storage = crate::storage::filesystem::get_storage_dir().ok();
            storage
                .map(|d| d.join("ice_results.json").exists())
                .unwrap_or(false)
        };

        let steps_done: &[bool] = &[
            initialized,                // Init
            registered,                 // Register
            online,                     // Online
            has_candidates,             // PublishCandidates
            has_contacts,               // AddContact
            has_verified,               // VerifyContact
            has_contacts,               // Discover (proxy)
            has_connection,             // Connect
            has_ice,                    // IceCheck
            has_ice,                    // Negotiate (proxy)
            has_secure,                 // SecureSession
            has_punch,                  // Punch
            punch_established,          // Ready
        ];

        let mut found_active = false;
        SetupStep::all()
            .into_iter()
            .zip(steps_done.iter())
            .map(|(step, &done)| {
                if done {
                    (step, StepStatus::Done)
                } else if !found_active {
                    found_active = true;
                    (step, StepStatus::Active)
                } else {
                    (step, StepStatus::Pending)
                }
            })
            .collect()
    }

    pub fn refresh(&mut self) {
        // Reload profile
        if let Ok(profile) = crate::storage::filesystem::read_profile() {
            self.username = profile.username;
            self.fingerprint = profile.fingerprint;
            self.registry_url = profile.registry_url;
        }
        self.is_online = crate::session::manager::is_online();
        self.contacts = crate::contacts::manager::load_contacts().unwrap_or_default();
        self.punch_sessions = crate::punch::session::load_punch_sessions().unwrap_or_default();
        self.setup_steps = Self::compute_wizard_steps(
            &self.username,
            self.is_online,
            &self.contacts,
            &self.punch_sessions,
        );
        // Reload messages for active peer
        if let Some(ref peer) = self.active_peer {
            self.messages = crate::chat::storage::load_messages(peer).unwrap_or_default();
            // Auto-scroll to bottom on refresh
            if !self.messages.is_empty() {
                self.chat_scroll = self.messages.len() - 1;
            }
        }
    }

    pub fn select_contact(&mut self, idx: usize) {
        if idx < self.contacts.len() {
            self.selected_contact_idx = idx;
            let peer = self.contacts[idx].username.clone();
            self.messages = crate::chat::storage::load_messages(&peer).unwrap_or_default();
            self.active_peer = Some(peer);
            self.chat_scroll = self.messages.len().saturating_sub(1);
        }
    }

    pub fn next_contact(&mut self) {
        if !self.contacts.is_empty() {
            self.selected_contact_idx =
                (self.selected_contact_idx + 1) % self.contacts.len();
            self.select_contact(self.selected_contact_idx);
        }
    }

    pub fn prev_contact(&mut self) {
        if !self.contacts.is_empty() {
            self.selected_contact_idx = self.selected_contact_idx
                .checked_sub(1)
                .unwrap_or(self.contacts.len() - 1);
            self.select_contact(self.selected_contact_idx);
        }
    }

    pub fn scroll_chat_up(&mut self) {
        self.chat_scroll = self.chat_scroll.saturating_sub(1);
    }

    pub fn scroll_chat_down(&mut self) {
        if !self.messages.is_empty() && self.chat_scroll < self.messages.len() - 1 {
            self.chat_scroll += 1;
        }
    }

    pub fn active_step_command(&self) -> Option<String> {
        self.setup_steps
            .iter()
            .find(|(_, status)| *status == StepStatus::Active)
            .map(|(step, _)| format!("rust-messenger {}", step.label()))
    }

    pub fn connection_status_for_peer(&self, peer: &str) -> &'static str {
        let session = self.punch_sessions.iter().find(|s| s.peer.eq_ignore_ascii_case(peer));
        match session {
            Some(s) if s.state == crate::punch::state::PunchState::Established => "CONNECTED",
            Some(_) => "PUNCHING",
            None => "OFFLINE",
        }
    }

    pub fn unread_count_for(&self, peer: &str) -> usize {
        crate::chat::storage::load_messages(peer)
            .unwrap_or_default()
            .iter()
            .filter(|m| {
                m.direction == Direction::Incoming && m.status != MessageStatus::Read
            })
            .count()
    }
}
