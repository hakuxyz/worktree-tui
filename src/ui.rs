use crate::app::{App, Tab, Worktree};
use crate::dialog::{Dialog, InputField};
use crate::theme::{Theme, palette};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Paragraph, Tabs, BorderType, Clear,
};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    // If a dialog is open, dim the background and render the dialog on top.
    if !matches!(app.dialog, Dialog::None) {
        // Draw the normal UI first (dimmed via Clear is not needed; we just overlay).
        draw_frame(frame, app, theme);
        draw_dialog(frame, app, theme, area);
        return;
    }

    draw_frame(frame, app, theme);
}

fn draw_frame(frame: &mut Frame, app: &App, theme: &Theme) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // header
            Constraint::Min(0),      // body
            Constraint::Length(3),   // status/footer
        ])
        .split(frame.area());

    draw_header(frame, main_chunks[0], app, theme);
    draw_body(frame, main_chunks[1], app, theme);
    draw_footer(frame, main_chunks[2], app, theme);
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
    match app.active_tab {
        Tab::Worktrees => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            draw_worktree_list(frame, chunks[0], app, theme);
            draw_details_panel(frame, chunks[1], app, theme);
        }
        Tab::Branches => {
            draw_branch_list(frame, area, app, theme);
        }
        Tab::Status => {
            draw_status_panel(frame, area, app, theme);
        }
    }
}

fn draw_worktree_list(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let filtered = app.filtered_worktrees();
    let title = if app.search_query.is_empty() {
        format!(" Worktrees ({})", filtered.len())
    } else {
        format!(" Search: '{}' ({})", app.search_query, filtered.len())
    };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(&title, theme.title),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.active);

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, wt)| {
            let is_selected = i == app.selected_index;
            let style = if is_selected {
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
    if !filtered.is_empty() {
        let sel = app.selected_index.min(filtered.len().saturating_sub(1));
        list_state.select(Some(sel));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected)
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_details_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
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
                Line::from(Span::styled("Enter: open in $EDITOR", theme.muted)),
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

fn draw_branch_list(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Branches ", theme.title),
            Span::styled(format!(" ({})", app.branches.len()), theme.muted),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.active);

    let items: Vec<ListItem> = app
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let selected = i == app.selected_index;
            let style = if selected {
                theme.selected
            } else {
                theme.text
            };

            ListItem::new(Line::from(vec![
                Span::styled("●  ", style),
                Span::styled(branch.as_str(), style.add_modifier(Modifier::BOLD)),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if !app.branches.is_empty() {
        list_state.select(Some(app.selected_index.min(app.branches.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected)
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_status_panel(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Git Status ", theme.title),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(theme.surface)
        .border_style(theme.border);

    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let content = match &app.git_status {
        Some(status) => {
            let mut lines = vec![
                Line::from(Span::styled("Branch:  ", theme.muted) + Span::styled(&status.current_branch, theme.text)),
                Line::from(""),
            ];
            if status.files.is_empty() {
                lines.push(Line::from(Span::styled("Working tree clean", theme.success)));
            } else {
                for f in &status.files {
                    let (icon, color) = match f.status.as_str() {
                        "M" => (" M", theme.warning),
                        "A" => (" A", theme.success),
                        "D" => (" D", theme.error),
                        "??" => ("??", theme.muted),
                        _ => ("  ", theme.text),
                    };
                    lines.push(Line::from(vec![
                        Span::styled(icon, color),
                        Span::styled(format!("  {}", f.path), theme.text),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press 'r' to refresh", theme.muted)));
            lines
        }
        None => {
            vec![
                Line::from(Span::styled("No status available.", theme.muted)),
                Line::from(Span::styled("Press 'r' to refresh.", theme.muted)),
            ]
        }
    };

    frame.render_widget(block, area);
    let paragraph = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let help = if app.show_help {
        " q:quit  j/k:move  g/G:top-bottom  ?:help  a:add  d:remove  r:refresh  s:tab  e:editor  /:search  b:branch "
    } else {
        " q:quit  j/k:move  ?:help  s:switch tab  e:editor  /:search "
    };

    let status_text = match &app.status_message {
        Some(msg) => {
            let style = match msg.kind {
                crate::app::StatusKind::Info => theme.text,
                crate::app::StatusKind::Success => theme.success,
                crate::app::StatusKind::Warning => theme.warning,
                crate::app::StatusKind::Error => theme.error,
            };
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

/// Render a modal dialog overlay.
fn draw_dialog(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    // Dim background with a clear block.
    let clear = Clear;
    frame.render_widget(clear, area);

    match &app.dialog {
        Dialog::AddWorktree(dlg) => {
            let width = 60.min(area.width.saturating_sub(4));
            let height = 12;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 2;
            let dialog_area = Rect { x, y, width, height };

            let block = Block::default()
                .title(Line::from(vec![Span::styled(" Add Worktree ", theme.title)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(theme.surface)
                .border_style(theme.active);

            let inner = dialog_area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            });

            let path_label = if matches!(dlg.field, InputField::Path) {
                Span::styled("Path:  ", theme.muted) + Span::styled(&dlg.path, theme.text).bold()
            } else {
                Span::styled("Path:  ", theme.muted) + Span::styled(&dlg.path, theme.text)
            };
            let branch_label = if matches!(dlg.field, InputField::Branch) {
                Span::styled("Branch:", theme.muted) + Span::styled(&dlg.branch, theme.text).bold()
            } else {
                Span::styled("Branch:", theme.muted) + Span::styled(&dlg.branch, theme.text)
            };

            let mut lines = vec![
                Line::from(""),
                Line::from(path_label),
                Line::from(branch_label),
                Line::from(""),
                Line::from(Span::styled("Tab: switch field  Enter: create  Esc: cancel", theme.muted)),
            ];
            if let Some(err) = &dlg.error {
                lines.insert(1, Line::from(Span::styled(err, theme.error)));
            }

            frame.render_widget(block, dialog_area);
            let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
            frame.render_widget(paragraph, inner);
        }
        Dialog::CreateBranch(dlg) => {
            let width = 50.min(area.width.saturating_sub(4));
            let height = 10;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 2;
            let dialog_area = Rect { x, y, width, height };

            let block = Block::default()
                .title(Line::from(vec![Span::styled(" Create Branch ", theme.title)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(theme.surface)
                .border_style(theme.active);

            let inner = dialog_area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            });

            let branch_label = Span::styled("Name: ", theme.muted)
                + Span::styled(&dlg.name, theme.text);

            let mut lines = vec![
                Line::from(""),
                Line::from(branch_label),
                Line::from(""),
                Line::from(Span::styled("Enter: create  Esc: cancel", theme.muted)),
            ];
            if let Some(err) = &dlg.error {
                lines.insert(1, Line::from(Span::styled(err, theme.error)));
            }

            frame.render_widget(block, dialog_area);
            let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
            frame.render_widget(paragraph, inner);
        }
        Dialog::ConfirmRemove(path) => {
            let width = 50.min(area.width.saturating_sub(4));
            let height = 9;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 2;
            let dialog_area = Rect { x, y, width, height };

            let block = Block::default()
                .title(Line::from(vec![Span::styled(" Remove Worktree ", theme.title)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(theme.surface)
                .border_style(theme.warning);

            let inner = dialog_area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            });

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled("Remove worktree?", theme.text)),
                Line::from(Span::styled(path, theme.muted)),
                Line::from(""),
                Line::from(Span::styled("y: confirm  n/Esc: cancel", theme.muted)),
            ];

            frame.render_widget(block, dialog_area);
            let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(paragraph, inner);
        }
        Dialog::RenameBranch(_) => {}
        Dialog::None => {}
    }
}
