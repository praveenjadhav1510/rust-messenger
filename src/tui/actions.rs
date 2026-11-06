use crate::contacts::models::TrustLevel;

/// Every user-facing operation exposed by the TUI. Used by the command
/// palette, the dashboard checklist and the keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Init,
    Whoami,
    Register,
    Search,
    Lookup,
    Recover,
    Rename,
    RemoveAccount,
    Restore,
    Lock,
    Unlock,
    Status,
    AddContact,
    ShowContact,
    RemoveContact,
    Verify,
    Unverify,
    Block,
    Unblock,
    Requests,
    AcceptRequest,
    RejectRequest,
    Inbox,
    Unread,
    MessageHistory,
    MessageClear,
    ConversationShow,
    Online,
    Offline,
    Connect,
    Disconnect,
    Peers,
    Ping,
    Netinfo,
    Discover,
    Negotiate,
    Capabilities,
    SecureSession,
    TestUdp,
    CandidatePairs,
    IceCheck,
    Punch,
    PunchStatus,
    PublishCandidates,
    RefreshCandidates,
    FetchCandidates,
    Listen,
    SendEncrypted,
    QueueMessage,
    DevInject,
}

impl Command {
    pub fn label(self) -> &'static str {
        match self {
            Command::Init => "Initialize identity",
            Command::Whoami => "Show identity",
            Command::Register => "Register username",
            Command::Search => "Search registry",
            Command::Lookup => "Look up user",
            Command::Recover => "Recover account",
            Command::Rename => "Rename account",
            Command::RemoveAccount => "Delete account",
            Command::Restore => "Restore account",
            Command::Lock => "Lock account",
            Command::Unlock => "Unlock account",
            Command::Status => "Query presence",
            Command::AddContact => "Add contact",
            Command::ShowContact => "Show contact",
            Command::RemoveContact => "Remove contact",
            Command::Verify => "Verify contact",
            Command::Unverify => "Unverify contact",
            Command::Block => "Block contact",
            Command::Unblock => "Unblock contact",
            Command::Requests => "Message requests",
            Command::AcceptRequest => "Accept request",
            Command::RejectRequest => "Reject request",
            Command::Inbox => "Inbox",
            Command::Unread => "Unread messages",
            Command::MessageHistory => "Open conversation",
            Command::MessageClear => "Clear conversation",
            Command::ConversationShow => "Conversation details",
            Command::Online => "Go online",
            Command::Offline => "Go offline",
            Command::Connect => "Connect to peer",
            Command::Disconnect => "Disconnect peer",
            Command::Peers => "List peer sessions",
            Command::Ping => "Ping peer",
            Command::Netinfo => "Discover network / NAT",
            Command::Discover => "Discover peer",
            Command::Negotiate => "Negotiate connection",
            Command::Capabilities => "Show capabilities",
            Command::SecureSession => "Establish secure session",
            Command::TestUdp => "Test UDP loopback",
            Command::CandidatePairs => "List candidate pairs",
            Command::IceCheck => "Run ICE check",
            Command::Punch => "UDP hole punch",
            Command::PunchStatus => "Punch status",
            Command::PublishCandidates => "Publish ICE candidates",
            Command::RefreshCandidates => "Refresh ICE candidates",
            Command::FetchCandidates => "Fetch peer candidates",
            Command::Listen => "Toggle message listener",
            Command::SendEncrypted => "Send encrypted message",
            Command::QueueMessage => "Queue local message",
            Command::DevInject => "Inject test message",
        }
    }

    /// Extra search terms so the palette can be driven with CLI command names
    /// and common synonyms, not just the display label.
    pub fn keywords(self) -> &'static str {
        match self {
            Command::Init => "init initialize identity setup keys",
            Command::Whoami => "whoami identity me profile",
            Command::Register => "register username signup",
            Command::Search => "search registry find user",
            Command::Lookup => "lookup user profile public key",
            Command::Recover => "recover recovery code account keys rotate",
            Command::Rename => "rename username account",
            Command::RemoveAccount => "remove delete deactivate account",
            Command::Restore => "restore account reactivate",
            Command::Lock => "lock account",
            Command::Unlock => "unlock account",
            Command::Status => "status presence online query",
            Command::AddContact => "contacts add contact",
            Command::ShowContact => "contacts show contact info",
            Command::RemoveContact => "contacts remove delete contact",
            Command::Verify => "verify contact trusted",
            Command::Unverify => "unverify contact untrusted",
            Command::Block => "block contact",
            Command::Unblock => "unblock contact",
            Command::Requests => "requests list message requests",
            Command::AcceptRequest => "accept request",
            Command::RejectRequest => "reject request",
            Command::Inbox => "inbox message list conversations",
            Command::Unread => "unread messages",
            Command::MessageHistory => "message history conversation open",
            Command::MessageClear => "message clear conversation delete",
            Command::ConversationShow => "conversation show details",
            Command::Online => "online presence announce",
            Command::Offline => "offline",
            Command::Connect => "connect peer session",
            Command::Disconnect => "disconnect peer session",
            Command::Peers => "peers peer sessions list",
            Command::Ping => "ping peer",
            Command::Netinfo => "netinfo network nat stun interfaces",
            Command::Discover => "discover peer status capabilities",
            Command::Negotiate => "negotiate connection handshake",
            Command::Capabilities => "capabilities ice turn files",
            Command::SecureSession => "secure session encryption keys",
            Command::TestUdp => "test udp transport loopback",
            Command::CandidatePairs => "candidate pairs ice",
            Command::IceCheck => "ice check connectivity",
            Command::Punch => "punch hole punching udp",
            Command::PunchStatus => "punch status",
            Command::PublishCandidates => "publish candidates ice",
            Command::RefreshCandidates => "refresh candidates ice",
            Command::FetchCandidates => "fetch candidates remote peer",
            Command::Listen => "listen listener receive incoming",
            Command::SendEncrypted => "send message encrypted e2e",
            Command::QueueMessage => "queue message local offline",
            Command::DevInject => "dev inject simulate test message",
        }
    }

    pub fn category(self) -> &'static str {
        match self {
            Command::Init
            | Command::Whoami
            | Command::Register
            | Command::Search
            | Command::Lookup
            | Command::Recover
            | Command::Rename
            | Command::RemoveAccount
            | Command::Restore
            | Command::Lock
            | Command::Unlock
            | Command::Status => "Identity & Registry",
            Command::AddContact
            | Command::ShowContact
            | Command::RemoveContact
            | Command::Verify
            | Command::Unverify
            | Command::Block
            | Command::Unblock => "Contacts",
            Command::Requests
            | Command::AcceptRequest
            | Command::RejectRequest
            | Command::Inbox
            | Command::Unread
            | Command::MessageHistory
            | Command::MessageClear
            | Command::ConversationShow
            | Command::SendEncrypted
            | Command::QueueMessage
            | Command::DevInject => "Messages",
            Command::Online
            | Command::Offline
            | Command::Connect
            | Command::Disconnect
            | Command::Peers
            | Command::Ping
            | Command::Netinfo
            | Command::Discover
            | Command::Negotiate
            | Command::Capabilities
            | Command::SecureSession
            | Command::TestUdp
            | Command::CandidatePairs
            | Command::IceCheck
            | Command::Punch
            | Command::PunchStatus
            | Command::PublishCandidates
            | Command::RefreshCandidates
            | Command::FetchCandidates => "Network",
            Command::Listen => "Runtime",
        }
    }

    /// Commands that need a recovery code or other multi-field input open a
    /// form instead of running directly.
    pub fn form_kind(self) -> Option<FormKind> {
        match self {
            Command::Register => Some(FormKind::Register),
            Command::Lookup => Some(FormKind::Lookup),
            Command::Recover => Some(FormKind::Recover),
            Command::Rename => Some(FormKind::Rename),
            Command::RemoveAccount => Some(FormKind::RemoveAccount),
            Command::Restore => Some(FormKind::Restore),
            Command::Lock => Some(FormKind::Lock),
            Command::Unlock => Some(FormKind::Unlock),
            Command::Status => Some(FormKind::Status),
            Command::AddContact => Some(FormKind::AddContact),
            Command::Discover => Some(FormKind::PeerCommand(PeerCommand::Discover)),
            Command::Connect => Some(FormKind::PeerCommand(PeerCommand::Connect)),
            Command::Disconnect => Some(FormKind::PeerCommand(PeerCommand::Disconnect)),
            Command::Ping => Some(FormKind::PeerCommand(PeerCommand::Ping)),
            Command::Negotiate => Some(FormKind::PeerCommand(PeerCommand::Negotiate)),
            Command::SecureSession => Some(FormKind::PeerCommand(PeerCommand::SecureSession)),
            Command::CandidatePairs => Some(FormKind::PeerCommand(PeerCommand::CandidatePairs)),
            Command::IceCheck => Some(FormKind::PeerCommand(PeerCommand::IceCheck)),
            Command::Punch => Some(FormKind::PeerCommand(PeerCommand::Punch)),
            Command::PunchStatus => Some(FormKind::PeerCommand(PeerCommand::PunchStatus)),
            Command::FetchCandidates => Some(FormKind::PeerCommand(PeerCommand::FetchCandidates)),
            Command::DevInject => Some(FormKind::Inject),
            _ => None,
        }
    }

    /// Commands that target an existing contact / peer.
    pub fn needs_target(self) -> bool {
        matches!(
            self,
            Command::Status
                | Command::ShowContact
                | Command::RemoveContact
                | Command::Verify
                | Command::Unverify
                | Command::Block
                | Command::Unblock
                | Command::MessageHistory
                | Command::MessageClear
                | Command::ConversationShow
                | Command::SendEncrypted
                | Command::QueueMessage
                | Command::Discover
                | Command::Connect
                | Command::Disconnect
                | Command::Ping
                | Command::Negotiate
                | Command::SecureSession
                | Command::CandidatePairs
                | Command::IceCheck
                | Command::Punch
                | Command::PunchStatus
                | Command::FetchCandidates
        )
    }
}

pub fn palette_commands() -> Vec<Command> {
    vec![
        Command::Init,
        Command::Whoami,
        Command::Register,
        Command::Search,
        Command::Lookup,
        Command::Recover,
        Command::Rename,
        Command::RemoveAccount,
        Command::Restore,
        Command::Lock,
        Command::Unlock,
        Command::Status,
        Command::AddContact,
        Command::ShowContact,
        Command::RemoveContact,
        Command::Verify,
        Command::Unverify,
        Command::Block,
        Command::Unblock,
        Command::Requests,
        Command::AcceptRequest,
        Command::RejectRequest,
        Command::Inbox,
        Command::Unread,
        Command::MessageHistory,
        Command::MessageClear,
        Command::ConversationShow,
        Command::Online,
        Command::Offline,
        Command::Connect,
        Command::Disconnect,
        Command::Peers,
        Command::Ping,
        Command::Netinfo,
        Command::Discover,
        Command::Negotiate,
        Command::Capabilities,
        Command::SecureSession,
        Command::TestUdp,
        Command::CandidatePairs,
        Command::IceCheck,
        Command::Punch,
        Command::PunchStatus,
        Command::PublishCandidates,
        Command::RefreshCandidates,
        Command::FetchCandidates,
        Command::Listen,
        Command::SendEncrypted,
        Command::QueueMessage,
        Command::DevInject,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerCommand {
    Discover,
    Connect,
    Disconnect,
    Ping,
    Negotiate,
    SecureSession,
    CandidatePairs,
    IceCheck,
    Punch,
    PunchStatus,
    FetchCandidates,
}

impl PeerCommand {
    pub fn label(self) -> &'static str {
        match self {
            PeerCommand::Discover => "Discover peer",
            PeerCommand::Connect => "Connect to peer",
            PeerCommand::Disconnect => "Disconnect peer",
            PeerCommand::Ping => "Ping peer",
            PeerCommand::Negotiate => "Negotiate connection",
            PeerCommand::SecureSession => "Establish secure session",
            PeerCommand::CandidatePairs => "List candidate pairs",
            PeerCommand::IceCheck => "Run ICE check",
            PeerCommand::Punch => "UDP hole punch",
            PeerCommand::PunchStatus => "Punch status",
            PeerCommand::FetchCandidates => "Fetch peer candidates",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormKind {
    Register,
    Lookup,
    Recover,
    Rename,
    RemoveAccount,
    Restore,
    Lock,
    Unlock,
    Status,
    AddContact,
    PeerCommand(PeerCommand),
    Inject,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub secret: bool,
    pub hint: &'static str,
}

impl Field {
    fn new(label: &'static str, hint: &'static str, secret: bool) -> Self {
        Self {
            label,
            value: String::new(),
            secret,
            hint,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormState {
    pub kind: FormKind,
    pub title: String,
    pub description: String,
    pub fields: Vec<Field>,
    pub focus: usize,
    pub error: Option<String>,
}

impl FormState {
    pub fn new(kind: FormKind, prefill: Option<&str>) -> Self {
        let (title, description, mut fields) = match &kind {
            FormKind::Register => (
                "REGISTER USERNAME",
                "Claim a username on the identity registry. Your recovery code will be shown once.",
                vec![Field::new("Username", "3-20 chars, lowercase", false)],
            ),
            FormKind::Lookup => (
                "LOOK UP USER",
                "Fetch a user's public profile from the registry.",
                vec![Field::new("Username", "exact registry username", false)],
            ),
            FormKind::Recover => (
                "RECOVER ACCOUNT",
                "Recovery rotates your local keys. The old private key stops working.",
                vec![
                    Field::new("Username", "username to recover", false),
                    Field::new("Recovery code", "shown once during registration", true),
                ],
            ),
            FormKind::Rename => (
                "RENAME ACCOUNT",
                "Change your registered username. The recovery code is required.",
                vec![
                    Field::new("New username", "new registry username", false),
                    Field::new("Recovery code", "current recovery code", true),
                ],
            ),
            FormKind::RemoveAccount => (
                "DELETE ACCOUNT",
                "This deactivates your identity. Type DELETE to confirm.",
                vec![
                    Field::new("Recovery code", "current recovery code", true),
                    Field::new("Confirmation", "type DELETE", false),
                ],
            ),
            FormKind::Restore => (
                "RESTORE ACCOUNT",
                "Restore a deactivated identity with a fresh keypair.",
                vec![
                    Field::new("Username", "deactivated username", false),
                    Field::new("Recovery code", "recovery code", true),
                ],
            ),
            FormKind::Lock => (
                "LOCK ACCOUNT",
                "Mark this identity as untrusted until recovery is completed.",
                vec![Field::new("Recovery code", "current recovery code", true)],
            ),
            FormKind::Unlock => (
                "UNLOCK ACCOUNT",
                "Restore trust for this identity.",
                vec![Field::new("Recovery code", "current recovery code", true)],
            ),
            FormKind::Status => (
                "QUERY PRESENCE",
                "Fetch registry presence for a user.",
                vec![Field::new("Username", "registry username", false)],
            ),
            FormKind::AddContact => (
                "ADD CONTACT",
                "Fetch the peer's public key from the registry and store it locally.",
                vec![Field::new("Username", "registry username", false)],
            ),
            FormKind::PeerCommand(cmd) => (
                "PEER ACTION",
                "Run a network action against a contact.",
                vec![Field::new("Username", cmd.label(), false)],
            ),
            FormKind::Inject => (
                "INJECT TEST MESSAGE",
                "Simulate an incoming message from a local contact.",
                vec![
                    Field::new("From", "existing contact", false),
                    Field::new("Message", "message body", false),
                ],
            ),
        };

        if let Some(prefill) = prefill.filter(|value| !value.is_empty())
            && let Some(first) = fields.first_mut()
        {
            first.value = prefill.to_string();
        }

        Self {
            kind,
            title: title.to_string(),
            description: description.to_string(),
            fields,
            focus: 0,
            error: None,
        }
    }

    pub fn submit(&mut self) -> std::result::Result<RunRequest, String> {
        for field in &self.fields {
            if field.value.trim().is_empty() {
                self.error = Some(format!("{} is required.", field.label));
                return Err(self.error.clone().unwrap());
            }
        }

        let value = |i: usize| self.fields[i].value.trim().to_string();

        let request = match &self.kind {
            FormKind::Register => RunRequest::Register(value(0)),
            FormKind::Lookup => RunRequest::Lookup(value(0)),
            FormKind::Recover => RunRequest::Recover {
                username: value(0),
                code: value(1),
            },
            FormKind::Rename => RunRequest::Rename {
                new_username: value(0),
                code: value(1),
            },
            FormKind::RemoveAccount => {
                if value(1) != "DELETE" {
                    self.error = Some("Type DELETE exactly to confirm.".to_string());
                    return Err(self.error.clone().unwrap());
                }
                RunRequest::RemoveAccount { code: value(0) }
            }
            FormKind::Restore => RunRequest::Restore {
                username: value(0),
                code: value(1),
            },
            FormKind::Lock => RunRequest::Lock { code: value(0) },
            FormKind::Unlock => RunRequest::Unlock { code: value(0) },
            FormKind::Status => RunRequest::Status(value(0)),
            FormKind::AddContact => RunRequest::AddContact(value(0)),
            FormKind::PeerCommand(cmd) => match cmd {
                PeerCommand::Discover => RunRequest::Discover(value(0)),
                PeerCommand::Connect => RunRequest::Connect(value(0)),
                PeerCommand::Disconnect => RunRequest::Disconnect(value(0)),
                PeerCommand::Ping => RunRequest::Ping(value(0)),
                PeerCommand::Negotiate => RunRequest::Negotiate(value(0)),
                PeerCommand::SecureSession => RunRequest::SecureSession(value(0)),
                PeerCommand::CandidatePairs => RunRequest::CandidatePairs(value(0)),
                PeerCommand::IceCheck => RunRequest::IceCheck(value(0)),
                PeerCommand::Punch => RunRequest::Punch(value(0)),
                PeerCommand::PunchStatus => RunRequest::PunchStatus(value(0)),
                PeerCommand::FetchCandidates => RunRequest::FetchCandidates(value(0)),
            },
            FormKind::Inject => RunRequest::Inject {
                peer: value(0),
                text: value(1),
            },
        };

        Ok(request)
    }
}

/// A fully-resolved operation that the application can execute in the
/// background.
#[derive(Debug, Clone)]
pub enum RunRequest {
    Init { force: bool },
    Register(String),
    Lookup(String),
    Recover { username: String, code: String },
    Rename { new_username: String, code: String },
    RemoveAccount { code: String },
    Restore { username: String, code: String },
    Lock { code: String },
    Unlock { code: String },
    Status(String),
    Search(String),
    AddContact(String),
    RemoveContact(String),
    ShowContact(String),
    SetTrust { username: String, level: TrustLevel },
    AcceptRequest(String),
    RejectRequest(String),
    SendEncrypted { peer: String, text: String },
    QueueMessage { peer: String, text: String },
    ClearChat(String),
    MarkRead(String),
    Inject { peer: String, text: String },
    Connect(String),
    Disconnect(String),
    Ping(String),
    Netinfo,
    Capabilities,
    Discover(String),
    Negotiate(String),
    SecureSession(String),
    CandidatePairs(String),
    IceCheck(String),
    Punch(String),
    PunchStatus(String),
    PublishCandidates,
    RefreshCandidates,
    FetchCandidates(String),
    TestUdp,
    Online,
    Offline,
}

impl RunRequest {
    pub fn label(&self) -> &'static str {
        match self {
            RunRequest::Init { .. } => "Initializing identity",
            RunRequest::Register(_) => "Registering username",
            RunRequest::Lookup(_) => "Looking up user",
            RunRequest::Recover { .. } => "Recovering account",
            RunRequest::Rename { .. } => "Renaming account",
            RunRequest::RemoveAccount { .. } => "Deleting account",
            RunRequest::Restore { .. } => "Restoring account",
            RunRequest::Lock { .. } => "Locking account",
            RunRequest::Unlock { .. } => "Unlocking account",
            RunRequest::Status(_) => "Querying presence",
            RunRequest::Search(_) => "Searching registry",
            RunRequest::AddContact(_) => "Adding contact",
            RunRequest::RemoveContact(_) => "Removing contact",
            RunRequest::ShowContact(_) => "Loading contact",
            RunRequest::SetTrust { .. } => "Updating trust level",
            RunRequest::AcceptRequest(_) => "Accepting request",
            RunRequest::RejectRequest(_) => "Rejecting request",
            RunRequest::SendEncrypted { .. } => "Sending encrypted message",
            RunRequest::QueueMessage { .. } => "Queueing message",
            RunRequest::ClearChat(_) => "Clearing conversation",
            RunRequest::MarkRead(_) => "Marking messages read",
            RunRequest::Inject { .. } => "Injecting test message",
            RunRequest::Connect(_) => "Connecting to peer",
            RunRequest::Disconnect(_) => "Disconnecting peer",
            RunRequest::Ping(_) => "Pinging peer",
            RunRequest::Netinfo => "Discovering network",
            RunRequest::Capabilities => "Loading capabilities",
            RunRequest::Discover(_) => "Discovering peer",
            RunRequest::Negotiate(_) => "Negotiating connection",
            RunRequest::SecureSession(_) => "Establishing secure session",
            RunRequest::CandidatePairs(_) => "Building candidate pairs",
            RunRequest::IceCheck(_) => "Running ICE checks",
            RunRequest::Punch(_) => "Hole punching",
            RunRequest::PunchStatus(_) => "Reading punch status",
            RunRequest::PublishCandidates => "Publishing candidates",
            RunRequest::RefreshCandidates => "Refreshing candidates",
            RunRequest::FetchCandidates(_) => "Fetching peer candidates",
            RunRequest::TestUdp => "Testing UDP transport",
            RunRequest::Online => "Going online",
            RunRequest::Offline => "Going offline",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_validate_required_fields() {
        let mut form = FormState::new(FormKind::Register, None);
        assert!(form.submit().is_err());
        form.fields[0].value = "alice".to_string();
        assert!(matches!(
            form.submit(),
            Ok(RunRequest::Register(name)) if name == "alice"
        ));
    }

    #[test]
    fn remove_account_requires_exact_confirmation() {
        let mut form = FormState::new(FormKind::RemoveAccount, None);
        form.fields[0].value = "code".to_string();
        form.fields[1].value = "delete".to_string();
        assert!(form.submit().is_err());
        form.fields[1].value = "DELETE".to_string();
        assert!(form.submit().is_ok());
    }

    #[test]
    fn peer_forms_prefill_the_selected_target() {
        let form = FormState::new(FormKind::PeerCommand(PeerCommand::IceCheck), Some("zoro"));
        assert_eq!(form.fields[0].value, "zoro");
        let mut form = form;
        assert!(matches!(
            form.submit(),
            Ok(RunRequest::IceCheck(peer)) if peer == "zoro"
        ));
    }
}
