use crate::app::{Action, App, InputHandler};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, MouseEvent};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
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
}

/// Spawns the async event loop: reads crossterm events on a blocking thread,
/// forwards them via an mpsc channel, and drives Terminal rendering.
pub struct EventLoop {
    tx: mpsc::UnboundedSender<AppEvent>,
    running: Arc<AtomicBool>,
    input_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

impl EventLoop {
    pub fn new() -> anyhow::Result<(Self, Terminal<CrosstermBackend<Stdout>>, mpsc::UnboundedReceiver<AppEvent>)> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
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
        };
        Ok((this, terminal, rx))
    }

    pub async fn run(
        self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
        mut app: App,
        handler: InputHandler,
        mut rx: mpsc::UnboundedReceiver<AppEvent>,
    ) -> anyhow::Result<()> {
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
                    handler.apply(&mut app, action);
                }
                AppEvent::Mouse(mouse) => handler.handle_mouse(&mut app, mouse),
                AppEvent::Resize(w, h) => {
                    let _ = terminal.resize(ratatui::layout::Rect {
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
            }
            if !self.running.load(Ordering::SeqCst) || !app.running {
                break;
            }
        }

        self.shutdown(terminal).await?;
        Ok(())
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
        let _ = terminal::disable_raw_mode();
        Ok(())
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
