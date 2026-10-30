mod app;
mod config;
mod dialog;
mod event;
mod git;
mod theme;
mod ui;

use app::{App, Worktree};
use config::Config;
use event::EventLoop;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use std::io::stdout;
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
    let theme = config.theme.to_theme();

    // Determine the repo path: use current dir, or first arg.
    let repo_path = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let mut app = App::new(config).with_git(&repo_path);

    // Seed demo data so the TUI is immediately visible.
    app.worktrees = Worktree::demo();

    let handler = app::InputHandler::new(app.config.clone());

    let git = app.git.clone();
    let (event_loop, mut terminal, rx) = EventLoop::new(git, theme)?;
    terminal.resize(Rect {
        x: 0,
        y: 0,
        width: app.width,
        height: app.height,
    })?;

    event_loop.run(terminal, app, handler, rx).await?;

    Ok(())
}
