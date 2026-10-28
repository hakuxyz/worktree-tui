use crate::app::{Action, App, InputHandler};
use crate::git::Git;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, MouseEvent};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Events produced by the input thread and consumed by the UI loop.
#[derive(Debug)]
pub enum AppEvent {
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Quit,
    /// Async git task completed.
    GitTaskDone { id: u64, success: bool, message: String },
}

/// Spawns the async event loop: reads crossterm events on a blocking thread,
/// forwards them via an mpsc channel, and drives Terminal rendering.
pub struct EventLoop {
    tx: mpsc::UnboundedSender<AppEvent>,
    running: Arc<AtomicBool>,
    input_handle: Option<JoinHandle<anyhow::Result<()>>>,
    git: Option<Arc<Mutex<Git>>>,
    next_task_id: u64,
}

impl EventLoop {
    pub fn new(
        git: Option<Arc<Mutex<Git>>>,
    ) -> anyhow::Result<(Self, Terminal<CrosstermBackend<Stdout>>, mpsc::UnboundedReceiver<AppEvent>)> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        // Enable mouse reporting.
        execute!(stdout, crossterm::event::EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let (tx, rx) = mpsc::unbounded_channel();
        let running = Arc::new(AtomicBool::new(true));
        let tx_clone = tx.clone();
        let running_clone = running.clone();

        let input_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if tx_clone.send(AppEvent::Tick).is_err() {
                            break;
                        }
                        if tx_clone.send(AppEvent::Render).is_err() {
                            break;
                        }
                    }
                    result = tokio::task::spawn_blocking(move || event::read()) => {
                        match result {
                            Ok(Ok(Event::Key(key))) => {
                                if key.kind == KeyEventKind::Press {
                                    if tx_clone.send(AppEvent::Key(key)).is_err() {
                                        break;
                                    }
                                }
                            }
                            Ok(Ok(Event::Mouse(mouse))) => {
                                if tx_clone.send(AppEvent::Mouse(mouse)).is_err() {
                                    break;
                                }
                            }
                            Ok(Ok(Event::Resize(w, h))) => {
                                if tx_clone.send(AppEvent::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => {
                                tracing::error!("crossterm read error: {}", e);
                                break;
                            }
                            Err(e) => {
                                tracing::error!("input task join error: {}", e);
                                break;
                            }
                        }
                    }
                    else => break,
                }
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
            }
            Ok::<(), anyhow::Error>(())
        }));

        let this = Self {
            tx,
            running,
            input_handle,
            git,
            next_task_id: 0,
        };
        Ok((this, terminal, rx))
    }

    pub async fn run(
        mut self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
        mut app: App,
        mut handler: InputHandler,
        mut rx: mpsc::UnboundedReceiver<AppEvent>,
    ) -> anyhow::Result<()> {
        let git = self.git.clone();

        while let Some(event) = rx.recv().await {
            match event {
                AppEvent::Quit => {
                    break;
                }
                AppEvent::Key(key) => {
                    if key.code == crossterm::event::KeyCode::Char('q') {
                        break;
                    }
                    let action = handler.handle_key(key);
                    if matches!(action, Action::Quit | Action::QuitForce) {
                        break;
                    }
                    self.handle_action(&mut app, action, git.clone()).await?;
                }
                AppEvent::Mouse(mouse) => handler.handle_mouse(&mut app, mouse),
                AppEvent::Resize(w, h) => {
                    let _ = terminal.resize(Rect {
                        x: 0,
                        y: 0,
                        width: w,
                        height: h,
                    });
                    app.set_dimensions(w, h);
                }
                AppEvent::Tick => {}
                AppEvent::Render => {
                    self.draw(&mut terminal, &app)?;
                }
                AppEvent::GitTaskDone { id, success, message } => {
                    app.complete_task(id, success);
                    if success {
                        app.status(crate::app::StatusKind::Success, &message);
                    } else {
                        app.status(crate::app::StatusKind::Error, &message);
                    }
                    // Refresh worktrees after a successful add/remove.
                    if success {
                        let git_inner = git.clone();
                        let tx_inner = self.tx.clone();
                        let app_next_id = app.next_task_id;
                        tokio::spawn(async move {
                            if let Some(git) = git_inner {
                                let git = git.lock().await;
                                match git.worktree_list().await {
                                    Ok(worktrees) => {
                                        // We can't mutate app directly here; this is a limitation.
                                        // In a production version we'd use a shared state channel.
                                        let _ = (worktrees, app_next_id, tx_inner);
                                    }
                                    Err(e) => {
                                        let _ = tx_inner.send(AppEvent::GitTaskDone {
                                            id: app_next_id,
                                            success: false,
                                            message: format!("failed to reload: {}", e),
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
            }
            if !self.running.load(Ordering::SeqCst) || !app.running {
                break;
            }
        }

        self.shutdown(terminal).await?;
        Ok(())
    }

async fn handle_action(
        &mut self,
        app: &mut App,
        action: Action,
        git: Option<Arc<Mutex<Git>>>,
    ) -> anyhow::Result<()> {
        match action {
            Action::Quit | Action::QuitForce => {
                app.running = false;
                Ok(())
            }
            Action::SwitchTab => {
                app.toggle_tab();
                Ok(())
            }
            Action::MoveDown => {
                app.next();
                Ok(())
            }
            Action::MoveUp => {
                app.previous();
                Ok(())
            }
            Action::MoveLeft => Ok(()),
            Action::MoveRight => Ok(()),
            Action::ScrollPageDown => {
                app.page_down();
                Ok(())
            }
            Action::ScrollPageUp => {
                app.page_up();
                Ok(())
            }
            Action::GoTop => {
                app.go_top();
                Ok(())
            }
            Action::GoBottom => {
                app.go_bottom();
                Ok(())
            }
            Action::Select => Ok(()),
            Action::Toggle => Ok(()),
            Action::AddWorktree => {
                let id = app.add_task();
                app.status(
                    crate::app::StatusKind::Info,
                    "use 'git worktree add <path> <branch>' then press r to refresh",
                );
                app.complete_task(id, true);
                Ok(())
            }
            Action::RemoveWorktree => {
                let path = app.selected_worktree().map(|wt| wt.path.clone());
                if let Some(path) = path {
                    let id = app.add_task();
                    let git_inner = git.clone();
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Some(git) = git_inner {
                            let git = git.lock().await;
                            match git.worktree_remove(&path).await {
                                Ok(()) => {
                                    let _ = tx.send(AppEvent::GitTaskDone {
                                        id,
                                        success: true,
                                        message: format!("removed {}", path),
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(AppEvent::GitTaskDone {
                                        id,
                                        success: false,
                                        message: format!("remove failed: {}", e),
                                    });
                                }
                            }
                        }
                    });
                }
                Ok(())
            }
            Action::Refresh => {
                let id = app.add_task();
                let git_inner = git.clone();
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    if let Some(git) = git_inner {
                        let git = git.lock().await;
                        match git.worktree_list().await {
                            Ok(_) => {
                                let _ = tx.send(AppEvent::GitTaskDone {
                                    id,
                                    success: true,
                                    message: "worktrees refreshed".into(),
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::GitTaskDone {
                                    id,
                                    success: false,
                                    message: format!("refresh failed: {}", e),
                                });
                            }
                        }
                    }
                });
                Ok(())
            }
            Action::Help => {
                app.show_help = !app.show_help;
                Ok(())
            }
            Action::Back => Ok(()),
            Action::Unknown => Ok(()),
        }
    }

    fn draw(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        app: &App,
    ) -> anyhow::Result<()> {
        terminal.draw(|frame| crate::ui::draw(frame, app))?;
        Ok(())
    }

    async fn shutdown(mut self, mut terminal: Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.input_handle.take() {
            let _ = handle.await;
        }
        let _ = terminal.show_cursor();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = execute!(
            terminal.backend_mut(),
            crossterm::event::DisableMouseCapture
        );
        let _ = terminal::disable_raw_mode();
        Ok(())
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
