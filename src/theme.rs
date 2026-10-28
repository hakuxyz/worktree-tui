use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};

/// Catppuccin Mocha-inspired color palette.
/// See: https://github.com/catppuccin/catppuccin
pub mod palette {
    use super::Color;

    pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);
    pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);
    pub const PINK: Color = Color::Rgb(245, 194, 231);
    pub const MAUVE: Color = Color::Rgb(233, 181, 242);
    pub const RED: Color = Color::Rgb(243, 139, 168);
    pub const MAROON: Color = Color::Rgb(235, 160, 172);
    pub const PEACH: Color = Color::Rgb(250, 179, 135);
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);
    pub const GREEN: Color = Color::Rgb(166, 227, 161);
    pub const TEAL: Color = Color::Rgb(140, 222, 198);
    pub const SKY: Color = Color::Rgb(166, 207, 232);
    pub const SAPPHIRE: Color = Color::Rgb(116, 199, 233);
    pub const BLUE: Color = Color::Rgb(137, 180, 250);
    pub const LAVENDER: Color = Color::Rgb(211, 188, 250);
    pub const TEXT: Color = Color::Rgb(205, 214, 244);
    pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);
    pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
    pub const OVERLAY2: Color = Color::Rgb(147, 153, 178);
    pub const OVERLAY1: Color = Color::Rgb(127, 133, 153);
    pub const OVERLAY0: Color = Color::Rgb(108, 113, 134);
    pub const SURFACE2: Color = Color::Rgb(88, 91, 112);
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
    pub const BASE: Color = Color::Rgb(30, 30, 46);
    pub const MANTLE: Color = Color::Rgb(24, 24, 37);
    pub const CRUST: Color = Color::Rgb(20, 20, 33);
}

/// A named theme built from the Catppuccin palette.
#[derive(Clone, Debug)]
pub struct Theme {
    pub base: Style,
    pub surface: Style,
    pub border: Style,
    pub text: Style,
    pub muted: Style,
    pub highlight: Style,
    pub selected: Style,
    pub active: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub title: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

impl Theme {
    pub fn catppuccin_mocha() -> Self {
        Self {
            base: Style::new().bg(palette::BASE).fg(palette::TEXT),
            surface: Style::new().bg(palette::MANTLE).fg(palette::TEXT),
            border: Style::new().fg(palette::SURFACE2),
            text: Style::new().fg(palette::TEXT),
            muted: Style::new().fg(palette::SUBTEXT0),
            highlight: Style::new().fg(palette::LAVENDER).bold(),
            selected: Style::new().bg(palette::SURFACE0).fg(palette::TEXT).bold(),
            active: Style::new().bg(palette::BLUE).fg(palette::BASE).bold(),
            success: Style::new().fg(palette::GREEN),
            warning: Style::new().fg(palette::YELLOW),
            error: Style::new().fg(palette::RED),
            title: Style::new().fg(palette::PINK).bold(),
        }
    }
}
