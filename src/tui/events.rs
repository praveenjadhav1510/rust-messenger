use crate::tui::state::AppState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum Action {
    Quit,
    Refresh,
    SendMessage,
    None,
}

/// Poll for a keyboard event with a 100ms timeout.
/// Returns the action the app should take.
pub fn handle_events(state: &mut AppState) -> std::io::Result<Action> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // Only handle key-press, not release
            if key.kind != KeyEventKind::Press {
                return Ok(Action::None);
            }

            match key.code {
                // Quit
                KeyCode::Esc => return Ok(Action::Quit),
                KeyCode::Char('q') if state.input_buffer.is_empty() => return Ok(Action::Quit),

                // Help overlay toggle
                KeyCode::Char('?') if state.input_buffer.is_empty() => {
                    state.show_help = !state.show_help;
                }

                // Refresh data
                KeyCode::F(5) => {
                    state.refresh();
                    state.status_message =
                        Some("Data refreshed from disk.".to_string());
                    state.status_is_error = false;
                    return Ok(Action::Refresh);
                }

                // Contact navigation
                KeyCode::Up if state.input_buffer.is_empty() => {
                    state.prev_contact();
                }
                KeyCode::Down if state.input_buffer.is_empty() => {
                    state.next_contact();
                }

                // Chat scroll
                KeyCode::PageUp => {
                    for _ in 0..5 {
                        state.scroll_chat_up();
                    }
                }
                KeyCode::PageDown => {
                    for _ in 0..5 {
                        state.scroll_chat_down();
                    }
                }

                // Send message on Enter
                KeyCode::Enter => {
                    if !state.input_buffer.trim().is_empty() {
                        return Ok(Action::SendMessage);
                    }
                }

                // Text input
                KeyCode::Backspace => {
                    state.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    // Ctrl+C
                    if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                        return Ok(Action::Quit);
                    }
                    state.input_buffer.push(c);
                }

                _ => {}
            }
        }
    }
    Ok(Action::None)
}
