//! Small, reusable presentation helpers for the TUI.
//!
//! Everything here is deliberately stateless so screens can compose widgets
//! without fighting the borrow checker.

use crate::contacts::models::TrustLevel;
use crate::tui::state::{BORDER, BURNT, CREAM, DIM, GREEN, PANEL_ALT, PANEL_BG, RED_ACCENT, WHEAT};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

/// A rounded, padded panel. `focused` paints the border with the accent.
pub fn panel(title: &str, focused: bool) -> Block<'static> {
    let border = if focused { BURNT } else { BORDER };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                title.to_string(),
                Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Left)
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0))
}

/// Panel with no title (still rounded and padded).
pub fn plain_panel(focused: bool) -> Block<'static> {
    let border = if focused { BURNT } else { BORDER };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(PANEL_BG))
        .padding(Padding::new(1, 1, 0, 0))
}

/// A tinted chip used for statuses and labels.
pub fn badge(text: impl Into<String>, fg: Color, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {} ", text.into()),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}

/// Neutral outlined-looking chip (no background).
pub fn chip(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(
        format!("[{}]", text.into()),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

pub fn muted(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(DIM))
}

pub fn accent(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(BURNT))
}

pub fn strong(text: impl Into<String>) -> Span<'static> {
    Span::styled(
        text.into(),
        Style::default().fg(CREAM).add_modifier(Modifier::BOLD),
    )
}

pub fn key_hint(key: &str, description: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(Color::Rgb(20, 20, 22))
                .bg(WHEAT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(description.to_string(), Style::default().fg(DIM)),
        Span::raw("   "),
    ]
}

pub fn trust_color(level: TrustLevel) -> Color {
    match level {
        TrustLevel::Verified => GREEN,
        TrustLevel::Unverified => WHEAT,
        TrustLevel::Blocked => RED_ACCENT,
    }
}

pub fn trust_badge(level: TrustLevel) -> Span<'static> {
    let bg = match level {
        TrustLevel::Verified => Color::Rgb(30, 60, 42),
        TrustLevel::Unverified => Color::Rgb(62, 52, 30),
        TrustLevel::Blocked => Color::Rgb(64, 28, 26),
    };
    badge(level.to_string(), trust_color(level), bg)
}

pub fn online_badge(online: bool) -> Span<'static> {
    if online {
        badge("ONLINE", Color::Rgb(196, 240, 206), Color::Rgb(30, 62, 42))
    } else {
        badge("OFFLINE", Color::Rgb(224, 196, 190), Color::Rgb(58, 38, 36))
    }
}

pub fn listener_badge(running: bool) -> Span<'static> {
    if running {
        badge(
            "LISTENING",
            Color::Rgb(196, 240, 206),
            Color::Rgb(30, 62, 42),
        )
    } else {
        badge("IDLE", Color::Rgb(210, 200, 190), Color::Rgb(48, 44, 42))
    }
}

/// Braille spinner frame.
pub fn spinner(tick: u64) -> &'static str {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    FRAMES[(tick as usize) % FRAMES.len()]
}

/// Progress bar made of block characters (no widget dependency).
pub fn progress_bar(done: usize, total: usize, width: usize) -> Line<'static> {
    if total == 0 || width == 0 {
        return Line::from("");
    }
    let filled = (done * width) / total;
    let mut spans = Vec::new();
    for i in 0..width {
        if i < filled {
            spans.push(Span::styled("█", Style::default().fg(BURNT)));
        } else {
            spans.push(Span::styled("░", Style::default().fg(BORDER)));
        }
    }
    Line::from(spans)
}

pub fn truncate(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let count = text.chars().count();
    if count <= width {
        return text.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let mut out: String = text.chars().take(width - 1).collect();
    out.push('…');
    out
}

pub fn pad(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        truncate(text, width)
    } else {
        format!("{}{}", text, " ".repeat(width - len))
    }
}

/// Insert a thin separator line into a line list.
pub fn separator(width: usize) -> Line<'static> {
    Line::from(Span::styled("─".repeat(width), Style::default().fg(BORDER)))
}

/// A two-column key/value line used by info panels.
pub fn kv_line(label: &str, value: impl Into<String>, width: usize) -> Line<'static> {
    let value = value.into();
    Line::from(vec![
        Span::styled(format!("{} ", pad(label, width)), Style::default().fg(DIM)),
        Span::styled(value, Style::default().fg(CREAM)),
    ])
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

pub fn dim_backdrop(frame: &mut ratatui::Frame, area: Rect) {
    frame.render_widget(Clear, area);
}

pub fn styled_paragraph<'a>(lines: Vec<Line<'a>>) -> Paragraph<'a> {
    Paragraph::new(lines)
        .style(Style::default().fg(CREAM))
        .wrap(Wrap { trim: false })
}

/// Section heading with a leading accent bar.
pub fn section(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("▌ ", Style::default().fg(BURNT)),
        Span::styled(
            title.to_string(),
            Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Empty-state paragraph.
pub fn empty_state(title: &str, body: &str) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            title.to_string(),
            Style::default().fg(WHEAT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(body.to_string(), Style::default().fg(DIM))),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
}

/// A horizontal rule used inside cards.
pub fn rule() -> Line<'static> {
    Line::from(Span::styled("─".repeat(80), Style::default().fg(BORDER)))
}

pub fn alt_bg() -> Style {
    Style::default().bg(PANEL_ALT)
}

pub fn bordered_full() -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
}
