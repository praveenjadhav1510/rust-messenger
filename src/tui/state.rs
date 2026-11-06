use crate::candidates::manager::CandidateManager;
use crate::candidates::publication::CandidatePublication;
use crate::chat::conversation::list_conversations;
use crate::chat::models::{Direction, Message, MessageStatus};
use crate::chat::storage::load_messages;
use crate::config::profile::Profile;
use crate::connection::manager::load_sessions;
use crate::connection::models::PeerSession;
use crate::contacts::manager::load_contacts;
use crate::contacts::models::{Contact, TrustLevel};
use crate::ice::connectivity::{IceSession, load_ice_sessions};
use crate::messaging::manager::MessageManager;
use crate::peer::capabilities::{PeerCapabilities, load_local_capabilities};
use crate::protocol::requests::{MessageRequest, load_requests};
use crate::punch::session::{PunchSession, load_punch_sessions};
use crate::punch::state::PunchState;
use crate::registry::models::{SearchResult, UserLookupResponse};
use crate::secure::session::{SecureSession, load_secure_sessions};
use crate::session::manager::is_online;
use crate::storage::filesystem::{get_storage_dir, read_profile};
use crate::tui::actions::{Command, FormKind, FormState, RunRequest};

// ── Palette ────────────────────────────────────────────────────────────────
pub const BG: ratatui::style::Color = ratatui::style::Color::Rgb(20, 20, 22);
pub const PANEL_BG: ratatui::style::Color = ratatui::style::Color::Rgb(26, 26, 29);
pub const PANEL_ALT: ratatui::style::Color = ratatui::style::Color::Rgb(32, 31, 34);
pub const BORDER: ratatui::style::Color = ratatui::style::Color::Rgb(70, 62, 58);
pub const CREAM: ratatui::style::Color = ratatui::style::Color::Rgb(242, 210, 189);
pub const LINEN: ratatui::style::Color = ratatui::style::Color::Rgb(234, 221, 202);
pub const WHEAT: ratatui::style::Color = ratatui::style::Color::Rgb(245, 222, 179);
pub const BURNT: ratatui::style::Color = ratatui::style::Color::Rgb(220, 104, 46);
pub const DARK_RED: ratatui::style::Color = ratatui::style::Color::Rgb(170, 40, 40);
pub const RED_ACCENT: ratatui::style::Color = ratatui::style::Color::Rgb(255, 92, 72);
pub const GREEN: ratatui::style::Color = ratatui::style::Color::Rgb(122, 196, 140);
pub const DIM: ratatui::style::Color = ratatui::style::Color::Rgb(96, 88, 82);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Chat,
    Contacts,
    Network,
    Registry,
    Messages,
    Settings,
}

impl Screen {
    pub const ALL: [Screen; 7] = [
        Screen::Dashboard,
        Screen::Chat,
        Screen::Contacts,
        Screen::Network,
        Screen::Registry,
        Screen::Messages,
        Screen::Settings,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Chat => "Chat",
            Screen::Contacts => "Contacts",
            Screen::Network => "Network",
            Screen::Registry => "Registry",
            Screen::Messages => "Messages",
            Screen::Settings => "Settings",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagesTab {
    Inbox,
    Unread,
    Requests,
}

#[derive(Debug, Clone)]
pub struct ChatRow {
    pub username: String,
    pub total: usize,
    pub unread: usize,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub trust: TrustLevel,
    pub account_status: String,
}

#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub label: &'static str,
    pub detail: &'static str,
    pub done: bool,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub created_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub text: String,
    pub kind: LogKind,
    pub at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone)]
pub struct InfoState {
    pub title: String,
    pub lines: Vec<String>,
    pub scroll: u16,
}

#[derive(Debug, Clone)]
pub struct SecretState {
    pub title: String,
    pub code: String,
    pub warning: String,
    pub revealed: bool,
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub phrase: Option<String>,
    pub input: String,
    pub request: RunRequest,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Help,
    Info(InfoState),
    Secret(SecretState),
    Form(FormState),
    Confirm(ConfirmState),
    TaskLog,
}

#[derive(Debug, Clone)]
pub struct PaletteState {
    pub query: String,
    pub selected: usize,
}

pub struct AppState {
    // identity
    pub profile: Option<Profile>,
    pub online: bool,
    pub session_id: Option<String>,
    pub storage_dir: String,

    // navigation
    pub screen: Screen,
    pub should_quit: bool,

    // data
    pub contacts: Vec<Contact>,
    pub contact_index: usize,
    pub chats: Vec<ChatRow>,
    pub chat_index: usize,
    pub messages: Vec<Message>,
    pub chat_offset: usize,
    pub chat_focus: bool,
    pub composer: String,
    pub peers: Vec<PeerSession>,
    pub peer_index: usize,
    pub punch_sessions: Vec<PunchSession>,
    pub secure_sessions: Vec<SecureSession>,
    pub ice_sessions: Vec<IceSession>,
    pub publication: Option<CandidatePublication>,
    pub capabilities: Option<PeerCapabilities>,
    pub requests: Vec<MessageRequest>,
    pub request_index: usize,
    pub unread: Vec<(String, Message)>,
    pub messages_tab: MessagesTab,
    pub messages_index: usize,

    // registry
    pub search_query: String,
    pub search_focus: bool,
    pub search_results: Vec<SearchResult>,
    pub search_index: usize,
    pub lookup: Option<UserLookupResponse>,

    // dashboard
    pub checklist: Vec<ChecklistItem>,
    pub checklist_index: usize,

    // overlays
    pub modal: Option<Modal>,
    pub palette: Option<PaletteState>,
    pub toasts: Vec<Toast>,
    pub task_log: Vec<LogEntry>,

    // runtime
    pub listener_running: bool,
    pub heartbeat_running: bool,
    pub running_tasks: usize,
    pub tick: u64,
}

impl AppState {
    pub fn new() -> Self {
        let mut state = Self {
            profile: None,
            online: false,
            session_id: None,
            storage_dir: String::new(),
            screen: Screen::Dashboard,
            should_quit: false,
            contacts: Vec::new(),
            contact_index: 0,
            chats: Vec::new(),
            chat_index: 0,
            messages: Vec::new(),
            chat_offset: 0,
            chat_focus: false,
            composer: String::new(),
            peers: Vec::new(),
            peer_index: 0,
            punch_sessions: Vec::new(),
            secure_sessions: Vec::new(),
            ice_sessions: Vec::new(),
            publication: None,
            capabilities: None,
            requests: Vec::new(),
            request_index: 0,
            unread: Vec::new(),
            messages_tab: MessagesTab::Inbox,
            messages_index: 0,
            search_query: String::new(),
            search_focus: false,
            search_results: Vec::new(),
            search_index: 0,
            lookup: None,
            checklist: Vec::new(),
            checklist_index: 0,
            modal: None,
            palette: None,
            toasts: Vec::new(),
            task_log: Vec::new(),
            listener_running: false,
            heartbeat_running: false,
            running_tasks: 0,
            tick: 0,
        };
        state.refresh_all();
        state
    }

    // ── Identity helpers ────────────────────────────────────────────────
    pub fn username(&self) -> String {
        self.profile
            .as_ref()
            .and_then(|p| p.username.clone())
            .unwrap_or_else(|| "not registered".to_string())
    }

    pub fn fingerprint(&self) -> String {
        self.profile
            .as_ref()
            .and_then(|p| p.fingerprint.clone())
            .unwrap_or_else(|| "—".to_string())
    }

    pub fn registry_url(&self) -> String {
        self.profile
            .as_ref()
            .map(|p| p.registry_url.clone())
            .unwrap_or_else(|| "https://user-registry-ten.vercel.app".to_string())
    }

    pub fn has_identity(&self) -> bool {
        self.profile.is_some()
    }

    pub fn is_registered(&self) -> bool {
        self.profile
            .as_ref()
            .and_then(|p| p.username.as_ref())
            .map(|u| !u.is_empty())
            .unwrap_or(false)
    }

    pub fn current_contact(&self) -> Option<&Contact> {
        self.contacts.get(self.contact_index)
    }

    pub fn current_contact_name(&self) -> Option<String> {
        self.current_contact().map(|c| c.username.clone())
    }

    pub fn current_peer(&self) -> Option<&PeerSession> {
        let name = self.current_network_target()?;
        self.peer_for(&name)
    }

    pub fn current_peer_name(&self) -> Option<String> {
        self.current_peer().map(|p| p.username.clone())
    }

    /// Find an active peer session by username.
    pub fn peer_for(&self, name: &str) -> Option<&PeerSession> {
        self.peers
            .iter()
            .find(|peer| peer.username.eq_ignore_ascii_case(name))
    }

    /// Names available on the Network screen: active peer sessions first,
    /// followed by contacts that do not have a session yet. This lets every
    /// network action be started from the TUI even before a session exists.
    pub fn network_targets(&self) -> Vec<String> {
        let mut targets: Vec<String> = Vec::new();
        for peer in &self.peers {
            if !targets
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&peer.username))
            {
                targets.push(peer.username.clone());
            }
        }
        for contact in &self.contacts {
            if !targets
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&contact.username))
            {
                targets.push(contact.username.clone());
            }
        }
        targets
    }

    pub fn current_network_target(&self) -> Option<String> {
        self.network_targets().get(self.peer_index).cloned()
    }

    pub fn current_chat(&self) -> Option<&ChatRow> {
        self.chats.get(self.chat_index)
    }

    pub fn current_chat_name(&self) -> Option<String> {
        self.current_chat().map(|c| c.username.clone())
    }

    pub fn current_request(&self) -> Option<&MessageRequest> {
        let index = if self.screen == Screen::Messages && self.messages_tab == MessagesTab::Requests
        {
            self.messages_index
        } else {
            self.request_index
        };
        self.requests.get(index)
    }

    /// The best target for a contact/peer action. The current screen wins so
    /// network screens never accidentally target an unrelated chat.
    pub fn action_target(&self) -> Option<String> {
        match self.screen {
            Screen::Network => self
                .current_network_target()
                .or_else(|| self.current_contact_name())
                .or_else(|| self.current_chat_name()),
            Screen::Chat | Screen::Messages => self
                .current_chat_name()
                .or_else(|| self.current_contact_name())
                .or_else(|| self.current_peer_name()),
            _ => self
                .current_contact_name()
                .or_else(|| self.current_chat_name())
                .or_else(|| self.current_peer_name()),
        }
    }

    /// Target for commands that only make sense on a stored contact.
    ///
    /// On the Contacts screen the selection is authoritative. Elsewhere the
    /// generic action target is used, but only when it really is a contact, so
    /// a chat or peer session can never be passed to a contact mutation.
    pub fn contact_target(&self) -> Option<String> {
        if self.screen == Screen::Contacts {
            return self.current_contact_name();
        }
        let name = self.action_target()?;
        self.contacts
            .iter()
            .any(|contact| contact.username.eq_ignore_ascii_case(&name))
            .then_some(name)
    }

    pub fn punch_state_for(&self, peer: &str) -> Option<PunchState> {
        self.punch_sessions
            .iter()
            .find(|s| s.peer.eq_ignore_ascii_case(peer))
            .map(|s| s.state)
    }

    pub fn is_connected(&self, peer: &str) -> bool {
        self.punch_state_for(peer) == Some(PunchState::Established)
            || self.peers.iter().any(|p| {
                p.username.eq_ignore_ascii_case(peer) && p.state.to_string() == "CONNECTED"
            })
    }

    pub fn has_ice(&self, peer: &str) -> bool {
        self.ice_sessions
            .iter()
            .any(|s| s.peer.eq_ignore_ascii_case(peer))
    }

    pub fn has_secure(&self, peer: &str) -> bool {
        self.secure_sessions
            .iter()
            .any(|s| s.peer.eq_ignore_ascii_case(peer) && s.established)
    }

    pub fn unread_count(&self, peer: &str) -> usize {
        load_messages(peer)
            .unwrap_or_default()
            .iter()
            .filter(|m| m.direction == Direction::Incoming && m.status != MessageStatus::Read)
            .count()
    }

    pub fn publication_seconds_left(&self) -> Option<i64> {
        let publ = self.publication.as_ref()?;
        let expires = chrono::DateTime::parse_from_rfc3339(&publ.expires_at).ok()?;
        Some((expires.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_seconds())
    }

    // ── Refresh ─────────────────────────────────────────────────────────
    pub fn refresh_all(&mut self) {
        self.profile = read_profile().ok();
        self.online = is_online();
        self.session_id = crate::session::manager::load_session()
            .ok()
            .map(|s| s.session_id);

        self.contacts = load_contacts().unwrap_or_default();
        self.peers = load_sessions().unwrap_or_default();
        self.punch_sessions = load_punch_sessions().unwrap_or_default();
        self.secure_sessions = load_secure_sessions().unwrap_or_default();
        self.ice_sessions = load_ice_sessions().unwrap_or_default();
        self.publication = CandidateManager::load_publication().ok();
        self.capabilities = load_local_capabilities().ok();
        self.requests = load_requests().unwrap_or_default();
        self.unread = MessageManager::get_unread_messages().unwrap_or_default();
        self.storage_dir = get_storage_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~/.rust-messenger".to_string());

        self.rebuild_chats();
        self.reload_messages();
        self.rebuild_checklist();
        self.clamp_selections();
    }

    fn rebuild_chats(&mut self) {
        let mut rows = Vec::new();
        for summary in list_conversations().unwrap_or_default() {
            let contact = self
                .contacts
                .iter()
                .find(|c| c.username.eq_ignore_ascii_case(&summary.username));
            let unread = load_messages(&summary.username)
                .unwrap_or_default()
                .iter()
                .filter(|m| m.direction == Direction::Incoming && m.status != MessageStatus::Read)
                .count();
            rows.push(ChatRow {
                username: summary.username,
                total: summary.total_messages,
                unread,
                last_activity: summary.last_activity,
                trust: contact
                    .map(|c| c.trust_level)
                    .unwrap_or(TrustLevel::Unverified),
                account_status: contact
                    .map(|c| c.account_status.clone())
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
            });
        }

        // Contacts without a conversation still deserve a chat row.
        for contact in &self.contacts {
            if !rows
                .iter()
                .any(|r| r.username.eq_ignore_ascii_case(&contact.username))
            {
                rows.push(ChatRow {
                    username: contact.username.clone(),
                    total: 0,
                    unread: 0,
                    last_activity: None,
                    trust: contact.trust_level,
                    account_status: contact.account_status.clone(),
                });
            }
        }

        rows.sort_by(|a, b| match (a.last_activity, b.last_activity) {
            (Some(x), Some(y)) => y.cmp(&x),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.username.cmp(&b.username),
        });

        let current = self.current_chat_name();
        self.chats = rows;
        if let Some(name) = current
            && let Some(idx) = self
                .chats
                .iter()
                .position(|c| c.username.eq_ignore_ascii_case(&name))
        {
            self.chat_index = idx;
        }
    }

    pub fn reload_messages(&mut self) {
        let peer = self.current_chat_name();
        if let Some(peer) = peer {
            self.messages = load_messages(&peer).unwrap_or_default();
            let max_offset = self.messages.len().saturating_sub(1);
            self.chat_offset = self.chat_offset.min(max_offset);
        } else {
            self.messages.clear();
            self.chat_offset = 0;
        }
    }

    pub fn open_chat(&mut self, peer: &str, focus_composer: bool) {
        if let Some(idx) = self
            .chats
            .iter()
            .position(|c| c.username.eq_ignore_ascii_case(peer))
        {
            self.chat_index = idx;
        } else if let Some(contact) = self
            .contacts
            .iter()
            .find(|c| c.username.eq_ignore_ascii_case(peer))
        {
            self.chats.push(ChatRow {
                username: contact.username.clone(),
                total: 0,
                unread: 0,
                last_activity: None,
                trust: contact.trust_level,
                account_status: contact.account_status.clone(),
            });
            self.chat_index = self.chats.len() - 1;
        }
        self.screen = Screen::Chat;
        self.chat_offset = 0;
        self.chat_focus = focus_composer;
        self.reload_messages();
    }

    fn rebuild_checklist(&mut self) {
        let peer = self.action_target();
        let peer_connected = peer
            .as_deref()
            .map(|p| self.is_connected(p))
            .unwrap_or(false);
        let peer_ice = peer.as_deref().map(|p| self.has_ice(p)).unwrap_or(false);
        let publication_ok = self
            .publication_seconds_left()
            .map(|s| s > 0)
            .unwrap_or(false);
        let verified = self
            .contacts
            .iter()
            .any(|c| c.trust_level == TrustLevel::Verified);
        let punch_established = self
            .punch_sessions
            .iter()
            .any(|s| s.state == PunchState::Established);

        self.checklist = vec![
            ChecklistItem {
                label: "Initialize identity",
                detail: "Generate Ed25519 keys and local profile",
                done: self.has_identity(),
                command: Command::Init,
            },
            ChecklistItem {
                label: "Register username",
                detail: "Claim your name on the registry",
                done: self.is_registered(),
                command: Command::Register,
            },
            ChecklistItem {
                label: "Go online",
                detail: "Announce presence and start heartbeat",
                done: self.online,
                command: Command::Online,
            },
            ChecklistItem {
                label: "Publish ICE candidates",
                detail: "Upload network candidates (expires in 60s)",
                done: publication_ok,
                command: Command::PublishCandidates,
            },
            ChecklistItem {
                label: "Add a contact",
                detail: "Fetch and store a peer public key",
                done: !self.contacts.is_empty(),
                command: Command::AddContact,
            },
            ChecklistItem {
                label: "Verify a contact",
                detail: "Mark a fingerprint as trusted",
                done: verified,
                command: Command::Verify,
            },
            ChecklistItem {
                label: "Run ICE check",
                detail: "Find a reachable candidate pair",
                done: peer_ice,
                command: Command::IceCheck,
            },
            ChecklistItem {
                label: "Establish connection",
                detail: "Secure session + UDP hole punch",
                done: peer_connected && punch_established,
                command: Command::Punch,
            },
            ChecklistItem {
                label: "Start listener",
                detail: "Receive messages in the background",
                done: self.listener_running,
                command: Command::Listen,
            },
            ChecklistItem {
                label: "Open a chat",
                detail: "Send an end-to-end encrypted message",
                done: !self.messages.is_empty(),
                command: Command::SendEncrypted,
            },
        ];
        self.checklist_index = self
            .checklist_index
            .min(self.checklist.len().saturating_sub(1));
    }

    fn clamp_selections(&mut self) {
        if !self.contacts.is_empty() {
            self.contact_index = self.contact_index.min(self.contacts.len() - 1);
        } else {
            self.contact_index = 0;
        }
        let target_len = self.network_targets().len();
        if target_len > 0 {
            self.peer_index = self.peer_index.min(target_len - 1);
        } else {
            self.peer_index = 0;
        }
        if !self.chats.is_empty() {
            self.chat_index = self.chat_index.min(self.chats.len() - 1);
        } else {
            self.chat_index = 0;
        }
        if !self.requests.is_empty() {
            self.request_index = self.request_index.min(self.requests.len() - 1);
        } else {
            self.request_index = 0;
        }
        if !self.search_results.is_empty() {
            self.search_index = self.search_index.min(self.search_results.len() - 1);
        } else {
            self.search_index = 0;
        }
        let message_len = self.message_row_count();
        if message_len > 0 {
            self.messages_index = self.messages_index.min(message_len - 1);
        } else {
            self.messages_index = 0;
        }
    }

    pub fn message_row_count(&self) -> usize {
        match self.messages_tab {
            MessagesTab::Inbox => self.chats.len(),
            MessagesTab::Unread => self.unread.len(),
            MessagesTab::Requests => self.requests.len(),
        }
    }

    // ── Toasts / logs ───────────────────────────────────────────────────
    pub fn toast(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast {
            text: text.into(),
            kind,
            created_tick: self.tick,
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub fn expire_toasts(&mut self) {
        self.toasts
            .retain(|t| self.tick.saturating_sub(t.created_tick) < 60);
    }

    pub fn log(&mut self, text: impl Into<String>, kind: LogKind) {
        self.task_log.push(LogEntry {
            text: text.into(),
            kind,
            at: chrono::Local::now(),
        });
        if self.task_log.len() > 250 {
            self.task_log.remove(0);
        }
    }

    pub fn open_form(&mut self, kind: FormKind, prefill: Option<&str>) {
        self.modal = Some(Modal::Form(FormState::new(kind, prefill)));
    }

    pub fn open_info(&mut self, title: impl Into<String>, lines: Vec<String>) {
        self.modal = Some(Modal::Info(InfoState {
            title: title.into(),
            lines,
            scroll: 0,
        }));
    }

    pub fn open_confirm(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        phrase: Option<&str>,
        request: RunRequest,
    ) {
        self.modal = Some(Modal::Confirm(ConfirmState {
            title: title.into(),
            message: message.into(),
            phrase: phrase.map(|s| s.to_string()),
            input: String::new(),
            request,
        }));
    }

    pub fn task_log_lines(&self) -> Vec<String> {
        if self.task_log.is_empty() {
            return vec!["No activity yet.".to_string()];
        }
        self.task_log
            .iter()
            .rev()
            .take(120)
            .rev()
            .map(|e| format!("{}  {}", e.at.format("%H:%M:%S"), e.text))
            .collect()
    }
}
