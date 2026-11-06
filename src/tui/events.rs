use crate::contacts::models::TrustLevel;
use crate::tui::actions::{Command, FormKind, RunRequest, palette_commands};
use crate::tui::state::{AppState, MessagesTab, Modal, PaletteState, Screen, ToastKind};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum Outcome {
    None,
    Quit,
    Run(RunRequest),
    Refresh,
    ToggleListener,
    CancelTasks,
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) -> Outcome {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if ctrl && key.code == KeyCode::Char('c') {
        return Outcome::Quit;
    }

    // ── Overlays ────────────────────────────────────────────────────────
    if state.palette.is_some() {
        return handle_palette(state, key);
    }

    if state.modal.is_some() {
        return handle_modal(state, key);
    }

    // ── Registry search input ───────────────────────────────────────────
    if state.search_focus {
        match key.code {
            KeyCode::Esc => {
                state.search_focus = false;
            }
            KeyCode::Enter => {
                let query = state.search_query.trim().to_string();
                state.search_focus = false;
                if query.is_empty() {
                    state.toast("Type a query first.", ToastKind::Error);
                } else {
                    return Outcome::Run(RunRequest::Search(query));
                }
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Char(c) if !ctrl => state.search_query.push(c),
            _ => {}
        }
        return Outcome::None;
    }

    // ── Global keys ─────────────────────────────────────────────────────
    // While composing a chat message, printable characters belong to the
    // composer; only modified shortcuts (Ctrl+P) and function keys stay global.
    let typing = state.screen == Screen::Chat && state.chat_focus;

    match key.code {
        KeyCode::F(5) => return Outcome::Refresh,
        KeyCode::F(6) => {
            let request = if state.online {
                RunRequest::Offline
            } else {
                RunRequest::Online
            };
            return Outcome::Run(request);
        }
        KeyCode::F(7) => return Outcome::ToggleListener,
        KeyCode::F(8) => {
            state.modal = Some(Modal::TaskLog);
            return Outcome::None;
        }
        KeyCode::F(1) => {
            state.screen = Screen::Dashboard;
            return Outcome::None;
        }
        KeyCode::F(2) => {
            state.screen = Screen::Chat;
            return Outcome::None;
        }
        KeyCode::F(3) => {
            state.screen = Screen::Contacts;
            return Outcome::None;
        }
        KeyCode::F(4) => {
            state.screen = Screen::Network;
            return Outcome::None;
        }
        KeyCode::Tab => {
            let idx = state.screen.index();
            state.screen = Screen::ALL[(idx + 1) % Screen::ALL.len()];
            return Outcome::None;
        }
        KeyCode::BackTab => {
            let idx = state.screen.index();
            state.screen = Screen::ALL[(idx + Screen::ALL.len() - 1) % Screen::ALL.len()];
            return Outcome::None;
        }
        KeyCode::Char('p') if ctrl => {
            state.palette = Some(PaletteState {
                query: String::new(),
                selected: 0,
            });
            return Outcome::None;
        }
        KeyCode::Char(':') if !typing => {
            state.palette = Some(PaletteState {
                query: String::new(),
                selected: 0,
            });
            return Outcome::None;
        }
        KeyCode::Char('?') if !typing => {
            state.modal = Some(Modal::Help);
            return Outcome::None;
        }
        _ => {}
    }

    if let KeyCode::Char(c) = key.code
        && !ctrl
        && !typing
        && let Some(index) = "1234567".find(c)
    {
        state.screen = Screen::ALL[index];
        return Outcome::None;
    }

    // Quit when not typing anywhere.
    if key.code == KeyCode::Esc && state.screen == Screen::Chat && state.chat_focus {
        state.chat_focus = false;
        return Outcome::None;
    }

    if key.code == KeyCode::Char('q') && !state.chat_focus {
        return Outcome::Quit;
    }

    match state.screen {
        Screen::Dashboard => dashboard_keys(state, key),
        Screen::Chat => chat_keys(state, key, ctrl),
        Screen::Contacts => contacts_keys(state, key),
        Screen::Network => network_keys(state, key),
        Screen::Registry => registry_keys(state, key),
        Screen::Messages => messages_keys(state, key),
        Screen::Settings => settings_keys(state, key),
    }
}

fn handle_palette(state: &mut AppState, key: KeyEvent) -> Outcome {
    let items = filtered_commands(state);
    let total = items.len();

    if state.palette.is_none() {
        return Outcome::None;
    }

    match key.code {
        KeyCode::Esc => {
            state.palette = None;
        }
        KeyCode::Up => {
            if total > 0
                && let Some(palette) = state.palette.as_mut()
            {
                palette.selected = palette.selected.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if total > 0
                && let Some(palette) = state.palette.as_mut()
            {
                palette.selected = (palette.selected + 1) % total;
            }
        }
        KeyCode::Enter => {
            let selected = state
                .palette
                .as_ref()
                .map(|palette| palette.selected)
                .unwrap_or(0);
            let command = items.get(selected).copied();
            state.palette = None;
            if let Some(command) = command {
                return resolve_command(state, command);
            }
        }
        KeyCode::Backspace => {
            if let Some(palette) = state.palette.as_mut() {
                palette.query.pop();
                palette.selected = 0;
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(palette) = state.palette.as_mut() {
                palette.query.push(c);
                palette.selected = 0;
            }
        }
        _ => {}
    }
    Outcome::None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModalKind {
    Help,
    Info,
    Secret,
    Form,
    Confirm,
    TaskLog,
}

fn modal_kind(state: &AppState) -> Option<ModalKind> {
    match state.modal.as_ref()? {
        Modal::Help => Some(ModalKind::Help),
        Modal::Info(_) => Some(ModalKind::Info),
        Modal::Secret(_) => Some(ModalKind::Secret),
        Modal::Form(_) => Some(ModalKind::Form),
        Modal::Confirm(_) => Some(ModalKind::Confirm),
        Modal::TaskLog => Some(ModalKind::TaskLog),
    }
}

fn handle_modal(state: &mut AppState, key: KeyEvent) -> Outcome {
    let Some(kind) = modal_kind(state) else {
        return Outcome::None;
    };

    match kind {
        ModalKind::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?')) {
                state.modal = None;
            }
            Outcome::None
        }
        ModalKind::TaskLog => {
            if key.code == KeyCode::Char('x') {
                return Outcome::CancelTasks;
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                state.modal = None;
            }
            Outcome::None
        }
        ModalKind::Info => {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?')
            ) {
                state.modal = None;
                return Outcome::None;
            }
            if let Some(Modal::Info(info)) = state.modal.as_mut() {
                match key.code {
                    KeyCode::Up => info.scroll = info.scroll.saturating_sub(1),
                    KeyCode::Down => info.scroll = info.scroll.saturating_add(1),
                    KeyCode::PageUp => info.scroll = info.scroll.saturating_sub(8),
                    KeyCode::PageDown => info.scroll = info.scroll.saturating_add(8),
                    _ => {}
                }
            }
            Outcome::None
        }
        ModalKind::Secret => {
            match key.code {
                KeyCode::Char('r') => {
                    if let Some(Modal::Secret(secret)) = state.modal.as_mut() {
                        secret.revealed = !secret.revealed;
                    }
                }
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    state.modal = None;
                }
                _ => {}
            }
            Outcome::None
        }
        ModalKind::Confirm => {
            if key.code == KeyCode::Esc {
                state.modal = None;
                return Outcome::None;
            }

            if key.code == KeyCode::Enter {
                let mut request = None;
                let mut error = None;
                if let Some(Modal::Confirm(confirm)) = state.modal.as_ref() {
                    let phrase_ok = confirm
                        .phrase
                        .as_ref()
                        .map(|phrase| phrase == &confirm.input)
                        .unwrap_or(true);
                    if phrase_ok {
                        request = Some(confirm.request.clone());
                    } else {
                        error = Some(format!(
                            "Type '{}' to confirm.",
                            confirm.phrase.clone().unwrap_or_default()
                        ));
                    }
                }
                if let Some(request) = request {
                    state.modal = None;
                    return Outcome::Run(request);
                }
                if let Some(error) = error {
                    state.toast(error, ToastKind::Error);
                    if let Some(Modal::Confirm(confirm)) = state.modal.as_mut() {
                        confirm.input.clear();
                    }
                }
                return Outcome::None;
            }

            if let Some(Modal::Confirm(confirm)) = state.modal.as_mut() {
                match key.code {
                    KeyCode::Backspace => {
                        confirm.input.pop();
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        confirm.input.push(c);
                    }
                    _ => {}
                }
            }
            Outcome::None
        }
        ModalKind::Form => {
            if key.code == KeyCode::Esc {
                state.modal = None;
                return Outcome::None;
            }

            if key.code == KeyCode::Enter {
                let submit = state
                    .modal
                    .as_ref()
                    .and_then(|modal| match modal {
                        Modal::Form(form) => Some(form.focus + 1 >= form.fields.len()),
                        _ => None,
                    })
                    .unwrap_or(false);

                if submit {
                    let result = if let Some(Modal::Form(form)) = state.modal.as_mut() {
                        form.submit()
                    } else {
                        Err("Form closed.".to_string())
                    };
                    match result {
                        Ok(request) => {
                            state.modal = None;
                            return Outcome::Run(request);
                        }
                        Err(error) => {
                            if let Some(Modal::Form(form)) = state.modal.as_mut() {
                                form.error = Some(error);
                            }
                        }
                    }
                } else if let Some(Modal::Form(form)) = state.modal.as_mut() {
                    form.focus += 1;
                }
                return Outcome::None;
            }

            if let Some(Modal::Form(form)) = state.modal.as_mut() {
                match key.code {
                    KeyCode::Tab | KeyCode::Down => {
                        form.focus = (form.focus + 1) % form.fields.len();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.focus = (form.focus + form.fields.len() - 1) % form.fields.len();
                    }
                    KeyCode::Backspace => {
                        if let Some(field) = form.fields.get_mut(form.focus) {
                            field.value.pop();
                        }
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(field) = form.fields.get_mut(form.focus) {
                            field.value.push(c);
                        }
                    }
                    _ => {}
                }
            }
            Outcome::None
        }
    }
}

fn dashboard_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Up => {
            state.checklist_index = state.checklist_index.saturating_sub(1);
            Outcome::None
        }
        KeyCode::Down => {
            if !state.checklist.is_empty() {
                state.checklist_index = (state.checklist_index + 1) % state.checklist.len();
            }
            Outcome::None
        }
        KeyCode::Enter => {
            if let Some(item) = state.checklist.get(state.checklist_index) {
                let command = item.command;
                resolve_command(state, command)
            } else {
                Outcome::None
            }
        }
        KeyCode::Char('i') => resolve_command(state, Command::Init),
        KeyCode::Char('r') => resolve_command(state, Command::Register),
        KeyCode::Char('o') => resolve_command(
            state,
            if state.online {
                Command::Offline
            } else {
                Command::Online
            },
        ),
        _ => Outcome::None,
    }
}

fn chat_keys(state: &mut AppState, key: KeyEvent, ctrl: bool) -> Outcome {
    if state.chat_focus {
        match key.code {
            KeyCode::Enter => {
                let text = state.composer.trim().to_string();
                let Some(peer) = state.current_chat_name() else {
                    state.toast("No conversation selected.", ToastKind::Error);
                    return Outcome::None;
                };
                if text.is_empty() {
                    return Outcome::None;
                }
                state.composer.clear();
                Outcome::Run(RunRequest::SendEncrypted { peer, text })
            }
            KeyCode::Char('q') if ctrl => {
                let text = state.composer.trim().to_string();
                let Some(peer) = state.current_chat_name() else {
                    state.toast("No conversation selected.", ToastKind::Error);
                    return Outcome::None;
                };
                if text.is_empty() {
                    return Outcome::None;
                }
                state.composer.clear();
                Outcome::Run(RunRequest::QueueMessage { peer, text })
            }
            KeyCode::Backspace => {
                state.composer.pop();
                Outcome::None
            }
            KeyCode::Char(c) if !ctrl => {
                state.composer.push(c);
                Outcome::None
            }
            KeyCode::PageUp => {
                state.chat_offset = state.chat_offset.saturating_add(3);
                Outcome::None
            }
            KeyCode::PageDown => {
                state.chat_offset = state.chat_offset.saturating_sub(3);
                Outcome::None
            }
            _ => Outcome::None,
        }
    } else {
        match key.code {
            KeyCode::Up => {
                if !state.chats.is_empty() {
                    state.chat_index = state.chat_index.saturating_sub(1);
                    state.reload_messages();
                }
                Outcome::None
            }
            KeyCode::Down => {
                if !state.chats.is_empty() {
                    state.chat_index = (state.chat_index + 1) % state.chats.len();
                    state.reload_messages();
                }
                Outcome::None
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                if state.current_chat().is_some() {
                    state.chat_focus = true;
                } else {
                    state.toast("Add a contact to start chatting.", ToastKind::Info);
                }
                Outcome::None
            }
            KeyCode::PageUp => {
                state.chat_offset = state.chat_offset.saturating_add(5);
                Outcome::None
            }
            KeyCode::PageDown => {
                state.chat_offset = state.chat_offset.saturating_sub(5);
                Outcome::None
            }
            KeyCode::Char('m') => {
                if let Some(peer) = state.current_chat_name() {
                    return Outcome::Run(RunRequest::MarkRead(peer));
                }
                Outcome::None
            }
            KeyCode::Char('x') => {
                if let Some(peer) = state.current_chat_name() {
                    state.open_confirm(
                        "CLEAR CONVERSATION",
                        format!("Delete the entire conversation with '{}'?", peer),
                        Some("YES"),
                        RunRequest::ClearChat(peer),
                    );
                }
                Outcome::None
            }
            KeyCode::Char('c') => {
                conversation_details(state);
                Outcome::None
            }
            KeyCode::Char('v') => {
                if let Some(peer) = state.current_chat_name() {
                    return Outcome::Run(RunRequest::SetTrust {
                        username: peer,
                        level: TrustLevel::Verified,
                    });
                }
                Outcome::None
            }
            _ => Outcome::None,
        }
    }
}

fn conversation_details(state: &mut AppState) {
    let Some(chat) = state.current_chat() else {
        return;
    };
    let mut lines = vec![
        ("Username".to_string(), chat.username.clone()),
        ("Trust level".to_string(), chat.trust.to_string()),
        ("Account status".to_string(), chat.account_status.clone()),
        ("Messages".to_string(), chat.total.to_string()),
        (
            "Unread".to_string(),
            state.unread_count(&chat.username).to_string(),
        ),
        (
            "Last activity".to_string(),
            chat.last_activity
                .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_else(|| "—".to_string()),
        ),
        (
            "Punch state".to_string(),
            state
                .punch_state_for(&chat.username)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "—".to_string()),
        ),
        (
            "Secure session".to_string(),
            if state.has_secure(&chat.username) {
                "ESTABLISHED".to_string()
            } else {
                "—".to_string()
            },
        ),
    ];
    if let Some(contact) = state
        .contacts
        .iter()
        .find(|c| c.username.eq_ignore_ascii_case(&chat.username))
    {
        lines.push(("Fingerprint".to_string(), contact.fingerprint.clone()));
    }
    state.open_info(
        "CONVERSATION DETAILS",
        lines
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect(),
    );
}

fn contacts_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Up => {
            state.contact_index = state.contact_index.saturating_sub(1);
            Outcome::None
        }
        KeyCode::Down => {
            if !state.contacts.is_empty() {
                state.contact_index = (state.contact_index + 1) % state.contacts.len();
            }
            Outcome::None
        }
        KeyCode::Enter => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::ShowContact(name));
            }
            Outcome::None
        }
        KeyCode::Char('a') => {
            state.open_form(FormKind::AddContact, None);
            Outcome::None
        }
        KeyCode::Char('d') => {
            if let Some(name) = state.current_contact_name() {
                state.open_confirm(
                    "REMOVE CONTACT",
                    format!("Remove '{}' from your local contacts?", name),
                    None,
                    RunRequest::RemoveContact(name),
                );
            }
            Outcome::None
        }
        KeyCode::Char('v') => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::SetTrust {
                    username: name,
                    level: TrustLevel::Verified,
                });
            }
            Outcome::None
        }
        KeyCode::Char('u') => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::SetTrust {
                    username: name,
                    level: TrustLevel::Unverified,
                });
            }
            Outcome::None
        }
        KeyCode::Char('b') => {
            if let Some(contact) = state.current_contact() {
                let name = contact.username.clone();
                let level = if contact.trust_level == TrustLevel::Blocked {
                    TrustLevel::Unverified
                } else {
                    TrustLevel::Blocked
                };
                return Outcome::Run(RunRequest::SetTrust {
                    username: name,
                    level,
                });
            }
            Outcome::None
        }
        KeyCode::Char('s') => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::Status(name));
            }
            Outcome::None
        }
        KeyCode::Char('m') => {
            if let Some(name) = state.current_contact_name() {
                state.open_chat(&name, true);
            }
            Outcome::None
        }
        KeyCode::Char('p') => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::Ping(name));
            }
            Outcome::None
        }
        KeyCode::Char('c') => {
            if let Some(name) = state.current_contact_name() {
                return Outcome::Run(RunRequest::Connect(name));
            }
            Outcome::None
        }
        _ => Outcome::None,
    }
}

fn network_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Up => {
            let len = state.network_targets().len();
            if len > 0 {
                state.peer_index = state.peer_index.saturating_sub(1);
            }
            Outcome::None
        }
        KeyCode::Down => {
            let len = state.network_targets().len();
            if len > 0 {
                state.peer_index = (state.peer_index + 1) % len;
            }
            Outcome::None
        }
        KeyCode::Enter => {
            network_target_details(state);
            Outcome::None
        }
        KeyCode::Char('n') => Outcome::Run(RunRequest::Netinfo),
        KeyCode::Char('C') => Outcome::Run(RunRequest::Capabilities),
        KeyCode::Char('p') => Outcome::Run(RunRequest::PublishCandidates),
        KeyCode::Char('r') => Outcome::Run(RunRequest::RefreshCandidates),
        KeyCode::Char('f') => resolve_command(state, Command::FetchCandidates),
        KeyCode::Char('d') => resolve_command(state, Command::Discover),
        KeyCode::Char('c') => resolve_command(state, Command::Connect),
        KeyCode::Char('x') => resolve_command(state, Command::Disconnect),
        KeyCode::Char('P') => resolve_command(state, Command::Ping),
        KeyCode::Char('k') => resolve_command(state, Command::CandidatePairs),
        KeyCode::Char('i') => resolve_command(state, Command::IceCheck),
        KeyCode::Char('g') => resolve_command(state, Command::Negotiate),
        KeyCode::Char('s') => resolve_command(state, Command::SecureSession),
        KeyCode::Char('u') => resolve_command(state, Command::Punch),
        KeyCode::Char('t') => resolve_command(state, Command::PunchStatus),
        KeyCode::Char('T') => Outcome::Run(RunRequest::TestUdp),
        _ => Outcome::None,
    }
}

fn network_target_details(state: &mut AppState) {
    let Some(name) = state.current_network_target() else {
        return;
    };

    if let Some(peer) = state.peer_for(&name).cloned() {
        let lines = vec![
            ("Username".to_string(), peer.username),
            ("State".to_string(), peer.state.to_string()),
            ("Transport".to_string(), peer.transport.to_string()),
            ("Session".to_string(), peer.session_id),
            ("Connected".to_string(), peer.connected_at),
            ("Last seen".to_string(), peer.last_seen),
        ];
        state.open_info(
            "PEER SESSION",
            lines
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect(),
        );
        return;
    }

    if let Some(contact) = state
        .contacts
        .iter()
        .find(|contact| contact.username.eq_ignore_ascii_case(&name))
        .cloned()
    {
        let lines = vec![
            ("Username".to_string(), contact.username),
            ("Fingerprint".to_string(), contact.fingerprint),
            ("Trust level".to_string(), contact.trust_level.to_string()),
            ("Account status".to_string(), contact.account_status),
            (
                "Added".to_string(),
                contact.added_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            ),
            (
                "Session".to_string(),
                "none yet · press c to connect, g to negotiate, i for ICE".to_string(),
            ),
        ];
        state.open_info(
            "CONTACT TARGET",
            lines
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect(),
        );
    }
}

fn registry_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Char('/') => {
            state.search_focus = true;
            Outcome::None
        }
        KeyCode::Up => {
            state.search_index = state.search_index.saturating_sub(1);
            Outcome::None
        }
        KeyCode::Down => {
            if !state.search_results.is_empty() {
                state.search_index = (state.search_index + 1) % state.search_results.len();
            }
            Outcome::None
        }
        KeyCode::Enter => {
            if let Some(result) = state.search_results.get(state.search_index) {
                return Outcome::Run(RunRequest::Lookup(result.username.clone()));
            }
            let query = state.search_query.trim().to_string();
            if !query.is_empty() {
                return Outcome::Run(RunRequest::Search(query));
            }
            Outcome::None
        }
        KeyCode::Char('a') => {
            if let Some(result) = state.search_results.get(state.search_index) {
                let name = result.username.clone();
                return Outcome::Run(RunRequest::AddContact(name));
            }
            state.open_form(FormKind::AddContact, None);
            Outcome::None
        }
        KeyCode::Char('r') => {
            state.open_form(FormKind::Register, None);
            Outcome::None
        }
        KeyCode::Char('n') => {
            state.open_form(FormKind::Rename, None);
            Outcome::None
        }
        KeyCode::Char('R') => {
            state.open_form(FormKind::Recover, None);
            Outcome::None
        }
        KeyCode::Char('s') => {
            state.open_form(FormKind::Restore, None);
            Outcome::None
        }
        KeyCode::Char('l') => {
            state.open_form(FormKind::Lock, None);
            Outcome::None
        }
        KeyCode::Char('U') => {
            state.open_form(FormKind::Unlock, None);
            Outcome::None
        }
        KeyCode::Char('D') => {
            state.open_form(FormKind::RemoveAccount, None);
            Outcome::None
        }
        KeyCode::Char('L') => {
            state.open_form(FormKind::Lookup, None);
            Outcome::None
        }
        KeyCode::Char('i') => resolve_command(state, Command::Init),
        _ => Outcome::None,
    }
}

fn messages_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Left => {
            state.messages_tab = match state.messages_tab {
                MessagesTab::Inbox => MessagesTab::Requests,
                MessagesTab::Unread => MessagesTab::Inbox,
                MessagesTab::Requests => MessagesTab::Unread,
            };
            state.messages_index = 0;
            Outcome::None
        }
        KeyCode::Right => {
            state.messages_tab = match state.messages_tab {
                MessagesTab::Inbox => MessagesTab::Unread,
                MessagesTab::Unread => MessagesTab::Requests,
                MessagesTab::Requests => MessagesTab::Inbox,
            };
            state.messages_index = 0;
            Outcome::None
        }
        KeyCode::Up => {
            state.messages_index = state.messages_index.saturating_sub(1);
            Outcome::None
        }
        KeyCode::Down => {
            let len = state.message_row_count();
            if len > 0 {
                state.messages_index = (state.messages_index + 1) % len;
            }
            Outcome::None
        }
        KeyCode::Enter => {
            match state.messages_tab {
                MessagesTab::Inbox => {
                    if let Some(chat) = state.chats.get(state.messages_index) {
                        let peer = chat.username.clone();
                        state.open_chat(&peer, false);
                    }
                }
                MessagesTab::Unread => {
                    if let Some((peer, _)) = state.unread.get(state.messages_index) {
                        let peer = peer.clone();
                        state.open_chat(&peer, false);
                    }
                }
                MessagesTab::Requests => {
                    if let Some(request) = state.requests.get(state.messages_index) {
                        if request.status == crate::protocol::requests::RequestStatus::Pending {
                            let name = request.username.clone();
                            return Outcome::Run(RunRequest::AcceptRequest(name));
                        }
                        state.toast("Request is not pending.", ToastKind::Info);
                    }
                }
            }
            Outcome::None
        }
        KeyCode::Char('a') => {
            if state.messages_tab == MessagesTab::Requests
                && let Some(request) = state.requests.get(state.messages_index)
            {
                let name = request.username.clone();
                return Outcome::Run(RunRequest::AcceptRequest(name));
            }
            Outcome::None
        }
        KeyCode::Char('x') => {
            if state.messages_tab == MessagesTab::Requests
                && let Some(request) = state.requests.get(state.messages_index)
            {
                let name = request.username.clone();
                return Outcome::Run(RunRequest::RejectRequest(name));
            }
            Outcome::None
        }
        KeyCode::Char('m') => {
            let peer = match state.messages_tab {
                MessagesTab::Inbox => state
                    .chats
                    .get(state.messages_index)
                    .map(|c| c.username.clone()),
                MessagesTab::Unread => state
                    .unread
                    .get(state.messages_index)
                    .map(|(p, _)| p.clone()),
                MessagesTab::Requests => None,
            };
            if let Some(peer) = peer {
                return Outcome::Run(RunRequest::MarkRead(peer));
            }
            Outcome::None
        }
        _ => Outcome::None,
    }
}

fn settings_keys(state: &mut AppState, key: KeyEvent) -> Outcome {
    match key.code {
        KeyCode::Char('i') => resolve_command(state, Command::Init),
        KeyCode::Char('o') => resolve_command(
            state,
            if state.online {
                Command::Offline
            } else {
                Command::Online
            },
        ),
        KeyCode::Char('l') => Outcome::ToggleListener,
        KeyCode::Char('d') => {
            state.open_form(FormKind::Inject, None);
            Outcome::None
        }
        KeyCode::Char('c') => Outcome::Run(RunRequest::Capabilities),
        KeyCode::Char('n') => Outcome::Run(RunRequest::Netinfo),
        KeyCode::Char('p') => Outcome::Run(RunRequest::PublishCandidates),
        KeyCode::Char('r') => Outcome::Refresh,
        _ => Outcome::None,
    }
}

pub fn filtered_commands(state: &AppState) -> Vec<Command> {
    let query = state
        .palette
        .as_ref()
        .map(|p| p.query.to_lowercase())
        .unwrap_or_default();
    let mut matches: Vec<(u8, Command)> = palette_commands()
        .into_iter()
        .filter_map(|command| palette_score(command, &query).map(|score| (score, command)))
        .collect();
    matches.sort_by_key(|(score, _)| *score);
    matches.into_iter().map(|(_, command)| command).collect()
}

/// Lower scores rank first. Display labels win over aliases, which win over
/// loose subsequence matches, so typing a CLI verb finds the obvious action.
fn palette_score(command: Command, query: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    let label = command.label().to_lowercase();
    let keywords = command.keywords();
    let category = command.category().to_lowercase();

    if label == query {
        return Some(0);
    }
    if label.starts_with(query)
        || label.split_whitespace().any(|word| word == query)
        || keywords.split_whitespace().any(|word| word == query)
    {
        return Some(1);
    }
    if label.contains(query) {
        return Some(2);
    }
    if keywords.contains(query) {
        return Some(3);
    }
    if category.contains(query) {
        return Some(4);
    }
    if fuzzy_match(query, &label) || fuzzy_match(query, keywords) {
        return Some(5);
    }
    None
}

fn fuzzy_match(query: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    query.chars().all(|q| chars.any(|h| h == q))
}

/// Turn a palette/checklist command into an executable request, opening a
/// form when a target or secret is missing.
pub fn resolve_command(state: &mut AppState, command: Command) -> Outcome {
    match command {
        Command::Init => {
            if state.has_identity() {
                state.open_confirm(
                    "OVERWRITE IDENTITY",
                    "An identity already exists. Overwrite the keypair and profile?",
                    None,
                    RunRequest::Init { force: true },
                );
                Outcome::None
            } else {
                Outcome::Run(RunRequest::Init { force: false })
            }
        }
        Command::Whoami => {
            let lines = vec![
                ("Username".to_string(), state.username()),
                ("Fingerprint".to_string(), state.fingerprint()),
                ("Registry".to_string(), state.registry_url()),
                (
                    "Online".to_string(),
                    if state.online { "YES" } else { "NO" }.to_string(),
                ),
                ("Storage".to_string(), state.storage_dir.clone()),
            ];
            state.open_info(
                "LOCAL IDENTITY",
                lines
                    .into_iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect(),
            );
            Outcome::None
        }
        Command::Search => {
            state.screen = Screen::Registry;
            state.search_focus = true;
            Outcome::None
        }
        Command::AddContact => {
            state.open_form(FormKind::AddContact, None);
            Outcome::None
        }
        Command::Register
        | Command::Lookup
        | Command::Recover
        | Command::Rename
        | Command::RemoveAccount
        | Command::Restore
        | Command::Lock
        | Command::Unlock
        | Command::Status
        | Command::DevInject => {
            if let Some(kind) = command.form_kind() {
                state.open_form(kind, None);
            }
            Outcome::None
        }
        Command::Listen => Outcome::ToggleListener,
        Command::Online => Outcome::Run(RunRequest::Online),
        Command::Offline => Outcome::Run(RunRequest::Offline),
        Command::Netinfo => Outcome::Run(RunRequest::Netinfo),
        Command::Capabilities => Outcome::Run(RunRequest::Capabilities),
        Command::TestUdp => Outcome::Run(RunRequest::TestUdp),
        Command::PublishCandidates => Outcome::Run(RunRequest::PublishCandidates),
        Command::RefreshCandidates => Outcome::Run(RunRequest::RefreshCandidates),
        Command::Inbox => {
            state.screen = Screen::Messages;
            state.messages_tab = MessagesTab::Inbox;
            Outcome::None
        }
        Command::Unread => {
            state.screen = Screen::Messages;
            state.messages_tab = MessagesTab::Unread;
            Outcome::None
        }
        Command::Requests => {
            state.screen = Screen::Messages;
            state.messages_tab = MessagesTab::Requests;
            Outcome::None
        }
        Command::Peers => {
            state.screen = Screen::Network;
            Outcome::None
        }
        Command::SendEncrypted | Command::QueueMessage => {
            if let Some(peer) = state.action_target() {
                state.open_chat(&peer, true);
            } else {
                state.toast("Select a contact first.", ToastKind::Info);
            }
            Outcome::None
        }
        Command::AcceptRequest | Command::RejectRequest => {
            let on_requests =
                state.screen == Screen::Messages && state.messages_tab == MessagesTab::Requests;
            if on_requests && let Some(request) = state.current_request() {
                let name = request.username.clone();
                if command == Command::AcceptRequest {
                    Outcome::Run(RunRequest::AcceptRequest(name))
                } else {
                    Outcome::Run(RunRequest::RejectRequest(name))
                }
            } else {
                state.screen = Screen::Messages;
                state.messages_tab = MessagesTab::Requests;
                state.messages_index = 0;
                state.toast("Select a request first.", ToastKind::Info);
                Outcome::None
            }
        }
        Command::ConversationShow => {
            if state.current_chat().is_some() {
                conversation_details(state);
            } else {
                state.toast("Select a conversation first.", ToastKind::Info);
            }
            Outcome::None
        }
        Command::MessageHistory => {
            if let Some(peer) = state.action_target() {
                state.open_chat(&peer, false);
            } else {
                state.toast("Select a conversation first.", ToastKind::Info);
            }
            Outcome::None
        }
        Command::MessageClear => {
            if let Some(peer) = state.action_target() {
                state.open_confirm(
                    "CLEAR CONVERSATION",
                    format!("Delete the entire conversation with '{}'?", peer),
                    Some("YES"),
                    RunRequest::ClearChat(peer),
                );
            } else {
                state.toast("Select a conversation first.", ToastKind::Info);
            }
            Outcome::None
        }
        Command::ShowContact => {
            if let Some(name) = state.contact_target() {
                Outcome::Run(RunRequest::ShowContact(name))
            } else {
                state.screen = Screen::Contacts;
                state.toast("Select a contact first.", ToastKind::Info);
                Outcome::None
            }
        }
        Command::RemoveContact => {
            if let Some(name) = state.contact_target() {
                state.open_confirm(
                    "REMOVE CONTACT",
                    format!("Remove '{}' from your local contacts?", name),
                    None,
                    RunRequest::RemoveContact(name),
                );
            } else {
                state.screen = Screen::Contacts;
                state.toast("Select a contact first.", ToastKind::Info);
            }
            Outcome::None
        }
        Command::Verify => target_trust(state, TrustLevel::Verified),
        Command::Unverify => target_trust(state, TrustLevel::Unverified),
        Command::Block => target_trust(state, TrustLevel::Blocked),
        Command::Unblock => target_trust(state, TrustLevel::Unverified),
        _ => {
            // Commands that need a target peer.
            if command.needs_target() {
                if let Some(peer) = state.action_target() {
                    return run_peer_command(command, peer);
                }
                if let Some(kind) = command.form_kind() {
                    state.open_form(kind, None);
                }
                return Outcome::None;
            }
            if let Some(kind) = command.form_kind() {
                state.open_form(kind, None);
            }
            Outcome::None
        }
    }
}

fn target_trust(state: &mut AppState, level: TrustLevel) -> Outcome {
    if let Some(username) = state.contact_target() {
        Outcome::Run(RunRequest::SetTrust { username, level })
    } else {
        state.toast("Select a contact first.", ToastKind::Info);
        Outcome::None
    }
}

fn run_peer_command(command: Command, peer: String) -> Outcome {
    let request = match command {
        Command::Discover => RunRequest::Discover(peer),
        Command::Connect => RunRequest::Connect(peer),
        Command::Disconnect => RunRequest::Disconnect(peer),
        Command::Ping => RunRequest::Ping(peer),
        Command::Negotiate => RunRequest::Negotiate(peer),
        Command::SecureSession => RunRequest::SecureSession(peer),
        Command::CandidatePairs => RunRequest::CandidatePairs(peer),
        Command::IceCheck => RunRequest::IceCheck(peer),
        Command::Punch => RunRequest::Punch(peer),
        Command::PunchStatus => RunRequest::PunchStatus(peer),
        Command::FetchCandidates => RunRequest::FetchCandidates(peer),
        Command::Status => RunRequest::Status(peer),
        Command::ShowContact => RunRequest::ShowContact(peer),
        Command::SendEncrypted => RunRequest::SendEncrypted {
            peer,
            text: String::new(),
        },
        _ => return Outcome::None,
    };
    Outcome::Run(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_cli_verbs() {
        assert!(palette_score(Command::Whoami, "whoami").is_some());
        assert!(palette_score(Command::Punch, "punch").is_some());
        assert!(palette_score(Command::Listen, "listen").is_some());
        assert!(palette_score(Command::Init, "init").is_some());
        assert!(palette_score(Command::RemoveAccount, "delete account").is_some());
    }

    #[test]
    fn palette_ranks_direct_matches_above_fuzzy_ones() {
        let send = palette_score(Command::SendEncrypted, "send").unwrap();
        let search = palette_score(Command::Search, "send").unwrap();
        assert!(send < search);

        let punch = palette_score(Command::Punch, "punch").unwrap();
        let status = palette_score(Command::PunchStatus, "punch").unwrap();
        assert_eq!(punch, status);
    }

    #[test]
    fn every_palette_command_has_search_keywords() {
        for command in palette_commands() {
            assert!(
                !command.keywords().is_empty(),
                "{} has no palette keywords",
                command.label()
            );
        }
    }
}
