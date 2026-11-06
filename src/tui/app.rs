use crate::messaging::listener::{ListenerEvent, MessageListener};
use crate::tui::actions::RunRequest;
use crate::tui::events::{self, Outcome};
use crate::tui::services::{self, ServiceData, ServiceOutput};
use crate::tui::state::{AppState, LogKind, Modal, Screen, SecretState, ToastKind};
use crate::tui::ui;
use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::AbortHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskTag {
    Generic,
    Online,
    Offline,
}

enum TaskEvent {
    Finished {
        id: u64,
        tag: TaskTag,
        output: ServiceOutput,
    },
    Failed {
        id: u64,
        tag: TaskTag,
        error: String,
    },
    Listener(ListenerEvent),
    ListenerFailed(String),
}

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: AppState,
    tx: UnboundedSender<TaskEvent>,
    next_task_id: u64,
    tags: HashMap<u64, TaskTag>,
    aborts: HashMap<u64, AbortHandle>,
    listener: Option<tokio::task::JoinHandle<()>>,
    heartbeat: crate::session::heartbeat::HeartbeatService,
    event_stop: Arc<AtomicBool>,
}

impl App {
    fn new(
        terminal: Terminal<CrosstermBackend<Stdout>>,
        tx: UnboundedSender<TaskEvent>,
        event_stop: Arc<AtomicBool>,
    ) -> Self {
        Self {
            terminal,
            state: AppState::new(),
            tx,
            next_task_id: 1,
            tags: HashMap::new(),
            aborts: HashMap::new(),
            listener: None,
            heartbeat: crate::session::heartbeat::HeartbeatService::new(),
            event_stop,
        }
    }

    async fn run(
        &mut self,
        mut rx: UnboundedReceiver<TaskEvent>,
        mut keys: UnboundedReceiver<KeyEvent>,
    ) -> Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Resume the heartbeat if the persisted session says we are online.
        if self.state.online {
            self.start_heartbeat().await;
        }

        loop {
            self.terminal.draw(|frame| {
                ui::render(frame, &self.state);
            })?;

            tokio::select! {
                Some(key) = keys.recv() => {
                    let outcome = events::handle_key(&mut self.state, key);
                    self.handle_outcome(outcome).await;
                }
                Some(event) = rx.recv() => {
                    self.handle_task_event(event).await;
                }
                _ = ticker.tick() => {
                    self.on_tick();
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn on_tick(&mut self) {
        self.state.tick = self.state.tick.wrapping_add(1);
        self.state.expire_toasts();
        if self.state.tick.is_multiple_of(20) {
            self.state.refresh_all();
        }
    }

    async fn handle_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::None => {}
            Outcome::Quit => self.state.should_quit = true,
            Outcome::Refresh => {
                self.state.refresh_all();
                self.state
                    .toast("Data refreshed from disk.", ToastKind::Success);
            }
            Outcome::Run(request) => self.spawn_request(request),
            Outcome::ToggleListener => self.toggle_listener(),
            Outcome::CancelTasks => {
                let count = self.aborts.len();
                for (_, handle) in self.aborts.drain() {
                    handle.abort();
                }
                self.tags.clear();
                self.state.running_tasks = 0;
                self.state
                    .log(format!("Cancelled {} task(s).", count), LogKind::Info);
                self.state.toast(
                    format!("Cancelled {} running task(s).", count),
                    ToastKind::Info,
                );
            }
        }
    }

    fn spawn_request(&mut self, request: RunRequest) {
        let id = self.next_task_id;
        self.next_task_id += 1;
        let label = request.label().to_string();
        let tag = match request {
            RunRequest::Online => TaskTag::Online,
            RunRequest::Offline => TaskTag::Offline,
            _ => TaskTag::Generic,
        };
        let tx = self.tx.clone();
        let handle = tokio::spawn(async move {
            match services::execute(request).await {
                Ok(output) => {
                    let _ = tx.send(TaskEvent::Finished { id, tag, output });
                }
                Err(error) => {
                    let _ = tx.send(TaskEvent::Failed {
                        id,
                        tag,
                        error: format!("{error:#}"),
                    });
                }
            }
        });
        self.aborts.insert(id, handle.abort_handle());
        self.tags.insert(id, tag);
        self.state.running_tasks += 1;
        self.state.log(format!("▶ {}", label), LogKind::Info);
    }

    async fn handle_task_event(&mut self, event: TaskEvent) {
        match event {
            TaskEvent::Finished { id, tag, output } => {
                self.finish_task(id);
                self.state
                    .log(format!("✓ {}", output.summary), LogKind::Success);
                self.apply_output(output);
                self.after_mutation(tag).await;
            }
            TaskEvent::Failed { id, tag, error } => {
                self.finish_task(id);
                self.state.log(format!("✗ {}", error), LogKind::Error);
                self.state.toast(error, ToastKind::Error);
                if tag == TaskTag::Online {
                    self.state.heartbeat_running = false;
                }
            }
            TaskEvent::Listener(event) => match event {
                ListenerEvent::Incoming { peer, content } => {
                    self.state.log(
                        format!("✉ Message from {}: {}", peer, content),
                        LogKind::Info,
                    );
                    self.state
                        .toast(format!("New message from {}", peer), ToastKind::Info);
                    self.state.refresh_all();
                    if self
                        .state
                        .current_chat_name()
                        .map(|p| p.eq_ignore_ascii_case(&peer))
                        .unwrap_or(false)
                    {
                        self.state.reload_messages();
                    }
                }
                ListenerEvent::Ack { peer, status } => {
                    self.state.log(
                        format!("✓ Delivery receipt from {} ({})", peer, status as u8),
                        LogKind::Success,
                    );
                    self.state.refresh_all();
                }
            },
            TaskEvent::ListenerFailed(error) => {
                self.listener = None;
                self.state.listener_running = false;
                self.state
                    .log(format!("✗ Listener: {}", error), LogKind::Error);
                self.state
                    .toast(format!("Listener failed: {}", error), ToastKind::Error);
            }
        }
    }

    fn finish_task(&mut self, id: u64) {
        self.aborts.remove(&id);
        if let Some(tag) = self.tags.remove(&id) {
            let _ = tag;
        }
        self.state.running_tasks = self.state.running_tasks.saturating_sub(1);
    }

    fn apply_output(&mut self, output: ServiceOutput) {
        if let Some(secret) = output.secret {
            self.state.modal = Some(Modal::Secret(SecretState {
                title: secret.title,
                code: secret.code,
                warning: secret.warning,
                revealed: false,
            }));
        }
        match output.data {
            ServiceData::None => {
                self.state.toast(output.summary, ToastKind::Success);
            }
            ServiceData::Search(results) => {
                self.state.search_results = results;
                self.state.search_index = 0;
                self.state.screen = Screen::Registry;
                self.state.toast(output.summary, ToastKind::Success);
            }
            ServiceData::Lookup(user) => {
                let lines = vec![
                    ("Username".to_string(), user.username.clone()),
                    ("Fingerprint".to_string(), user.fingerprint.clone()),
                    ("Status".to_string(), user.status.clone()),
                    ("Public key".to_string(), user.public_key.clone()),
                ];
                self.state.lookup = Some(*user);
                self.state.open_info(
                    "REGISTRY USER",
                    lines
                        .into_iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect(),
                );
            }
            ServiceData::Lines(lines) => {
                self.state.open_info(
                    output.summary,
                    lines
                        .into_iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect(),
                );
            }
        }
    }

    async fn after_mutation(&mut self, tag: TaskTag) {
        match tag {
            TaskTag::Online => {
                self.start_heartbeat().await;
            }
            TaskTag::Offline => {
                let _ = self.heartbeat.stop().await;
                self.state.heartbeat_running = false;
            }
            TaskTag::Generic => {}
        }
        self.state.refresh_all();
        if self.state.screen == Screen::Chat {
            self.state.reload_messages();
        }
    }

    async fn start_heartbeat(&mut self) {
        match self.heartbeat.start().await {
            Ok(_) => {
                self.state.heartbeat_running = true;
                self.state
                    .log("Heartbeat started (30s interval).", LogKind::Info);
            }
            Err(error) => {
                self.state.heartbeat_running = false;
                if self.state.online {
                    self.state
                        .toast(format!("Heartbeat: {}", error), ToastKind::Error);
                }
            }
        }
    }

    fn toggle_listener(&mut self) {
        if let Some(handle) = self.listener.take() {
            handle.abort();
            self.state.listener_running = false;
            self.state.log("Message listener stopped.", LogKind::Info);
            self.state.toast("Listener stopped.", ToastKind::Info);
            return;
        }

        let (listener_tx, mut listener_rx) = mpsc::unbounded_channel::<ListenerEvent>();
        let app_tx = self.tx.clone();
        tokio::spawn(async move {
            while let Some(event) = listener_rx.recv().await {
                if app_tx.send(TaskEvent::Listener(event)).is_err() {
                    break;
                }
            }
        });

        let app_tx = self.tx.clone();
        let handle = tokio::spawn(async move {
            if let Err(error) = MessageListener::run_notifying(listener_tx).await {
                let _ = app_tx.send(TaskEvent::ListenerFailed(format!("{error:#}")));
            }
        });

        self.listener = Some(handle);
        self.state.listener_running = true;
        self.state.log("Message listener started.", LogKind::Info);
        self.state
            .toast("Listening for incoming messages.", ToastKind::Success);
    }

    async fn shutdown(&mut self) {
        self.event_stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.listener.take() {
            handle.abort();
        }
        for (_, handle) in self.aborts.drain() {
            handle.abort();
        }
        let _ = self.heartbeat.stop().await;
        self.state.listener_running = false;
        self.state.heartbeat_running = false;
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.event_stop.store(true, Ordering::Relaxed);
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub async fn launch() -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let (tx, rx) = mpsc::unbounded_channel::<TaskEvent>();
    let (key_tx, keys) = mpsc::unbounded_channel::<KeyEvent>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    std::thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            match event::poll(Duration::from_millis(80)) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(key)) => {
                        if key_tx.send(key).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                },
                Ok(false) => {}
                Err(_) => break,
            }
        }
    });

    let mut app = App::new(terminal, tx, stop);
    let result = app.run(rx, keys).await;
    app.shutdown().await;
    let _ = disable_raw_mode();
    let _ = app.terminal.backend_mut().execute(LeaveAlternateScreen);
    let _ = app.terminal.show_cursor();
    result
}
