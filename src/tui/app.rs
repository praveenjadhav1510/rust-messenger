use crate::tui::events::{Action, handle_events};
use crate::tui::state::AppState;
use crate::tui::ui;
use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: AppState,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let state = AppState::new();

        Ok(App { terminal, state })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Auto-refresh every 2 seconds
        let mut tick_count: u64 = 0;

        loop {
            // Draw
            self.terminal.draw(|frame| {
                ui::render(frame, &self.state);
            })?;

            // Handle events
            match handle_events(&mut self.state)? {
                Action::Quit => break,

                Action::SendMessage => {
                    let msg = self.state.input_buffer.trim().to_string();
                    self.state.input_buffer.clear();

                    if let Some(peer) = self.state.active_peer.clone() {
                        match self.send_message(&peer, &msg).await {
                            Ok(_) => {
                                self.state.status_message =
                                    Some(format!("✓ Message delivered to {}", peer));
                                self.state.status_is_error = false;
                                // Reload chat
                                self.state.messages =
                                    crate::chat::storage::load_messages(&peer)
                                        .unwrap_or_default();
                                if !self.state.messages.is_empty() {
                                    self.state.chat_scroll = self.state.messages.len() - 1;
                                }
                            }
                            Err(e) => {
                                self.state.status_message =
                                    Some(format!("✗ Send failed: {}", e));
                                self.state.status_is_error = true;
                            }
                        }
                    } else {
                        self.state.status_message =
                            Some("No contact selected.".to_string());
                        self.state.status_is_error = true;
                    }
                }

                Action::Refresh => {
                    // Already refreshed inside handle_events
                }

                Action::None => {}
            }

            // Auto-refresh every ~20 ticks (≈ 2 seconds at 100ms poll)
            tick_count += 1;
            self.state.tick = tick_count;
            if tick_count % 20 == 0 {
                self.state.refresh();
                // Clear stale status after 10 seconds
                if tick_count % 100 == 0 {
                    self.state.status_message = None;
                }
            }
        }

        Ok(())
    }

    async fn send_message(&mut self, peer: &str, text: &str) -> Result<()> {
        // Validate contact
        let contact = crate::contacts::manager::get_contact(peer)?;
        if contact.trust_level == crate::contacts::models::TrustLevel::Blocked {
            return Err(anyhow::anyhow!("Contact '{}' is blocked.", peer));
        }
        if text.is_empty() || text.len() > 4000 {
            return Err(anyhow::anyhow!("Message must be 1–4000 characters."));
        }

        // Use the same static method as the CLI send command
        let msg = crate::messaging::sender::MessageSender::send_message(peer, text).await?;
        if msg.status != crate::chat::models::MessageStatus::Delivered {
            return Err(anyhow::anyhow!("Message delivery failed (no ACK received)."));
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self
            .terminal
            .backend_mut()
            .execute(LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub async fn launch() -> Result<()> {
    let mut app = App::new()?;
    app.run().await
}
