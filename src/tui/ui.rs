//! Full-screen rendering for the interactive TUI.

use crate::chat::models::{Direction, MessageStatus};
use crate::contacts::models::TrustLevel;
use crate::tui::events;
use crate::tui::state::{
    AppState, BG, BORDER, BURNT, CREAM, ChatRow, DARK_RED, DIM, GREEN, LINEN, MessagesTab, Modal,
    PANEL_ALT, PANEL_BG, RED_ACCENT, Screen, ToastKind, WHEAT,
};
use crate::tui::widgets as w;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};

const SELECTED_BG: Color = Color::Rgb(52, 40, 34);
const POSITIVE_BG: Color = Color::Rgb(30, 62, 42);
const WARNING_BG: Color = Color::Rgb(62, 52, 30);
const NEGATIVE_BG: Color = Color::Rgb(64, 28, 26);

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::default().bg(BG).fg(CREAM)), area);

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, state, outer[0]);
    render_tabs(frame, state, outer[1]);

    match state.screen {
        Screen::Dashboard => render_dashboard(frame, state, outer[2]),
        Screen::Chat => render_chat(frame, state, outer[2]),
        Screen::Contacts => render_contacts(frame, state, outer[2]),
        Screen::Network => render_network(frame, state, outer[2]),
        Screen::Registry => render_registry(frame, state, outer[2]),
        Screen::Messages => render_messages(frame, state, outer[2]),
        Screen::Settings => render_settings(frame, state, outer[2]),
    }

    render_footer(frame, state, outer[3]);
    render_toasts(frame, state, outer[2]);
    render_overlays(frame, state, area);
}

// ── Header ─────────────────────────────────────────────────────────────────
fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Min(20), Constraint::Length(38)]).split(inner);

    let mut left = vec![
        Span::styled(" ◈ ", Style::default().fg(BURNT).bold()),
        Span::styled(
            "RUST-MESSENGER",
            Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().fg(DIM)),
        w::online_badge(state.online),
        Span::raw(" "),
        w::listener_badge(state.listener_running),
    ];
    if state.has_identity() {
        left.push(Span::raw("  "));
        left.push(Span::styled(
            state.username(),
            Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
        ));
        left.push(Span::styled(
            format!("  {}", state.fingerprint()),
            Style::default().fg(DIM),
        ));
    } else {
        left.push(Span::raw("  "));
        left.push(Span::styled(
            "no identity yet",
            Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
        ));
    }

    let mut right = Vec::new();
    if state.running_tasks > 0 {
        right.push(Span::styled(
            format!(
                "{} {} task(s) ",
                w::spinner(state.tick),
                state.running_tasks
            ),
            Style::default().fg(WHEAT),
        ));
        right.push(Span::styled("F8", Style::default().fg(BURNT).bold()));
        right.push(Span::raw("  "));
    }
    right.push(Span::styled(
        w::truncate(&state.registry_url(), 34),
        Style::default().fg(DIM),
    ));

    frame.render_widget(Paragraph::new(Line::from(left)), cols[0]);
    frame.render_widget(
        Paragraph::new(Line::from(right)).alignment(Alignment::Right),
        cols[1],
    );
}

// ── Tabs ───────────────────────────────────────────────────────────────────
fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut spans = vec![Span::raw(" ")];
    for screen in Screen::ALL {
        let active = screen == state.screen;
        let style = if active {
            Style::default()
                .fg(Color::Rgb(20, 20, 22))
                .bg(BURNT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        spans.push(Span::styled(
            format!(" {} {} ", screen.index() + 1, screen.title()),
            style,
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw("            "));
    spans.push(Span::styled(
        "Ctrl+P / :  command palette",
        Style::default().fg(DIM),
    ));

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
    frame.render_widget(Paragraph::new(Line::from(spans)), rows[0]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(BORDER),
        ))),
        rows[1],
    );
}

// ── Footer ─────────────────────────────────────────────────────────────────
fn render_footer(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let hints: Vec<(&str, &str)> = match state.screen {
        Screen::Dashboard => vec![
            ("↑↓", "select"),
            ("Enter", "run"),
            ("i", "init"),
            ("r", "register"),
            ("o", "online"),
        ],
        Screen::Chat if state.chat_focus => vec![
            ("Enter", "send encrypted"),
            ("Ctrl+Q", "queue"),
            ("Esc", "stop typing"),
            ("PgUp/PgDn", "scroll"),
        ],
        Screen::Chat => vec![
            ("↑↓", "conversation"),
            ("Enter/i", "write"),
            ("m", "mark read"),
            ("x", "clear"),
            ("c", "details"),
        ],
        Screen::Contacts => vec![
            ("a", "add"),
            ("m", "chat"),
            ("v", "verify"),
            ("b", "block"),
            ("d", "remove"),
        ],
        Screen::Network => vec![
            ("↑↓", "target"),
            ("n", "netinfo"),
            ("p", "publish"),
            ("i", "ICE"),
            ("u", "punch"),
            ("s", "secure"),
        ],
        Screen::Registry => vec![
            ("/", "search"),
            ("Enter", "lookup"),
            ("a", "add"),
            ("r", "register"),
            ("R", "recover"),
        ],
        Screen::Messages => vec![
            ("←→", "tab"),
            ("↑↓", "select"),
            ("Enter", "open"),
            ("m", "mark read"),
            ("a/x", "request"),
        ],
        Screen::Settings => vec![
            ("o", "online"),
            ("l", "listener"),
            ("d", "inject"),
            ("c", "capabilities"),
            ("r", "refresh"),
        ],
    };

    for (key, description) in hints {
        spans.extend(w::key_hint(key, description));
    }
    spans.push(Span::styled("? help", Style::default().fg(DIM)));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Dashboard ──────────────────────────────────────────────────────────────
fn render_dashboard(frame: &mut Frame, state: &AppState, area: Rect) {
    let rows = Layout::vertical([Constraint::Percentage(44), Constraint::Min(6)]).split(area);
    let cols =
        Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)]).split(rows[0]);

    render_identity_card(frame, state, cols[0]);
    render_runtime_card(frame, state, cols[1]);
    render_checklist(frame, state, rows[1]);
}

fn render_identity_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("IDENTITY", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                if state.is_registered() {
                    state.username()
                } else {
                    "Not registered".to_string()
                },
                Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            w::online_badge(state.online),
        ]),
        Line::from(""),
        w::kv_line("Fingerprint", state.fingerprint(), 12),
        w::kv_line("Registry", w::truncate(&state.registry_url(), 40), 12),
        w::kv_line(
            "Session",
            w::truncate(state.session_id.as_deref().unwrap_or("—"), 32),
            12,
        ),
        w::kv_line("Storage", w::truncate(&state.storage_dir, 40), 12),
        Line::from(""),
    ];

    let done = state.checklist.iter().filter(|i| i.done).count();
    lines.push(Line::from(vec![
        Span::styled("Setup ", Style::default().fg(DIM)),
        Span::styled(
            format!("{}/{}", done, state.checklist.len()),
            Style::default().fg(BURNT).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            if state.is_registered() {
                "Identity ready. Press ? for every shortcut."
            } else {
                "Start with i (init), then r (register)."
            },
            Style::default().fg(DIM),
        ),
    ]));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_runtime_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("RUNTIME", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let publication = match state.publication_seconds_left() {
        Some(seconds) if seconds > 0 => {
            Span::styled(format!("{}s left", seconds), Style::default().fg(GREEN))
        }
        Some(_) => Span::styled("expired", Style::default().fg(RED_ACCENT)),
        None => Span::styled("not published", Style::default().fg(DIM)),
    };

    let lines = vec![
        w::kv_line(
            "Heartbeat",
            if state.heartbeat_running {
                "RUNNING"
            } else {
                "stopped"
            },
            12,
        )
        .style(if state.heartbeat_running {
            Style::default().fg(GREEN)
        } else {
            Style::default().fg(DIM)
        }),
        w::kv_line(
            "Listener",
            if state.listener_running {
                "RUNNING"
            } else {
                "stopped"
            },
            12,
        )
        .style(if state.listener_running {
            Style::default().fg(GREEN)
        } else {
            Style::default().fg(DIM)
        }),
        Line::from(vec![
            Span::styled(
                format!("{} ", w::pad("Tasks", 12)),
                Style::default().fg(DIM),
            ),
            Span::styled(
                format!("{} running", state.running_tasks),
                Style::default().fg(CREAM),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", w::pad("Candidates", 12)),
                Style::default().fg(DIM),
            ),
            publication,
        ]),
        Line::from(""),
        w::kv_line("Contacts", state.contacts.len().to_string(), 12),
        w::kv_line("Chats", state.chats.len().to_string(), 12),
        w::kv_line("Peers", state.peers.len().to_string(), 12),
        w::kv_line("Requests", state.requests.len().to_string(), 12),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_checklist(frame: &mut Frame, state: &AppState, area: Rect) {
    let done = state.checklist.iter().filter(|i| i.done).count();
    let total = state.checklist.len().max(1);
    let block = w::panel(
        &format!("GETTING STARTED · {}/{}", done, state.checklist.len()),
        false,
    );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);

    let bar_width = (rows[0].width as usize).saturating_sub(12).clamp(8, 48);
    let mut bar = w::progress_bar(done, total, bar_width);
    bar.spans.push(Span::raw("  "));
    bar.spans.push(Span::styled(
        format!("{}%", (done * 100) / total),
        Style::default().fg(WHEAT).bold(),
    ));
    frame.render_widget(Paragraph::new(bar), rows[0]);

    let mut lines = Vec::new();
    let width = rows[1].width as usize;
    for (index, item) in state.checklist.iter().enumerate() {
        let selected = index == state.checklist_index;
        let checkbox = if item.done {
            Span::styled("[✓] ", Style::default().fg(GREEN).bold())
        } else {
            Span::styled("[ ] ", Style::default().fg(DIM))
        };
        let label_width = width.saturating_sub(item.detail.chars().count() + 14);
        let mut spans = vec![
            Span::styled(
                if selected { "▸ " } else { "  " },
                Style::default().fg(BURNT),
            ),
            checkbox,
            Span::styled(
                w::pad(item.label, label_width.max(12)),
                Style::default()
                    .fg(if item.done { DIM } else { CREAM })
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Span::raw("  "),
            Span::styled(item.detail, Style::default().fg(DIM)),
        ];
        if item.done {
            spans.push(Span::styled("  ✓", Style::default().fg(GREEN)));
        }
        let style = if selected {
            Style::default().bg(SELECTED_BG)
        } else {
            Style::default()
        };
        lines.push(Line::from(spans).style(style));
    }

    frame.render_widget(Paragraph::new(lines), rows[1]);
}

// ── Chat ───────────────────────────────────────────────────────────────────
fn render_chat(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(24)]).split(area);
    render_chat_list(frame, state, cols[0]);
    render_thread(frame, state, cols[1]);
}

fn render_chat_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let focused = !state.chat_focus;
    let block = w::panel(&format!("CONVERSATIONS · {}", state.chats.len()), focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.chats.is_empty() {
        frame.render_widget(
            w::empty_state("No conversations", "Add a contact, then press m to chat."),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, chat) in state.chats.iter().enumerate() {
        let selected = index == state.chat_index;
        lines.push(chat_row(chat, selected, width));
        lines.push(Line::from(Span::styled(
            format!(
                "   {} · {} msg{}",
                chat.trust,
                chat.total,
                if chat.total == 1 { "" } else { "s" }
            ),
            Style::default().fg(DIM),
        )));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn chat_row(chat: &ChatRow, selected: bool, width: usize) -> Line<'static> {
    let marker = Span::styled(
        if selected { "▸ " } else { "  " },
        Style::default().fg(BURNT),
    );
    let dot = Span::styled("● ", Style::default().fg(w::trust_color(chat.trust)));
    let name_width = width.saturating_sub(18).max(8);
    let mut spans = vec![
        marker,
        dot,
        Span::styled(
            w::pad(&chat.username, name_width),
            Style::default()
                .fg(if selected { CREAM } else { LINEN })
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ];
    if chat.unread > 0 {
        spans.push(w::badge(
            chat.unread.to_string(),
            Color::Rgb(22, 20, 18),
            BURNT,
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(
        chat.last_activity
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "—".to_string()),
        Style::default().fg(DIM),
    ));
    let style = if selected {
        Style::default().bg(SELECTED_BG)
    } else {
        Style::default()
    };
    Line::from(spans).style(style)
}

fn render_thread(frame: &mut Frame, state: &AppState, area: Rect) {
    let peer = state.current_chat_name();
    let title = match &peer {
        Some(peer) => format!("CHAT · {}", peer),
        None => "CHAT".to_string(),
    };
    let block = w::panel(&title, state.chat_focus);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([Constraint::Min(2), Constraint::Length(3)]).split(inner);

    if peer.is_none() {
        frame.render_widget(
            w::empty_state(
                "No conversation selected",
                "Choose a conversation on the left, or add a contact from the Contacts screen.",
            ),
            rows[0],
        );
    } else if state.messages.is_empty() {
        frame.render_widget(
            w::empty_state(
                "No messages yet",
                "Press i to write. Enter sends end-to-end encrypted; Ctrl+Q queues locally.",
            ),
            rows[0],
        );
    } else {
        render_message_list(frame, state, rows[0]);
    }

    render_composer(frame, state, rows[1]);
}

fn render_message_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let capacity = (area.height as usize / 3).max(1);
    let len = state.messages.len();
    let end = len.saturating_sub(state.chat_offset.min(len));
    let start = end.saturating_sub(capacity);
    let visible = &state.messages[start..end];
    let width = area.width as usize;
    let mut lines = Vec::new();

    if start > 0 {
        lines.push(Line::from(Span::styled(
            format!("  ↑ {} older message(s)", start),
            Style::default().fg(DIM),
        )));
    }

    for message in visible {
        let incoming = message.direction == Direction::Incoming;
        let who = if incoming {
            state
                .current_chat_name()
                .unwrap_or_else(|| "peer".to_string())
        } else {
            "you".to_string()
        };
        let accent = if incoming { GREEN } else { BURNT };
        let mut header = vec![
            Span::styled(
                if incoming { "  ← " } else { "  → " },
                Style::default().fg(accent).bold(),
            ),
            Span::styled(
                who,
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", message.timestamp.format("%m-%d %H:%M")),
                Style::default().fg(DIM),
            ),
            Span::raw("  "),
        ];
        if !incoming {
            header.push(status_badge(message.status));
        }
        lines.push(Line::from(header));

        let content = message.content.replace('\n', " ⏎ ");
        lines.push(Line::from(Span::styled(
            format!("    {}", w::truncate(&content, width.saturating_sub(6))),
            Style::default().fg(if incoming { LINEN } else { CREAM }),
        )));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_composer(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if state.chat_focus { BURNT } else { BORDER }))
        .title(Line::from(Span::styled(
            " MESSAGE ",
            Style::default()
                .fg(if state.chat_focus { BURNT } else { DIM })
                .add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(PANEL_BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let line = if state.chat_focus {
        Line::from(vec![
            Span::styled("› ", Style::default().fg(BURNT).bold()),
            Span::styled(state.composer.clone(), Style::default().fg(CREAM)),
            Span::styled("▏", Style::default().fg(BURNT)),
        ])
    } else if state.composer.is_empty() {
        Line::from(vec![
            Span::styled("› ", Style::default().fg(DIM)),
            Span::styled(
                "press i to type · Enter sends · Ctrl+Q queues locally",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("› ", Style::default().fg(DIM)),
            Span::styled(state.composer.clone(), Style::default().fg(DIM)),
            Span::styled("  (draft)", Style::default().fg(DIM).italic()),
        ])
    };

    frame.render_widget(Paragraph::new(line), inner);
}

fn status_badge(status: MessageStatus) -> Span<'static> {
    match status {
        MessageStatus::Delivered | MessageStatus::Read => {
            w::badge(status_label(status), Color::Rgb(196, 240, 206), POSITIVE_BG)
        }
        MessageStatus::Sent => w::badge(status_label(status), WHEAT, WARNING_BG),
        MessageStatus::Queued => w::badge(
            status_label(status),
            Color::Rgb(210, 200, 190),
            Color::Rgb(48, 44, 42),
        ),
        MessageStatus::Failed => {
            w::badge(status_label(status), Color::Rgb(255, 190, 180), NEGATIVE_BG)
        }
    }
}

fn status_label(status: MessageStatus) -> &'static str {
    match status {
        MessageStatus::Queued => "QUEUED",
        MessageStatus::Sent => "SENT",
        MessageStatus::Delivered => "DELIVERED",
        MessageStatus::Read => "READ",
        MessageStatus::Failed => "FAILED",
    }
}

// ── Contacts ───────────────────────────────────────────────────────────────
fn render_contacts(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([Constraint::Length(36), Constraint::Min(24)]).split(area);
    render_contact_list(frame, state, cols[0]);
    render_contact_detail(frame, state, cols[1]);
}

fn render_contact_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel(&format!("CONTACTS · {}", state.contacts.len()), true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.contacts.is_empty() {
        frame.render_widget(
            w::empty_state(
                "No contacts",
                "Press a to fetch a public key from the registry.",
            ),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, contact) in state.contacts.iter().enumerate() {
        let selected = index == state.contact_index;
        let marker = Span::styled(
            if selected { "▸ " } else { "  " },
            Style::default().fg(BURNT),
        );
        let name_width = width.saturating_sub(18).max(8);
        lines.push(
            Line::from(vec![
                marker,
                Span::styled(
                    w::pad(&contact.username, name_width),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                w::trust_badge(contact.trust_level),
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );
        lines.push(Line::from(Span::styled(
            format!("    {}", contact.account_status),
            Style::default().fg(DIM),
        )));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_contact_detail(frame: &mut Frame, state: &AppState, area: Rect) {
    let contact = state.current_contact();
    let title = match contact {
        Some(contact) => format!("CONTACT · {}", contact.username),
        None => "CONTACT DETAILS".to_string(),
    };
    let block = w::panel(&title, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(contact) = contact else {
        frame.render_widget(
            w::empty_state(
                "Select a contact",
                "Add one with a. Press ? for the full keyboard reference.",
            ),
            inner,
        );
        return;
    };

    let mut lines = vec![
        w::section("IDENTITY"),
        w::kv_line("Username", contact.username.clone(), 13),
        w::kv_line("Fingerprint", contact.fingerprint.clone(), 13),
        Line::from(vec![
            Span::styled(
                format!("{} ", w::pad("Trust", 13)),
                Style::default().fg(DIM),
            ),
            w::trust_badge(contact.trust_level),
        ]),
        w::kv_line("Status", contact.account_status.clone(), 13),
        w::kv_line(
            "Added",
            contact.added_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            13,
        ),
        Line::from(""),
        w::section("PUBLIC KEY"),
    ];
    for chunk in split_chunks(&contact.public_key, 60) {
        lines.push(Line::from(Span::styled(
            format!("  {}", chunk),
            Style::default().fg(DIM),
        )));
    }
    lines.push(Line::from(""));
    lines.push(w::section("LINK"));
    lines.push(w::kv_line(
        "Punch",
        state
            .punch_state_for(&contact.username)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string()),
        13,
    ));
    lines.push(w::kv_line(
        "ICE",
        if state.has_ice(&contact.username) {
            "SELECTED"
        } else {
            "—"
        },
        13,
    ));
    lines.push(w::kv_line(
        "Secure",
        if state.has_secure(&contact.username) {
            "ESTABLISHED"
        } else {
            "—"
        },
        13,
    ));
    if !contact.notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(w::section("NOTES"));
        lines.push(Line::from(Span::styled(
            format!("  {}", contact.notes),
            Style::default().fg(LINEN),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "m chat · s presence · p ping · c connect · v verify · b block · d remove",
        Style::default().fg(DIM),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

// ── Network ────────────────────────────────────────────────────────────────
fn render_network(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(24)]).split(area);
    render_peer_list(frame, state, cols[0]);

    let rows = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(9),
        Constraint::Length(8),
    ])
    .split(cols[1]);
    render_link_state(frame, state, rows[0]);
    render_publication(frame, state, rows[1]);
    render_capabilities(frame, state, rows[2]);
}

fn render_peer_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let targets = state.network_targets();
    let block = w::panel(&format!("TARGETS · {}", targets.len()), true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if targets.is_empty() {
        frame.render_widget(
            w::empty_state(
                "No targets yet",
                "Add a contact from the Contacts screen (a), then run ICE checks here.",
            ),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, name) in targets.iter().enumerate() {
        let selected = index == state.peer_index;
        let peer = state.peer_for(name);
        let contact = state
            .contacts
            .iter()
            .find(|contact| contact.username.eq_ignore_ascii_case(name));
        let name_width = width.saturating_sub(16).max(8);

        let badge = match (peer, contact) {
            (Some(peer), _) => Span::styled(
                peer.state.to_string(),
                Style::default()
                    .fg(if peer.state.to_string() == "CONNECTED" {
                        GREEN
                    } else {
                        WHEAT
                    })
                    .bold(),
            ),
            (None, Some(contact)) => w::trust_badge(contact.trust_level),
            (None, None) => Span::styled("—", Style::default().fg(DIM)),
        };

        lines.push(
            Line::from(vec![
                Span::styled(
                    if selected { "▸ " } else { "  " },
                    Style::default().fg(BURNT),
                ),
                Span::styled(
                    w::pad(name, name_width),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                badge,
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );

        let detail = match (peer, contact) {
            (Some(peer), _) => format!(
                "    {} · {}",
                peer.transport,
                w::truncate(&peer.session_id, 16)
            ),
            (None, Some(contact)) => {
                format!("    contact · {}", w::truncate(&contact.fingerprint, 18))
            }
            (None, None) => "    no session".to_string(),
        };
        lines.push(Line::from(Span::styled(detail, Style::default().fg(DIM))));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_link_state(frame: &mut Frame, state: &AppState, area: Rect) {
    let target = state.current_network_target();
    let title = match &target {
        Some(peer) => format!("LINK · {}", peer),
        None => "LINK STATE".to_string(),
    };
    let block = w::panel(&title, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(peer) = target else {
        frame.render_widget(
            w::empty_state(
                "No target selected",
                "Add a contact, then connect, negotiate, run ICE checks and punch.",
            ),
            inner,
        );
        return;
    };

    let session = state.peer_for(&peer);
    let contact = state
        .contacts
        .iter()
        .find(|contact| contact.username.eq_ignore_ascii_case(&peer));
    let mut lines = vec![
        w::kv_line(
            "Connection",
            session
                .map(|s| s.state.to_string())
                .unwrap_or_else(|| "no session".to_string()),
            13,
        ),
        w::kv_line(
            "Transport",
            session
                .map(|s| s.transport.to_string())
                .unwrap_or_else(|| "—".to_string()),
            13,
        ),
        w::kv_line(
            "Punch",
            state
                .punch_state_for(&peer)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "—".to_string()),
            13,
        ),
        w::kv_line(
            "Secure",
            if state.has_secure(&peer) {
                "ESTABLISHED".to_string()
            } else {
                "—".to_string()
            },
            13,
        ),
    ];

    if let Some(contact) = contact {
        lines.push(w::kv_line("Trust", contact.trust_level.to_string(), 13));
        lines.push(w::kv_line(
            "Fingerprint",
            w::truncate(&contact.fingerprint, 28),
            13,
        ));
    }

    if let Some(ice) = state
        .ice_sessions
        .iter()
        .find(|s| s.peer.eq_ignore_ascii_case(&peer))
    {
        lines.push(Line::from(""));
        lines.push(w::section("SELECTED ICE PAIR"));
        lines.push(Line::from(Span::styled(
            format!(
                "  local   {}:{} ({})",
                ice.selected_pair.local.address,
                ice.selected_pair.local.port,
                ice.selected_pair.local.candidate_type
            ),
            Style::default().fg(LINEN),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "  remote  {}:{} ({})",
                ice.selected_pair.remote.address,
                ice.selected_pair.remote.port,
                ice.selected_pair.remote.candidate_type
            ),
            Style::default().fg(LINEN),
        )));
        lines.push(Line::from(Span::styled(
            format!("  established {}", ice.established_at),
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No ICE pair yet · press i to run ICE checks.",
            Style::default().fg(DIM),
        )));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_publication(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("CANDIDATE PUBLICATION", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(publication) = state.publication.as_ref() else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "  Not published. Press p to publish ICE candidates.",
                    Style::default().fg(DIM),
                )),
                Line::from(Span::styled(
                    "  Publication expires after 60 seconds.",
                    Style::default().fg(DIM),
                )),
            ]),
            inner,
        );
        return;
    };

    let ttl = state.publication_seconds_left().unwrap_or(0);
    let ttl_span = if ttl > 0 {
        Span::styled(format!("{}s left", ttl), Style::default().fg(GREEN).bold())
    } else {
        Span::styled("EXPIRED", Style::default().fg(RED_ACCENT).bold())
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  session  ", Style::default().fg(DIM)),
            Span::styled(
                w::truncate(&publication.session_id, 24),
                Style::default().fg(LINEN),
            ),
            Span::raw("   "),
            ttl_span,
        ]),
        Line::from(vec![
            Span::styled("  count    ", Style::default().fg(DIM)),
            Span::styled(
                publication.candidates.len().to_string(),
                Style::default().fg(CREAM),
            ),
            Span::raw(" candidates"),
        ]),
    ];
    for candidate in publication.candidates.iter().take(2) {
        lines.push(Line::from(Span::styled(
            format!("  · {}", w::truncate(&candidate.to_display_string(), 46)),
            Style::default().fg(DIM),
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_capabilities(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("LOCAL CAPABILITIES", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let caps = state.capabilities.clone().unwrap_or_default();
    let yes = |value: bool| {
        if value {
            Span::styled("YES", Style::default().fg(GREEN).bold())
        } else {
            Span::styled("no", Style::default().fg(DIM))
        }
    };
    let line = Line::from(vec![
        Span::styled("  ICE ", Style::default().fg(DIM)),
        yes(caps.supports_ice),
        Span::styled("   TURN ", Style::default().fg(DIM)),
        yes(caps.supports_turn),
        Span::styled("   Files ", Style::default().fg(DIM)),
        yes(caps.supports_files),
        Span::styled("   Images ", Style::default().fg(DIM)),
        yes(caps.supports_images),
        Span::styled("   Max ", Style::default().fg(DIM)),
        Span::styled(
            format!("{} MB", caps.max_file_size_mb),
            Style::default().fg(CREAM),
        ),
        Span::styled("   v", Style::default().fg(DIM)),
        Span::styled(caps.client_version, Style::default().fg(CREAM)),
    ]);
    frame.render_widget(Paragraph::new(line), inner);
}

// ── Registry ───────────────────────────────────────────────────────────────
fn render_registry(frame: &mut Frame, state: &AppState, area: Rect) {
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(4)]).split(area);
    render_search_bar(frame, state, rows[0]);

    let cols = Layout::horizontal([Constraint::Percentage(46), Constraint::Min(24)]).split(rows[1]);
    render_search_results(frame, state, cols[0]);
    render_lookup_detail(frame, state, cols[1]);
}

fn render_search_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("SEARCH REGISTRY", state.search_focus);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let line = if state.search_focus {
        Line::from(vec![
            Span::styled("/ ", Style::default().fg(BURNT).bold()),
            Span::styled(state.search_query.clone(), Style::default().fg(CREAM)),
            Span::styled("▏", Style::default().fg(BURNT)),
        ])
    } else if state.search_query.is_empty() {
        Line::from(vec![
            Span::styled("/ ", Style::default().fg(DIM)),
            Span::styled(
                "press / to search the registry · Enter runs the query",
                Style::default().fg(DIM).italic(),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("/ ", Style::default().fg(DIM)),
            Span::styled(state.search_query.clone(), Style::default().fg(LINEN)),
            Span::styled(
                format!("   {} result(s) · Enter again", state.search_results.len()),
                Style::default().fg(DIM),
            ),
        ])
    };
    frame.render_widget(Paragraph::new(line), inner);
}

fn render_search_results(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel(&format!("RESULTS · {}", state.search_results.len()), true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.search_results.is_empty() {
        frame.render_widget(
            w::empty_state(
                "No results",
                "Search with /, or register your own username with r.",
            ),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, result) in state.search_results.iter().enumerate() {
        let selected = index == state.search_index;
        let name_width = width.saturating_sub(3).max(8);
        lines.push(
            Line::from(vec![
                Span::styled(
                    if selected { "▸ " } else { "  " },
                    Style::default().fg(BURNT),
                ),
                Span::styled(
                    w::pad(&result.username, name_width),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );
        lines.push(Line::from(Span::styled(
            format!("    {}", result.fingerprint),
            Style::default().fg(DIM),
        )));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_lookup_detail(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("REGISTRY ENTRY", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    if let Some(user) = state.lookup.as_ref() {
        lines.push(w::section("USER"));
        lines.push(w::kv_line("Username", user.username.clone(), 13));
        lines.push(w::kv_line("Fingerprint", user.fingerprint.clone(), 13));
        let status_color = if user.status.eq_ignore_ascii_case("active") {
            GREEN
        } else {
            WHEAT
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", w::pad("Status", 13)),
                Style::default().fg(DIM),
            ),
            Span::styled(
                user.status.to_uppercase(),
                Style::default().fg(status_color).bold(),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(w::section("PUBLIC KEY"));
        for chunk in split_chunks(&user.public_key, 56) {
            lines.push(Line::from(Span::styled(
                format!("  {}", chunk),
                Style::default().fg(DIM),
            )));
        }
        lines.push(Line::from(""));
        lines.push(w::section("ACTIONS"));
        lines.push(Line::from(Span::styled(
            "  a  add as contact",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(Span::styled(
            "  L  look up again",
            Style::default().fg(CREAM),
        )));
    } else if let Some(result) = state.search_results.get(state.search_index) {
        lines.push(w::section("SELECTED RESULT"));
        lines.push(w::kv_line("Username", result.username.clone(), 13));
        lines.push(w::kv_line("Fingerprint", result.fingerprint.clone(), 13));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Enter  fetch full public profile",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(Span::styled(
            "  a      add as contact",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Verify fingerprints out-of-band before trusting a contact.",
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(w::section("RECOVERY"));
        lines.push(Line::from(Span::styled(
            "  r  register username",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(Span::styled(
            "  R  recover with recovery code",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(Span::styled(
            "  n  rename account",
            Style::default().fg(CREAM),
        )));
        lines.push(Line::from(Span::styled(
            "  D  delete account",
            Style::default().fg(RED_ACCENT),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Recovery codes are displayed once and never stored.",
            Style::default().fg(DIM),
        )));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

// ── Messages ───────────────────────────────────────────────────────────────
fn render_messages(frame: &mut Frame, state: &AppState, area: Rect) {
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(4)]).split(area);
    render_message_tabs(frame, state, rows[0]);
    render_message_table(frame, state, rows[1]);
}

fn render_message_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("MESSAGES", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tab = |label: &str, active: bool, count: usize| -> Vec<Span<'static>> {
        let style = if active {
            Style::default()
                .fg(Color::Rgb(20, 20, 22))
                .bg(BURNT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        vec![
            Span::styled(format!(" {} {} ", label, count), style),
            Span::raw("  "),
        ]
    };

    let mut spans = tab(
        "INBOX",
        state.messages_tab == MessagesTab::Inbox,
        state.chats.len(),
    );
    spans.extend(tab(
        "UNREAD",
        state.messages_tab == MessagesTab::Unread,
        state.unread.len(),
    ));
    spans.extend(tab(
        "REQUESTS",
        state.messages_tab == MessagesTab::Requests,
        state.requests.len(),
    ));
    spans.push(Span::styled("   ← → switch", Style::default().fg(DIM)));

    frame.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn render_message_table(frame: &mut Frame, state: &AppState, area: Rect) {
    match state.messages_tab {
        MessagesTab::Inbox => render_inbox_table(frame, state, area),
        MessagesTab::Unread => render_unread_table(frame, state, area),
        MessagesTab::Requests => render_requests_table(frame, state, area),
    }
}

fn render_inbox_table(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("INBOX", true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.chats.is_empty() {
        frame.render_widget(
            w::empty_state(
                "Inbox is empty",
                "Conversations appear once you add contacts.",
            ),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("  {}", w::pad("USERNAME", 22)),
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{}  ", w::pad("UNREAD", 8)),
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{}  ", w::pad("MESSAGES", 10)),
            Style::default().fg(DIM),
        ),
        Span::styled(
            format!("{}  ", w::pad("LAST ACTIVITY", 18)),
            Style::default().fg(DIM),
        ),
        Span::styled("TRUST", Style::default().fg(DIM)),
    ])];
    lines.push(w::separator(width));

    for (index, chat) in state.chats.iter().enumerate() {
        let selected = index == state.messages_index;
        let mut spans = vec![
            Span::styled(
                if selected { "▸ " } else { "  " },
                Style::default().fg(BURNT),
            ),
            Span::styled(
                w::pad(&chat.username, 22),
                Style::default()
                    .fg(if selected { CREAM } else { LINEN })
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];
        if chat.unread > 0 {
            spans.push(w::badge(
                chat.unread.to_string(),
                Color::Rgb(22, 20, 18),
                BURNT,
            ));
            spans.push(Span::raw("  "));
        } else {
            spans.push(Span::styled(w::pad("—", 8), Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            format!("{}  ", w::pad(&chat.total.to_string(), 10)),
            Style::default().fg(CREAM),
        ));
        spans.push(Span::styled(
            format!(
                "{}  ",
                w::pad(
                    &chat
                        .last_activity
                        .map(|t| t.format("%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "—".to_string()),
                    18
                )
            ),
            Style::default().fg(DIM),
        ));
        spans.push(w::trust_badge(chat.trust));

        lines.push(Line::from(spans).style(if selected {
            Style::default().bg(SELECTED_BG)
        } else {
            Style::default()
        }));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_unread_table(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("UNREAD MESSAGES", true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.unread.is_empty() {
        frame.render_widget(
            w::empty_state("Nothing unread", "You are all caught up."),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, (peer, message)) in state.unread.iter().enumerate() {
        let selected = index == state.messages_index;
        lines.push(
            Line::from(vec![
                Span::styled(
                    if selected { "▸ " } else { "  " },
                    Style::default().fg(BURNT),
                ),
                Span::styled(
                    w::pad(peer, 20),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    w::truncate(
                        &message.content.replace('\n', " "),
                        width.saturating_sub(34),
                    ),
                    Style::default().fg(CREAM),
                ),
                Span::raw("  "),
                Span::styled(
                    message.timestamp.format("%m-%d %H:%M").to_string(),
                    Style::default().fg(DIM),
                ),
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_requests_table(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("MESSAGE REQUESTS", true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.requests.is_empty() {
        frame.render_widget(
            w::empty_state("No requests", "Incoming message requests will appear here."),
            inner,
        );
        return;
    }

    let width = inner.width as usize;
    let mut lines = Vec::new();
    for (index, request) in state.requests.iter().enumerate() {
        let selected = index == state.messages_index;
        let status = request.status.to_string();
        let status_span = match request.status {
            crate::protocol::requests::RequestStatus::Pending => {
                w::badge(status, WHEAT, WARNING_BG)
            }
            crate::protocol::requests::RequestStatus::Accepted => {
                w::badge(status, Color::Rgb(196, 240, 206), POSITIVE_BG)
            }
            crate::protocol::requests::RequestStatus::Rejected => {
                w::badge(status, Color::Rgb(255, 190, 180), NEGATIVE_BG)
            }
        };
        lines.push(
            Line::from(vec![
                Span::styled(
                    if selected { "▸ " } else { "  " },
                    Style::default().fg(BURNT),
                ),
                Span::styled(
                    w::pad(&request.username, width.saturating_sub(38).max(12)),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                status_span,
                Span::raw("  "),
                Span::styled(
                    request.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(DIM),
                ),
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "a accept · x reject · Enter accepts a pending request",
        Style::default().fg(DIM),
    )));
    frame.render_widget(Paragraph::new(lines), inner);
}

// ── Settings ───────────────────────────────────────────────────────────────
fn render_settings(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([Constraint::Percentage(52), Constraint::Min(26)]).split(area);
    let left = Layout::vertical([Constraint::Length(11), Constraint::Min(6)]).split(cols[0]);

    render_identity_settings(frame, state, left[0]);
    render_local_state(frame, state, left[1]);

    let right = Layout::vertical([Constraint::Length(9), Constraint::Min(6)]).split(cols[1]);
    render_runtime_settings(frame, state, right[0]);
    render_keyboard_reference(frame, state, right[1]);
}

fn render_identity_settings(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("IDENTITY & STORAGE", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        w::kv_line("Username", state.username(), 13),
        w::kv_line("Fingerprint", state.fingerprint(), 13),
        w::kv_line("Registry", w::truncate(&state.registry_url(), 44), 13),
        w::kv_line("Storage", w::truncate(&state.storage_dir, 44), 13),
        w::kv_line(
            "Session",
            w::truncate(state.session_id.as_deref().unwrap_or("—"), 40),
            13,
        ),
        w::kv_line(
            "Profile",
            if state.has_identity() {
                "loaded".to_string()
            } else {
                "missing".to_string()
            },
            13,
        ),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_local_state(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("LOCAL STATE", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        w::kv_line("Contacts", state.contacts.len().to_string(), 13),
        w::kv_line("Conversations", state.chats.len().to_string(), 13),
        w::kv_line("Unread", state.unread.len().to_string(), 13),
        w::kv_line("Requests", state.requests.len().to_string(), 13),
        w::kv_line("Peer sessions", state.peers.len().to_string(), 13),
        w::kv_line("ICE sessions", state.ice_sessions.len().to_string(), 13),
        w::kv_line(
            "Secure sessions",
            state.secure_sessions.len().to_string(),
            13,
        ),
        w::kv_line("Punch sessions", state.punch_sessions.len().to_string(), 13),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_runtime_settings(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("RUNTIME", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        w::kv_line(
            "Presence",
            if state.online {
                "ONLINE".to_string()
            } else {
                "OFFLINE".to_string()
            },
            13,
        ),
        w::kv_line(
            "Heartbeat",
            if state.heartbeat_running {
                "RUNNING".to_string()
            } else {
                "stopped".to_string()
            },
            13,
        ),
        w::kv_line(
            "Listener",
            if state.listener_running {
                "RUNNING".to_string()
            } else {
                "stopped".to_string()
            },
            13,
        ),
        w::kv_line("Active tasks", state.running_tasks.to_string(), 13),
        w::kv_line(
            "Candidates",
            match state.publication_seconds_left() {
                Some(seconds) if seconds > 0 => format!("{}s left", seconds),
                Some(_) => "expired".to_string(),
                None => "not published".to_string(),
            },
            13,
        ),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_keyboard_reference(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = w::panel("SHORTCUTS", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            w::chip("F1-F4", BURNT),
            Span::raw(" "),
            Span::styled("screens", Style::default().fg(DIM)),
            Span::raw("   "),
            w::chip("Tab", BURNT),
            Span::raw(" "),
            Span::styled("cycle", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            w::chip("1-7", BURNT),
            Span::raw(" "),
            Span::styled("jump", Style::default().fg(DIM)),
            Span::raw("   "),
            w::chip("F5", BURNT),
            Span::raw(" "),
            Span::styled("refresh", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            w::chip("F6", BURNT),
            Span::raw(" "),
            Span::styled("online", Style::default().fg(DIM)),
            Span::raw("  "),
            w::chip("F7", BURNT),
            Span::raw(" "),
            Span::styled("listener", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            w::chip("F8", BURNT),
            Span::raw(" "),
            Span::styled("activity", Style::default().fg(DIM)),
            Span::raw(" "),
            w::chip("Ctrl+P", BURNT),
            Span::raw(" "),
            Span::styled("palette", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            w::chip("?", BURNT),
            Span::raw(" "),
            Span::styled("help", Style::default().fg(DIM)),
            Span::raw("     "),
            w::chip("q", BURNT),
            Span::raw(" "),
            Span::styled("quit", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "{} contacts · {} conversations · tick {}",
                state.contacts.len(),
                state.chats.len(),
                state.tick
            ),
            Style::default().fg(DIM),
        )),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

// ── Toasts ─────────────────────────────────────────────────────────────────
fn render_toasts(frame: &mut Frame, state: &AppState, area: Rect) {
    if state.toasts.is_empty() {
        return;
    }
    let height = state.toasts.len() as u16;
    let width = area.width.min(54);
    if area.height < height || area.width < 20 {
        return;
    }
    let rect = Rect::new(
        area.x + area.width - width,
        area.y + area.height - height,
        width,
        height,
    );
    frame.render_widget(Clear, rect);

    let mut lines = Vec::new();
    for toast in &state.toasts {
        let (icon, fg, bg) = match toast.kind {
            ToastKind::Info => ("•", Color::Rgb(210, 200, 190), Color::Rgb(44, 40, 38)),
            ToastKind::Success => ("✓", Color::Rgb(196, 240, 206), POSITIVE_BG),
            ToastKind::Error => ("✗", Color::Rgb(255, 190, 180), NEGATIVE_BG),
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(fg).bg(bg).bold()),
            Span::raw(" "),
            Span::styled(
                w::truncate(&toast.text, width.saturating_sub(5) as usize),
                Style::default().fg(CREAM).bg(PANEL_ALT),
            ),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(PANEL_ALT)),
        rect,
    );
}

// ── Overlays ───────────────────────────────────────────────────────────────
fn render_overlays(frame: &mut Frame, state: &AppState, area: Rect) {
    match state.modal.as_ref() {
        Some(Modal::Help) => render_help(frame, area),
        Some(Modal::Info(info)) => render_info(frame, &info.title, &info.lines, info.scroll, area),
        Some(Modal::Secret(secret)) => render_secret(frame, secret, area),
        Some(Modal::Form(form)) => render_form(frame, form, area),
        Some(Modal::Confirm(confirm)) => render_confirm(frame, confirm, area),
        Some(Modal::TaskLog) => render_task_log(frame, state, area),
        None => {}
    }

    if state.palette.is_some() {
        render_palette(frame, state, area);
    }
}

fn overlay_frame(frame: &mut Frame, rect: Rect, title: &str, accent: Color) {
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .title(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(PANEL_BG))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));
    frame.render_widget(block, rect);
}

fn inner_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 2,
        rect.y + 1,
        rect.width.saturating_sub(4),
        rect.height.saturating_sub(2),
    )
}

fn render_help(frame: &mut Frame, area: Rect) {
    let rect = w::centered_rect(80, 84, area);
    overlay_frame(frame, rect, "HELP · KEYBOARD REFERENCE", BURNT);
    let inner = inner_rect(rect);

    let sections: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "NAVIGATION",
            vec![
                ("F1-F4 / 1-7", "jump to a screen"),
                ("Tab / Shift+Tab", "cycle screens"),
                ("Ctrl+P or :", "fuzzy command palette"),
                ("F5", "refresh data from disk"),
                ("F6", "toggle online / offline"),
                ("F7", "toggle message listener"),
                ("F8", "activity log and task control"),
                ("? / Esc / q", "this help · close · quit"),
            ],
        ),
        (
            "DASHBOARD",
            vec![
                ("↑ ↓", "select checklist item"),
                ("Enter", "run selected step"),
                ("i / r / o", "init · register · toggle online"),
            ],
        ),
        (
            "CHAT",
            vec![
                ("↑ ↓", "select conversation"),
                ("Enter / i", "open conversation · start typing"),
                ("Enter", "send end-to-end encrypted message"),
                ("Ctrl+Q", "queue message locally"),
                ("m / x / c / v", "mark read · clear · details · verify"),
                ("PgUp / PgDn", "scroll messages"),
            ],
        ),
        (
            "CONTACTS",
            vec![
                ("a / d", "add · remove contact"),
                ("v / u / b", "verify · unverify · block"),
                ("m / s / p / c", "chat · presence · ping · connect"),
            ],
        ),
        (
            "NETWORK",
            vec![
                ("↑ ↓", "select a peer or contact target"),
                ("n / C", "network discovery · capabilities"),
                ("p / r / f", "publish · refresh · fetch candidates"),
                ("d / g / i", "discover · negotiate · ICE check"),
                ("u / t / s", "punch · punch status · secure session"),
                ("c / x / P", "connect · disconnect · ping"),
                ("T", "UDP loopback test"),
            ],
        ),
        (
            "REGISTRY",
            vec![
                ("/", "focus search"),
                ("Enter", "look up selected result"),
                ("a / L", "add contact · look up"),
                ("r / n / R / s", "register · rename · recover · restore"),
                ("l / U / D", "lock · unlock · delete account"),
            ],
        ),
        (
            "MESSAGES",
            vec![
                ("← →", "inbox · unread · requests"),
                ("↑ ↓ / Enter", "select · open"),
                ("m / a / x", "mark read · accept · reject"),
            ],
        ),
        (
            "SETTINGS",
            vec![
                ("o / l / d", "online · listener · inject test message"),
                (
                    "c / n / p / r",
                    "capabilities · netinfo · publish · refresh",
                ),
            ],
        ),
    ];

    let mut lines = Vec::new();
    for (title, items) in sections {
        lines.push(w::section(title));
        for (key, description) in items {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", w::pad(key, 15)),
                    Style::default().fg(WHEAT).bold(),
                ),
                Span::styled(description.to_string(), Style::default().fg(LINEN)),
            ]));
        }
        lines.push(Line::from(""));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((0, 0)),
        inner,
    );
}

fn render_info(frame: &mut Frame, title: &str, lines: &[String], scroll: u16, area: Rect) {
    let rect = w::centered_rect(74, 76, area);
    overlay_frame(frame, rect, title, BURNT);
    let inner = inner_rect(rect);

    let body: Vec<Line<'static>> = lines
        .iter()
        .map(|line| {
            if let Some((label, value)) = line.split_once(": ")
                && label.chars().count() <= 18
            {
                return Line::from(vec![
                    Span::styled(format!("{} ", w::pad(label, 16)), Style::default().fg(DIM)),
                    Span::styled(value.to_string(), Style::default().fg(CREAM)),
                ]);
            }
            Line::from(Span::styled(line.clone(), Style::default().fg(LINEN)))
        })
        .collect();

    let max_scroll = (body.len() as u16).saturating_sub(inner.height);
    let scroll = scroll.min(max_scroll);
    let block = Paragraph::new(body)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(block, inner);

    let hint = if max_scroll > 0 {
        format!("↑/↓ scroll · Esc close · {}/{}", scroll + 1, max_scroll + 1)
    } else {
        "Esc / Enter close".to_string()
    };
    let hint_area = Rect::new(
        rect.x + 2,
        rect.y + rect.height - 1,
        rect.width.saturating_sub(4),
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(hint, Style::default().fg(DIM))))
            .alignment(Alignment::Right),
        hint_area,
    );
}

fn render_secret(frame: &mut Frame, secret: &crate::tui::state::SecretState, area: Rect) {
    let rect = w::centered_rect(62, 46, area);
    overlay_frame(frame, rect, &secret.title, WHEAT);
    let inner = inner_rect(rect);

    let masked = "•".repeat(secret.code.chars().count().max(12));
    let code = if secret.revealed {
        Span::styled(
            secret.code.clone(),
            Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(masked, Style::default().fg(DIM))
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  RECOVERY CODE",
            Style::default().fg(DIM).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::raw("    "), code]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", secret.warning),
            Style::default().fg(RED_ACCENT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  r reveal · Esc dismiss · this code is never persisted",
            Style::default().fg(DIM),
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_form(frame: &mut Frame, form: &crate::tui::actions::FormState, area: Rect) {
    let needed = 9 + form.fields.len() as u16 * 3;
    let width = area.width.saturating_sub(10).min(78);
    let width = width.max(area.width.min(40));
    let height = needed.min(area.height.saturating_sub(4)).max(12);
    let rect = Rect::new(
        area.x + (area.width.saturating_sub(width)) / 2,
        area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    );
    overlay_frame(frame, rect, &form.title, BURNT);
    let inner = inner_rect(rect);

    let mut lines = vec![
        Line::from(Span::styled(
            form.description.clone(),
            Style::default().fg(DIM),
        )),
        Line::from(""),
    ];

    for (index, field) in form.fields.iter().enumerate() {
        let focused = index == form.focus;
        lines.push(Line::from(vec![
            Span::styled(
                if focused { "▸ " } else { "  " },
                Style::default().fg(BURNT),
            ),
            Span::styled(
                field.label.to_string(),
                Style::default()
                    .fg(if focused { WHEAT } else { DIM })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        let value = if field.secret {
            "•".repeat(field.value.chars().count())
        } else {
            field.value.clone()
        };
        let mut spans = vec![
            Span::styled("  │ ", Style::default().fg(BORDER)),
            Span::styled(value, Style::default().fg(CREAM)),
        ];
        if focused {
            spans.push(Span::styled("▏", Style::default().fg(BURNT)));
        }
        lines.push(Line::from(spans).style(if focused {
            Style::default().bg(SELECTED_BG)
        } else {
            Style::default()
        }));
        lines.push(Line::from(Span::styled(
            format!("    {}", field.hint),
            Style::default().fg(DIM).italic(),
        )));
    }

    if let Some(error) = form.error.as_ref() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  ✗ {}", error),
            Style::default().fg(RED_ACCENT),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Tab / ↑ ↓ move · Enter next or submit · Esc cancel",
        Style::default().fg(DIM),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_confirm(frame: &mut Frame, confirm: &crate::tui::state::ConfirmState, area: Rect) {
    let rect = w::centered_rect(58, 40, area);
    overlay_frame(frame, rect, &confirm.title, RED_ACCENT);
    let inner = inner_rect(rect);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", confirm.message),
            Style::default().fg(LINEN),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Action: {}", confirm.request.label()),
            Style::default().fg(DIM),
        )),
        Line::from(""),
    ];

    if let Some(phrase) = confirm.phrase.as_ref() {
        lines.push(Line::from(vec![
            Span::styled("  Type ", Style::default().fg(DIM)),
            Span::styled(phrase.clone(), Style::default().fg(RED_ACCENT).bold()),
            Span::styled(" to confirm:", Style::default().fg(DIM)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  › ", Style::default().fg(RED_ACCENT)),
            Span::styled(confirm.input.clone(), Style::default().fg(CREAM)),
            Span::styled("▏", Style::default().fg(RED_ACCENT)),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "  Press Enter to confirm, Esc to cancel.",
            Style::default().fg(DIM),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Enter confirm · Esc cancel",
        Style::default().fg(DIM),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_task_log(frame: &mut Frame, state: &AppState, area: Rect) {
    let rect = w::centered_rect(80, 80, area);
    overlay_frame(
        frame,
        rect,
        &format!("ACTIVITY LOG · {} task(s) running", state.running_tasks),
        BURNT,
    );
    let inner = inner_rect(rect);

    let lines: Vec<Line<'static>> = state
        .task_log
        .iter()
        .rev()
        .take(inner.height as usize)
        .rev()
        .map(|entry| {
            let color = match entry.kind {
                crate::tui::state::LogKind::Info => LINEN,
                crate::tui::state::LogKind::Success => GREEN,
                crate::tui::state::LogKind::Error => RED_ACCENT,
            };
            Line::from(vec![
                Span::styled(
                    format!(" {} ", entry.at.format("%H:%M:%S")),
                    Style::default().fg(DIM),
                ),
                Span::styled(entry.text.clone(), Style::default().fg(color)),
            ])
        })
        .collect();

    if lines.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No activity yet.",
                Style::default().fg(DIM),
            ))),
            inner,
        );
    } else {
        frame.render_widget(Paragraph::new(lines), inner);
    }

    let hint_area = Rect::new(
        rect.x + 2,
        rect.y + rect.height - 1,
        rect.width.saturating_sub(4),
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "x cancel all running tasks · Esc close",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Right),
        hint_area,
    );
}

fn render_palette(frame: &mut Frame, state: &AppState, area: Rect) {
    let Some(palette) = state.palette.as_ref() else {
        return;
    };
    let rect = w::centered_rect(72, 72, area);
    overlay_frame(frame, rect, "COMMAND PALETTE", BURNT);
    let inner = inner_rect(rect);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("› ", Style::default().fg(BURNT).bold()),
            Span::styled(palette.query.clone(), Style::default().fg(CREAM)),
            Span::styled("▏", Style::default().fg(BURNT)),
        ])),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(rows[1].width as usize),
            Style::default().fg(BORDER),
        ))),
        rows[1],
    );

    let items = events::filtered_commands(state);
    let width = rows[2].width as usize;
    let visible = rows[2].height as usize;
    let start = if items.is_empty() {
        0
    } else {
        palette.selected.saturating_sub(visible.saturating_sub(1))
    };

    let mut lines = Vec::new();
    if items.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No matching command.",
            Style::default().fg(DIM),
        )));
    }
    for (offset, command) in items.iter().enumerate().skip(start).take(visible) {
        let selected = offset == palette.selected;
        let label_width = width.saturating_sub(28).max(16);
        lines.push(
            Line::from(vec![
                Span::styled(
                    if selected { "▸ " } else { "  " },
                    Style::default().fg(BURNT),
                ),
                Span::styled(
                    w::pad(command.label(), label_width),
                    Style::default()
                        .fg(if selected { CREAM } else { LINEN })
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::raw("  "),
                Span::styled(command.category(), Style::default().fg(DIM)),
            ])
            .style(if selected {
                Style::default().bg(SELECTED_BG)
            } else {
                Style::default()
            }),
        );
    }
    frame.render_widget(Paragraph::new(lines), rows[2]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "↑ ↓ select · Enter run · Esc close · type to filter",
            Style::default().fg(DIM),
        )))
        .alignment(Alignment::Right),
        rows[3],
    );
}

// ── Utilities ──────────────────────────────────────────────────────────────
fn split_chunks(text: &str, size: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(size.max(1))
        .map(|chunk| chunk.iter().collect())
        .collect()
}
