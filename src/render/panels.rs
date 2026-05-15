use dayroll::app::Overlay;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::ui_state::{ModalState, TaskFormField};

use super::{C_ACCENT, C_BG, C_DANGER, C_INFO, C_PANEL, C_TEXT, C_WARN, border_style, chip_style};

pub(super) fn draw_overlay(frame: &mut ratatui::Frame<'_>, overlay: Overlay) {
    match overlay {
        Overlay::None => {}
        Overlay::Help => {
            render_scrim(frame);
            let area = centered_rect(68, 60, frame.area());
            frame.render_widget(Clear, area);
            let text = vec![
                help_header_line(),
                Line::from(""),
                help_line("j/k or arrows", "move selection"),
                help_line("[/] or arrows", "previous / next day"),
                help_line("{/} or H/L", "previous / next month"),
                help_line("a e m d", "add, edit, move, delete"),
                help_line("Enter / Space", "toggle done"),
                help_line("/", "search mode"),
                help_line("u", "undo last action"),
                help_line("q", "quit confirm"),
                help_line("Esc", "close overlay"),
            ];
            let widget = Paragraph::new(text)
                .style(Style::default().fg(C_TEXT).bg(C_PANEL))
                .block(
                    Block::default()
                        .title(" HELP ")
                        .borders(Borders::ALL)
                        .border_style(border_style()),
                );
            frame.render_widget(widget, area);
        }
        Overlay::QuitConfirm => {
            render_scrim(frame);
            let area = centered_rect(42, 20, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Quit Dayroll?",
                    Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                help_line("y", "confirm"),
                help_line("n / Esc", "cancel"),
            ])
            .style(Style::default().fg(C_TEXT).bg(C_PANEL))
            .block(
                Block::default()
                    .title(Span::styled(
                        " QUIT ",
                        Style::default().fg(C_DANGER).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(border_style()),
            );
            frame.render_widget(widget, area);
        }
    }
}

pub(super) fn draw_modal(frame: &mut ratatui::Frame<'_>, modal: &ModalState) {
    match modal {
        ModalState::None => {}
        ModalState::MoveDate(state) => {
            render_scrim(frame);
            let area = centered_rect(58, 30, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Move task date",
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" target ", chip_style(C_TEXT, C_INFO)),
                    Span::raw(" "),
                    Span::styled(state.date.to_string(), Style::default().fg(C_TEXT)),
                ]),
                Line::from(""),
                help_line("←/→", "day"),
                help_line("↑/↓", "week"),
                help_line("{/}", "month"),
                help_line("Enter", "apply"),
                help_line("Esc", "cancel"),
            ])
            .style(Style::default().fg(C_TEXT).bg(C_PANEL))
            .block(
                Block::default()
                    .title(" DATE PICKER ")
                    .borders(Borders::ALL)
                    .border_style(border_style()),
            );
            frame.render_widget(widget, area);
        }
        ModalState::TaskForm(form) => {
            render_scrim(frame);
            let area = centered_rect(72, 46, frame.area());
            frame.render_widget(Clear, area);

            let title_label = if form.field == TaskFormField::Title {
                chip_style(C_BG, C_WARN)
            } else {
                chip_style(C_TEXT, C_INFO)
            };
            let prio_label = if form.field == TaskFormField::Priority {
                chip_style(C_BG, C_WARN)
            } else {
                chip_style(C_TEXT, C_INFO)
            };
            let date_label = if form.field == TaskFormField::Date {
                chip_style(C_BG, C_WARN)
            } else {
                chip_style(C_TEXT, C_INFO)
            };
            let desc_label = if form.field == TaskFormField::Description {
                chip_style(C_BG, C_WARN)
            } else {
                chip_style(C_TEXT, C_INFO)
            };

            let mut lines = vec![
                Line::from(Span::styled(
                    if form.todo_id.is_some() {
                        "Edit task"
                    } else {
                        "Add task"
                    },
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" title ", title_label),
                    Span::raw(format!(" {}", form.title)),
                ]),
                Line::from(vec![
                    Span::styled(" priority ", prio_label),
                    Span::raw(format!(" {:?}  (←/→)", form.priority)),
                ]),
                Line::from(vec![
                    Span::styled(" date ", date_label),
                    Span::raw(format!(" {}  (←/→ day, ↑/↓ week, {{/}} month)", form.date)),
                ]),
                Line::from(vec![
                    Span::styled(" description ", desc_label),
                    Span::raw(format!(" {}", description_preview(&form.description))),
                ]),
                Line::from(""),
                help_line("Tab / Shift+Tab", "cycle field"),
                help_line("Enter", "save"),
                help_line("Esc", "cancel"),
            ];

            if let Some(error) = &form.error {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("error: {error}"),
                    Style::default().fg(C_DANGER).add_modifier(Modifier::BOLD),
                )));
            }

            let widget = Paragraph::new(lines)
                .style(Style::default().fg(C_TEXT).bg(C_PANEL))
                .block(
                    Block::default()
                        .title(" TASK FORM ")
                        .borders(Borders::ALL)
                        .border_style(border_style()),
                );
            frame.render_widget(widget, area);
        }
    }
}

fn render_scrim(frame: &mut ratatui::Frame<'_>) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(22, 27, 34))),
        frame.area(),
    );
}

fn help_line(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", key),
            chip_style(C_TEXT, Color::Rgb(61, 73, 88)),
        ),
        Span::raw("  "),
        Span::styled(label.to_string(), Style::default().fg(C_TEXT)),
    ])
}

fn help_header_line() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", "Keyboard"),
            chip_style(C_TEXT, Color::Rgb(80, 94, 112)),
        ),
        Span::raw("  "),
        Span::styled(
            "Info",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn description_preview(description: &str) -> String {
    if description.is_empty() {
        return "<empty>".to_string();
    }
    let compact = description.lines().collect::<Vec<_>>().join(" ⏎ ");
    let max_len = 64;
    if compact.chars().count() > max_len {
        format!("{}...", compact.chars().take(max_len).collect::<String>())
    } else {
        compact
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100u16.saturating_sub(percent_y)) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100u16.saturating_sub(percent_y)) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100u16.saturating_sub(percent_x)) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100u16.saturating_sub(percent_x)) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
