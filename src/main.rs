mod app;
mod config;
mod event;
mod theme;
mod ui;

use app::{App, Worktree};
use config::Config;
use event::EventLoop;
use tracing_subscriber::{fmt, EnvFilter};

fn init_logging() {
    let _ = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .try_init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let config = Config::load().unwrap_or_default();
    let mut app = App::new(config);

    // Seed demo data so the TUI is immediately visible.
    app.worktrees = Worktree::demo();

    let handler = app::InputHandler::new(app.config.clone());

    let (event_loop, terminal, rx) = EventLoop::new()?;

    event_loop.run(terminal, app, handler, rx).await?;

    Ok(())
}
