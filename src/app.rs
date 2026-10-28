use crate::config::Config;
use crossterm::event::{KeyEvent, MouseEvent};
use std::collections::HashMap;

/// Application-level state shared across panels.
#[derive(Debug, Clone)]
pub struct App {
    pub running: bool,
    pub show_help: bool,
    pub active_tab: Tab,
    pub worktrees: Vec<Worktree>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub status_message: Option<StatusMessage>,
    pub pending_tasks: HashMap<TaskId, TaskStatus>,
    pub config: Config,
    pub width: u16,
    pub height: u16,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            running: true,
            show_help: false,
            active_tab: Tab::Worktrees,
            worktrees: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            status_message: None,
            pending_tasks: HashMap::new(),
            config,
            width: 80,
            height: 24,
        }
    }

    pub fn set_dimensions(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn selected_worktree(&self) -> Option<&Worktree> {
        self.worktrees.get(self.selected_index)
    }

    pub fn status(&mut self, kind: StatusKind, message: impl Into<String>) {
        self.status_message = Some(StatusMessage { kind, message: message.into() });
    }

    pub fn add_task(&mut self, id: TaskId) {
        self.pending_tasks.insert(id, TaskStatus::Running);
    }

    pub fn complete_task(&mut self, id: TaskId, success: bool) {
        if let Some(status) = self.pending_tasks.get_mut(&id) {
            *status = if success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
        }
    }

    pub fn has_pending(&self) -> bool {
        self.pending_tasks.values().any(|s| matches!(s, TaskStatus::Running))
    }

    pub fn next(&mut self) {
        if !self.worktrees.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.worktrees.len();
            self.scroll_to_selected();
        }
    }

    pub fn previous(&mut self) {
        if !self.worktrees.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.worktrees.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.scroll_to_selected();
        }
    }

    pub fn page_down(&mut self) {
        let visible = self.visible_count().saturating_sub(1);
        if visible == 0 {
            return;
        }
        self.selected_index = self.selected_index.saturating_add(visible).min(self.worktrees.len().saturating_sub(1));
        self.scroll_to_selected();
    }

    pub fn page_up(&mut self) {
        let visible = self.visible_count().saturating_sub(1);
        if visible == 0 {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(visible);
        self.scroll_to_selected();
    }

    pub fn go_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn go_bottom(&mut self) {
        if !self.worktrees.is_empty() {
            self.selected_index = self.worktrees.len() - 1;
            self.scroll_to_selected();
        }
    }

    fn scroll_to_selected(&mut self) {
        let visible = self.visible_count();
        if visible == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible {
            self.scroll_offset = self.selected_index.saturating_sub(visible).saturating_add(1);
        }
    }

    fn visible_count(&self) -> usize {
        let rows = self.height.saturating_sub(6) as usize;
        rows.max(1)
    }

    pub fn visible_worktrees(&self) -> impl Iterator<Item = &Worktree> {
        let end = (self.scroll_offset + self.visible_count()).min(self.worktrees.len());
        self.worktrees[self.scroll_offset..end].iter()
    }

    pub fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Worktrees => Tab::Branches,
            Tab::Branches => Tab::Status,
            Tab::Status => Tab::Worktrees,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Worktrees,
    Branches,
    Status,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Worktrees => write!(f, "Worktrees"),
            Tab::Branches => write!(f, "Branches"),
            Tab::Status => write!(f, "Status"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Worktree {
    pub id: String,
    pub path: String,
    pub branch: String,
    pub commit: String,
    pub is_locked: bool,
    pub is_main: bool,
    pub commit_date: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub kind: StatusKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
}

pub type TaskId = u64;

/// Normalised actions triggered by keybindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    QuitForce,
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    ScrollPageDown,
    ScrollPageUp,
    GoTop,
    GoBottom,
    Select,
    Toggle,
    AddWorktree,
    RemoveWorktree,
    Refresh,
    SwitchTab,
    Help,
    Back,
    Unknown,
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        match s {
            "quit" => Action::Quit,
            "quit_force" => Action::QuitForce,
            "move_down" => Action::MoveDown,
            "move_up" => Action::MoveUp,
            "move_left" => Action::MoveLeft,
            "move_right" => Action::MoveRight,
            "scroll_page_down" => Action::ScrollPageDown,
            "scroll_page_up" => Action::ScrollPageUp,
            "go_top" => Action::GoTop,
            "go_bottom" => Action::GoBottom,
            "select" => Action::Select,
            "toggle" => Action::Toggle,
            "add_worktree" => Action::AddWorktree,
            "remove_worktree" => Action::RemoveWorktree,
            "refresh" => Action::Refresh,
            "switch_tab" => Action::SwitchTab,
            "help" => Action::Help,
            "back" => Action::Back,
            _ => Action::Unknown,
        }
    }
}

/// Handles routing key/mouse events to App actions using the user config.
pub struct InputHandler {
    config: Config,
}

impl InputHandler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn handle_key(&self, key: KeyEvent) -> Action {
        let modifiers: Vec<String> = key.modifiers.modifiers_to_strings();
        let key_str = key_to_string(&key);
        if let Some(action_str) = self.config.action_for_key(&key_str, &modifiers) {
            Action::from(action_str)
        } else {
            Action::Unknown
        }
    }

    pub fn apply(&self, app: &mut App, action: Action) {
        match action {
            Action::Quit | Action::QuitForce => app.running = false,
            Action::MoveDown => app.next(),
            Action::MoveUp => app.previous(),
            Action::MoveLeft => {}
            Action::MoveRight => {}
            Action::ScrollPageDown => app.page_down(),
            Action::ScrollPageUp => app.page_up(),
            Action::GoTop => app.go_top(),
            Action::GoBottom => app.go_bottom(),
            Action::Select => {}
            Action::Toggle => {}
            Action::AddWorktree => {}
            Action::RemoveWorktree => {}
            Action::Refresh => {}
            Action::SwitchTab => app.toggle_tab(),
            Action::Help => app.show_help = !app.show_help,
            Action::Back => {}
            Action::Unknown => {}
        }
    }

    pub fn handle_mouse(&self, _app: &mut App, _mouse: MouseEvent) {
        // placeholder for future mouse support
    }
}

trait ModifiersExt {
    fn modifiers_to_strings(&self) -> Vec<String>;
}

impl ModifiersExt for crossterm::event::KeyModifiers {
    fn modifiers_to_strings(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.contains(crossterm::event::KeyModifiers::SHIFT) {
            out.push("Shift".into());
        }
        if self.contains(crossterm::event::KeyModifiers::CONTROL) {
            out.push("Control".into());
        }
        if self.contains(crossterm::event::KeyModifiers::ALT) {
            out.push("Alt".into());
        }
        out
    }
}

fn key_to_string(key: &KeyEvent) -> String {
    match key.code {
        crossterm::event::KeyCode::Backspace => "Backspace".into(),
        crossterm::event::KeyCode::Enter => "Enter".into(),
        crossterm::event::KeyCode::Left => "Left".into(),
        crossterm::event::KeyCode::Right => "Right".into(),
        crossterm::event::KeyCode::Up => "Up".into(),
        crossterm::event::KeyCode::Down => "Down".into(),
        crossterm::event::KeyCode::Home => "Home".into(),
        crossterm::event::KeyCode::End => "End".into(),
        crossterm::event::KeyCode::PageUp => "PageUp".into(),
        crossterm::event::KeyCode::PageDown => "PageDown".into(),
        crossterm::event::KeyCode::Tab => "Tab".into(),
        crossterm::event::KeyCode::BackTab => "BackTab".into(),
        crossterm::event::KeyCode::Delete => "Delete".into(),
        crossterm::event::KeyCode::Insert => "Insert".into(),
        crossterm::event::KeyCode::F(n) => format!("F{}", n),
        crossterm::event::KeyCode::Null => "Null".into(),
        crossterm::event::KeyCode::Esc => "Esc".into(),
        crossterm::event::KeyCode::Char(c) => c.to_string(),
        _ => "Unknown".into(),
    }
}
