use crate::app::{App, Tab, Worktree};
use crate::theme::{Theme, palette};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, BorderType};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = Theme::default();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // header
            Constraint::Min(0),      // body
            Constraint::Length(3),   // status/footer
        ])
        .split(frame.area());

    draw_header(frame, main_chunks[0], app, &theme);
    draw_body(frame, main_chunks[1], app, &theme);
    draw_footer(frame, main_chunks[2], app, &theme);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = Line::from(vec![
        Span::styled(" 󰊢 ", theme.title),
        Span::styled("worktree-tui ", theme.text.bold()),
        Span::styled(format!("│ {}", app.active_tab), theme.muted),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.border);

    frame.render_widget(block, area);
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    frame.render_widget(title, inner);

    // tab pills
    let tabs = Tabs::new([
        Line::from("Worktrees"),
        Line::from("Branches"),
        Line::from("Status"),
    ])
    .select(app.active_tab as usize)
    .style(theme.muted)
    .highlight_style(theme.highlight)
    .divider(Span::styled(" │ ", theme.border));

    let tab_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: 1,
    };
    frame.render_widget(tabs, tab_area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // Split body into list panel (left) and preview panel (right).
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    draw_worktree_list(frame, chunks[0], app, theme);
    draw_preview(frame, chunks[1], app, theme);
}

fn draw_worktree_list(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Worktrees ", theme.title),
            Span::styled(format!(" ({})", app.worktrees.len()), theme.muted),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(if app.active_tab == Tab::Worktrees { theme.active } else { theme.border });

    let items: Vec<ListItem> = app
        .visible_worktrees()
        .enumerate()
        .map(|(i, wt)| {
            let absolute = app.scroll_offset + i;
            let selected = absolute == app.selected_index;
            let style = if selected {
                theme.selected
            } else if wt.is_main {
                theme.success
            } else if wt.is_locked {
                theme.warning
            } else {
                theme.text
            };

            let ahead = if wt.ahead > 0 {
                format!(" ⇡{}", wt.ahead)
            } else {
                String::new()
            };
            let behind = if wt.behind > 0 {
                format!(" ⇣{}", wt.behind)
            } else {
                String::new()
            };
            let lock = if wt.is_locked { " 🔒" } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>7} ", &wt.commit[..7.min(wt.commit.len())]), theme.muted),
                Span::styled(wt.branch.as_str(), style.add_modifier(Modifier::BOLD)),
                Span::styled(lock, theme.warning),
                Span::styled(ahead + &behind, theme.muted),
                Span::styled(format!("  {}", wt.path), theme.text),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if !app.worktrees.is_empty() {
        list_state.select(Some(app.selected_index.saturating_sub(app.scroll_offset)));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected)
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Details ", theme.title),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.border);

    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let content = match app.selected_worktree() {
        Some(wt) => {
            vec![
                Line::from(Span::styled("Branch:  ", theme.muted) + Span::styled(&wt.branch, theme.text)),
                Line::from(Span::styled("Commit:  ", theme.muted) + Span::styled(&wt.commit, theme.text)),
                Line::from(Span::styled("Path:    ", theme.muted) + Span::styled(&wt.path, palette::BLUE)),
                Line::from(Span::styled("Main:    ", theme.muted) + Span::styled(
                    if wt.is_main { "yes" } else { "no" },
                    if wt.is_main { theme.success } else { theme.text },
                )),
                Line::from(Span::styled("Locked:  ", theme.muted) + Span::styled(
                    if wt.is_locked { "yes" } else { "no" },
                    if wt.is_locked { theme.warning } else { theme.text },
                )),
                Line::from(Span::styled("Ahead:   ", theme.muted) + Span::styled(wt.ahead.to_string(), theme.success)),
                Line::from(Span::styled("Behind:  ", theme.muted) + Span::styled(wt.behind.to_string(), theme.warning)),
                Line::from(""),
                Line::from(Span::styled("Press Enter to open in $EDITOR", theme.muted)),
            ]
        }
        None => {
            vec![
                Line::from(Span::styled("No worktrees found.", theme.muted)),
                Line::from(Span::styled("Press 'a' to add one.", theme.muted)),
            ]
        }
    };

    frame.render_widget(block, area);
    let paragraph = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let help = if app.show_help {
        " q:quit  j/k:move  g/G:top-bottom  ?:help  a:add  d:remove  r:refresh  s:tab  Esc:back "
    } else {
        " q:quit  j/k:move  ?:help  s:switch tab "
    };

    let status_text = match &app.status_message {
        Some(msg) => {
            format!(" {} │ {}", msg.message, help)
        }
        None => help.to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.border);

    frame.render_widget(block, area);
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let paragraph = Paragraph::new(Line::from(Span::styled(status_text, theme.muted)))
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}
