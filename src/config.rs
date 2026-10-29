use crate::theme::{Theme, palette};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A single key or key chord bound to an action.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
    pub action: String,
}

impl KeyBinding {
    pub fn new(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Vec::new(),
            action: action.into(),
        }
    }
}

/// User-configurable keybindings and theme overrides.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub keybindings: Vec<KeyBinding>,
    #[serde(skip)]
    pub _bindings_index: Option<HashMap<String, String>>,
    pub theme: ThemeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybindings: vec![
                KeyBinding::new("q", "quit"),
                KeyBinding::new("Q", "quit_force"),
                KeyBinding::new("j", "move_down"),
                KeyBinding::new("k", "move_up"),
                KeyBinding::new("h", "move_left"),
                KeyBinding::new("l", "move_right"),
                KeyBinding::new("J", "scroll_page_down"),
                KeyBinding::new("K", "scroll_page_up"),
                KeyBinding::new("g", "go_top"),
                KeyBinding::new("G", "go_bottom"),
                KeyBinding::new("Enter", "select"),
                KeyBinding::new(" ", "toggle"),
                KeyBinding::new("a", "add_worktree"),
                KeyBinding::new("d", "remove_worktree"),
                KeyBinding::new("r", "refresh"),
                KeyBinding::new("s", "switch_tab"),
                KeyBinding::new("?", "help"),
                KeyBinding::new("Esc", "back"),
            ],
            _bindings_index: None,
            theme: ThemeConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }
        let content = std::fs::read_to_string(&path)?;
        let mut config: Self = toml::from_str(&content)?;
        config.rebuild_index();
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn config_path() -> anyhow::Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("could not determine config dir"))?
            .join("worktree-tui");
        Ok(dir.join("config.toml"))
    }

    pub fn action_for_key(&self, key: &str, modifiers: &[String]) -> Option<&str> {
        for binding in &self.keybindings {
            if binding.key == key && binding.modifiers == modifiers {
                return Some(&binding.action);
            }
        }
        None
    }

    fn rebuild_index(&mut self) {
        let mut index = HashMap::new();
        for binding in &self.keybindings {
            index.insert(binding.action.clone(), binding.key.clone());
        }
        self._bindings_index = Some(index);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub base: String,
    pub surface: String,
    pub border: String,
    pub text: String,
    pub muted: String,
    pub highlight: String,
    pub selected: String,
    pub active: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub title: String,
}

impl ThemeConfig {
    pub fn to_theme(&self) -> Theme {
        Theme {
            base: style_from_hex(&self.base, palette::TEXT),
            surface: style_from_hex(&self.surface, palette::TEXT),
            border: Style::new().fg(parse_hex(&self.border).unwrap_or(palette::SURFACE2)),
            text: Style::new().fg(parse_hex(&self.text).unwrap_or(palette::TEXT)),
            muted: Style::new().fg(parse_hex(&self.muted).unwrap_or(palette::SUBTEXT0)),
            highlight: Style::new().fg(parse_hex(&self.highlight).unwrap_or(palette::LAVENDER)).bold(),
            selected: Style::new()
                .bg(parse_hex(&self.selected).unwrap_or(palette::SURFACE0))
                .fg(palette::TEXT)
                .bold(),
            active: Style::new()
                .bg(parse_hex(&self.active).unwrap_or(palette::BLUE))
                .fg(palette::BASE)
                .bold(),
            success: Style::new().fg(parse_hex(&self.success).unwrap_or(palette::GREEN)),
            warning: Style::new().fg(parse_hex(&self.warning).unwrap_or(palette::YELLOW)),
            error: Style::new().fg(parse_hex(&self.error).unwrap_or(palette::RED)),
            title: Style::new().fg(parse_hex(&self.title).unwrap_or(palette::PINK)).bold(),
        }
    }
}

fn parse_hex(hex: &str) -> Option<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(ratatui::style::Color::Rgb(r, g, b))
}

fn style_from_hex(hex: &str, fallback_fg: ratatui::style::Color) -> ratatui::style::Style {
    match parse_hex(hex) {
        Some(color) => ratatui::style::Style::new().bg(color).fg(fallback_fg),
        None => ratatui::style::Style::new(),
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            base: "#1e1e2e".into(),
            surface: "#181825".into(),
            border: "#585b70".into(),
            text: "#cdd6f4".into(),
            muted: "#a6adc8".into(),
            highlight: "#cba6f7".into(),
            selected: "#313244".into(),
            active: "#89b4fa".into(),
            success: "#a6e3a1".into(),
            warning: "#f9e2af".into(),
            error: "#f38ba8".into(),
            title: "#f5c2e7".into(),
        }
    }
}
