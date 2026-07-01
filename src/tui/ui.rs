use crate::chat::models::Direction;
use crate::contacts::models::TrustLevel;
use crate::punch::state::PunchState;
use crate::tui::state::{AppState, Screen, SetupStep, StepStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

// ── Palette ────────────────────────────────────────────────────────────────
const BG: Color = Color::Rgb(30, 30, 30);       // #1e1e1e
const CREAM: Color = Color::Rgb(242, 210, 189);  // #F2D2BD
const BURNT: Color = Color::Rgb(192, 64, 0);     // #C04000
const WHEAT: Color = Color::Rgb(245, 222, 179);  // #F5DEB3
const DARK_RED: Color = Color::Rgb(139, 0, 0);   // #8B0000
const LINEN: Color = Color::Rgb(234, 221, 202);  // #EADDCA
const RED_ACCENT: Color = Color::Rgb(255, 68, 51); // #FF4433
const DIM: Color = Color::Rgb(80, 70, 65);
const PANEL_BG: Color = Color::Rgb(24, 24, 24);

fn block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(BURNT)
    } else {
        Style::default().fg(DIM)
    };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0))
}

// ── Entry point ─────────────────────────────────────────────────────────────
pub fn render(frame: &mut Frame, state: &AppState) {
    // Fill entire terminal with bg
    frame.render_widget(
        Block::new().style(Style::default().bg(BG)),
        frame.area(),
    );

    let area = frame.area();

    // Outer layout: header / body / footer
    let outer = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(0),    // body
        Constraint::Length(5), // input + status
    ])
    .split(area);

    render_header(frame, state, outer[0]);
    render_body(frame, state, outer[1]);
    render_footer(frame, state, outer[2]);

    if state.show_help {
        render_help_overlay(frame, area);
    }
}

// ── Header ───────────────────────────────────────────────────────────────────
fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ])
    .split(area);

    // Left: app title
    let title_spans = vec![
        Span::styled("⬡ ", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(
            "RUST MESSENGER",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled("P2P · E2E-Encrypted · UDP", Style::default().fg(WHEAT)),
    ];
    let title = Paragraph::new(Line::from(title_spans))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(DIM))
                .style(Style::default().bg(PANEL_BG)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(title, cols[0]);

    // Right: identity info
    let username_str = state
        .username
        .as_deref()
        .unwrap_or("— not registered —");
    let fingerprint_str = state
        .fingerprint
        .as_deref()
        .unwrap_or("—");

    let online_indicator = if state.is_online {
        Span::styled("● ONLINE", Style::default().fg(BURNT).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("○ OFFLINE", Style::default().fg(DIM))
    };

    let peer_count = state
        .punch_sessions
        .iter()
        .filter(|s| s.state == PunchState::Established)
        .count();

    let id_spans = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(username_str, Style::default().fg(LINEN).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default().fg(DIM)),
        Span::styled(fingerprint_str, Style::default().fg(WHEAT)),
        Span::styled("   ", Style::default()),
        online_indicator,
        Span::styled(
            format!("  ╱  {} peer(s)", peer_count),
            Style::default().fg(DIM),
        ),
    ]);

    let id_para = Paragraph::new(id_spans)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(DIM))
                .style(Style::default().bg(PANEL_BG)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(id_para, cols[1]);
}

// ── Body ─────────────────────────────────────────────────────────────────────
fn render_body(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Length(36),  // wizard / setup steps
        Constraint::Min(0),      // chat
        Constraint::Length(26),  // contacts
    ])
    .split(area);

    render_wizard_panel(frame, state, cols[0]);
    render_chat_panel(frame, state, cols[1]);
    render_contacts_panel(frame, state, cols[2]);
}

// ── Setup Wizard Panel ────────────────────────────────────────────────────────
fn render_wizard_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let outer = block("  SETUP WIZARD", false).inner(area);
    frame.render_widget(block("  SETUP WIZARD", false), area);

    // Find next command suggestion
    let next_cmd = state.active_step_command();

    let inner = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(4), // next-command hint box
    ])
    .split(outer);

    // Step list
    let items: Vec<ListItem> = state
        .setup_steps
        .iter()
        .enumerate()
        .map(|(i, (step, status))| {
            let (icon, icon_style, label_style) = match status {
                StepStatus::Done => (
                    "✓",
                    Style::default().fg(BURNT),
                    Style::default().fg(DIM),
                ),
                StepStatus::Active => (
                    "▶",
                    Style::default()
                        .fg(RED_ACCENT)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(CREAM)
                        .add_modifier(Modifier::BOLD),
                ),
                StepStatus::Pending => (
                    "○",
                    Style::default().fg(DIM),
                    Style::default().fg(DIM),
                ),
            };

            let step_num = format!("{:02}.", i + 1);
            let line = Line::from(vec![
                Span::styled(format!("{} ", step_num), Style::default().fg(DIM)),
                Span::styled(format!("{} ", icon), icon_style),
                Span::styled(step.label(), label_style),
            ]);

            let desc_line = if *status == StepStatus::Active {
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        step.description(),
                        Style::default().fg(WHEAT),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        step.description(),
                        Style::default().fg(DIM),
                    ),
                ])
            };

            ListItem::new(Text::from(vec![line, desc_line]))
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(PANEL_BG));
    frame.render_widget(list, inner[0]);

    // Next command hint
    if let Some(cmd) = next_cmd {
        let hint_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BURNT))
            .style(Style::default().bg(Color::Rgb(40, 18, 10)))
            .title(Span::styled(
                " NEXT COMMAND ",
                Style::default().fg(RED_ACCENT).add_modifier(Modifier::BOLD),
            ));
        let hint_text = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("$ ", Style::default().fg(BURNT)),
                Span::styled(&cmd, Style::default().fg(CREAM).add_modifier(Modifier::BOLD)),
            ]),
        ]))
        .block(hint_block)
        .wrap(Wrap { trim: true });
        frame.render_widget(hint_text, inner[1]);
    } else {
        let ready_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BURNT))
            .style(Style::default().bg(Color::Rgb(20, 30, 20)));
        let ready_text = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("✓ ", Style::default().fg(BURNT)),
                Span::styled(
                    "All steps complete! Ready to message.",
                    Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]))
        .block(ready_block)
        .wrap(Wrap { trim: true });
        frame.render_widget(ready_text, inner[1]);
    }
}

// ── Chat Panel ────────────────────────────────────────────────────────────────
fn render_chat_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let peer_label = state
        .active_peer
        .as_deref()
        .unwrap_or("— no contact —");

    let conn_status = state
        .active_peer
        .as_ref()
        .map(|p| state.connection_status_for_peer(p))
        .unwrap_or("—");

    let conn_color = match conn_status {
        "CONNECTED" => BURNT,
        "PUNCHING" => WHEAT,
        _ => DIM,
    };

    let title = format!("  CHAT  ·  {}", peer_label);
    let chat_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BURNT))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .title_bottom(Line::from(vec![
            Span::styled(" Connection: ", Style::default().fg(DIM)),
            Span::styled(conn_status, Style::default().fg(conn_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ·  ChaCha20-Poly1305 E2E ", Style::default().fg(DIM)),
        ]))
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0));

    let inner = chat_block.inner(area);
    frame.render_widget(chat_block, area);

    if state.messages.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No messages yet.",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Type a message below and press Enter to send.",
                Style::default().fg(DIM),
            )]),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Build message lines
    let my_username = state.username.as_deref().unwrap_or("you");

    let mut all_lines: Vec<Line> = Vec::new();
    for msg in &state.messages {
        let ts = msg.timestamp.format("%H:%M").to_string();
        let is_outgoing = msg.direction == Direction::Outgoing;

        let status_icon = match msg.status {
            crate::chat::models::MessageStatus::Read => "✓✓",
            crate::chat::models::MessageStatus::Delivered => "✓",
            crate::chat::models::MessageStatus::Failed => "✗",
            _ => "…",
        };
        let status_color = match msg.status {
            crate::chat::models::MessageStatus::Read => BURNT,
            crate::chat::models::MessageStatus::Delivered => WHEAT,
            crate::chat::models::MessageStatus::Failed => DARK_RED,
            _ => DIM,
        };

        if is_outgoing {
            // Right-aligned outgoing
            all_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", my_username),
                    Style::default().fg(DIM),
                ),
                Span::styled(
                    format!("{} ", ts),
                    Style::default().fg(DIM),
                ),
            ]));
            all_lines.push(Line::from(vec![
                Span::styled("  ▶ ", Style::default().fg(BURNT)),
                Span::styled(
                    msg.content.clone(),
                    Style::default()
                        .fg(CREAM)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", status_icon),
                    Style::default().fg(status_color),
                ),
            ]));
        } else {
            // Left-aligned incoming
            let sender = state
                .active_peer
                .as_deref()
                .unwrap_or("peer");
            all_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", sender),
                    Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}", ts),
                    Style::default().fg(DIM),
                ),
            ]));
            all_lines.push(Line::from(vec![
                Span::styled("  ◀ ", Style::default().fg(LINEN)),
                Span::styled(
                    msg.content.clone(),
                    Style::default().fg(LINEN),
                ),
            ]));
        }
        all_lines.push(Line::from("")); // spacing
    }

    // Compute visible window
    let visible_height = inner.height as usize;
    let total = all_lines.len();
    let scroll_top = if total > visible_height {
        // Clamp scroll to bottom
        let max_scroll = total - visible_height;
        state.chat_scroll.min(max_scroll)
    } else {
        0
    };

    let para = Paragraph::new(Text::from(all_lines))
        .scroll((scroll_top as u16, 0));
    frame.render_widget(para, inner);

    // Scrollbar
    if total > visible_height {
        let mut scrollbar_state =
            ScrollbarState::new(total).position(scroll_top);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(DIM)),
            inner,
            &mut scrollbar_state,
        );
    }
}

// ── Contacts Panel ────────────────────────────────────────────────────────────
fn render_contacts_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let outer = block("  CONTACTS", false);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    if state.contacts.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No contacts yet.",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Run:",
                Style::default().fg(DIM),
            )]),
            Line::from(vec![Span::styled(
                "contacts add <user>",
                Style::default().fg(WHEAT),
            )]),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = state
        .contacts
        .iter()
        .enumerate()
        .map(|(i, contact)| {
            let is_selected = i == state.selected_contact_idx;

            let trust_indicator = match contact.trust_level {
                TrustLevel::Verified => Span::styled(
                    "✓",
                    Style::default().fg(BURNT).add_modifier(Modifier::BOLD),
                ),
                TrustLevel::Blocked => Span::styled(
                    "✗",
                    Style::default().fg(DARK_RED).add_modifier(Modifier::BOLD),
                ),
                TrustLevel::Unverified => Span::styled("?", Style::default().fg(DIM)),
            };

            let conn_status = state.connection_status_for_peer(&contact.username);
            let conn_dot = match conn_status {
                "CONNECTED" => Span::styled("● ", Style::default().fg(BURNT)),
                "PUNCHING" => Span::styled("◌ ", Style::default().fg(WHEAT)),
                _ => Span::styled("○ ", Style::default().fg(DIM)),
            };

            let unread = state.unread_count_for(&contact.username);
            let unread_badge = if unread > 0 {
                Span::styled(
                    format!(" [{}]", unread),
                    Style::default()
                        .fg(RED_ACCENT)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(CREAM)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(50, 25, 10))
            } else {
                Style::default().fg(LINEN)
            };

            let prefix = if is_selected {
                Span::styled("▶ ", Style::default().fg(BURNT))
            } else {
                Span::raw("  ")
            };

            let row1 = Line::from(vec![
                prefix,
                conn_dot,
                Span::styled(&contact.username, name_style),
                unread_badge,
            ]);

            let fingerprint_short = if contact.fingerprint.len() >= 9 {
                &contact.fingerprint[..9]
            } else {
                &contact.fingerprint
            };

            let row2 = Line::from(vec![
                Span::raw("    "),
                trust_indicator,
                Span::styled(
                    format!(" {} · {}", fingerprint_short, conn_status),
                    Style::default().fg(DIM),
                ),
            ]);

            ListItem::new(Text::from(vec![row1, row2, Line::raw("")]))
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(PANEL_BG));
    frame.render_widget(list, inner);
}

// ── Footer (input + status) ───────────────────────────────────────────────────
fn render_footer(frame: &mut Frame, state: &AppState, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Length(3), // input
        Constraint::Length(2), // status / keybinds
    ])
    .split(area);

    // Input bar
    let peer_hint = state
        .active_peer
        .as_deref()
        .unwrap_or("?");

    let input_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BURNT))
        .title(Span::styled(
            format!(" Message → {} ", peer_hint),
            Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(PANEL_BG));

    let input_text = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(&state.input_buffer, Style::default().fg(LINEN)),
        Span::styled(
            "▌",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::RAPID_BLINK),
        ),
    ]))
    .block(input_block);

    frame.render_widget(input_text, rows[0]);

    // Status + keybindings bar
    let status_msg = state.status_message.as_deref().unwrap_or("");
    let status_style = if state.status_is_error {
        Style::default().fg(RED_ACCENT)
    } else {
        Style::default().fg(BURNT)
    };

    let keybinds = Line::from(vec![
        Span::styled("  F5", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Refresh  ", Style::default().fg(DIM)),
        Span::styled("↑↓", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Contact  ", Style::default().fg(DIM)),
        Span::styled("PgUp/Dn", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Scroll Chat  ", Style::default().fg(DIM)),
        Span::styled("Enter", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Send  ", Style::default().fg(DIM)),
        Span::styled("?", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Help  ", Style::default().fg(DIM)),
        Span::styled("Esc", Style::default().fg(BURNT).add_modifier(Modifier::BOLD)),
        Span::styled(":Quit  ", Style::default().fg(DIM)),
        if !status_msg.is_empty() {
            Span::styled(format!("  │  {}", status_msg), status_style)
        } else {
            Span::raw("")
        },
    ]);

    let status_bar = Paragraph::new(keybinds)
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(status_bar, rows[1]);
}

// ── Help Overlay ──────────────────────────────────────────────────────────────
fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Centered popup
    let popup_area = centered_rect(60, 80, area);
    frame.render_widget(Clear, popup_area);

    let help_lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(
            "  KEYBOARD SHORTCUTS",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        keybind_line("F5", "Refresh all data from disk"),
        keybind_line("↑ / ↓", "Navigate contacts list"),
        keybind_line("PgUp / PgDn", "Scroll chat history"),
        keybind_line("Enter", "Send typed message"),
        keybind_line("Backspace", "Delete last character"),
        keybind_line("?", "Toggle this help panel"),
        keybind_line("Esc / q", "Quit the TUI"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  SETUP WIZARD",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(BURNT)),
            Span::styled("Done  ", Style::default().fg(DIM)),
            Span::styled("▶ ", Style::default().fg(RED_ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("Active (next step)  ", Style::default().fg(DIM)),
            Span::styled("○ ", Style::default().fg(DIM)),
            Span::styled("Pending", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  CONNECTION STATUS",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ● CONNECTED", Style::default().fg(BURNT)),
            Span::styled("   UDP hole-punch established", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("  ◌ PUNCHING ", Style::default().fg(WHEAT)),
            Span::styled("   Negotiating connection", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("  ○ OFFLINE  ", Style::default().fg(DIM)),
            Span::styled("   No active connection", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  MESSAGE RECEIPTS",
            Style::default()
                .fg(CREAM)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓✓ Read  ", Style::default().fg(BURNT)),
            Span::styled("✓ Delivered  ", Style::default().fg(WHEAT)),
            Span::styled("✗ Failed  ", Style::default().fg(DARK_RED)),
            Span::styled("… Pending", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press ? to close",
            Style::default()
                .fg(DIM)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let help_widget = Paragraph::new(Text::from(help_lines))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BURNT))
                .title(Span::styled(
                    " HELP ",
                    Style::default()
                        .fg(RED_ACCENT)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(Color::Rgb(22, 18, 15))),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help_widget, popup_area);
}

fn keybind_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:15}", key), Style::default().fg(CREAM).add_modifier(Modifier::BOLD)),
        Span::styled(desc, Style::default().fg(WHEAT)),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
