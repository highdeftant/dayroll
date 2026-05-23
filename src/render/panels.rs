use dayroll::app::Overlay;
use dayroll::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::ui_state::{ModalState, TaskFormField};

use super::{border_style, chip_style};

pub(super) fn draw_overlay(frame: &mut ratatui::Frame<'_>, overlay: Overlay, theme: &Theme) {
    match overlay {
        Overlay::None => {}
        Overlay::Help => {
            render_scrim(frame, theme);
            let area = centered_rect(68, 60, 68, 14, frame.area());
            frame.render_widget(Clear, area);
            let text = vec![
                help_header_line(theme),
                Line::from(""),
                help_line("j/k or arrows", "move selection", theme),
                help_line("[/] or arrows", "previous / next day", theme),
                help_line("{/} or H/L", "previous / next month", theme),
                help_line("a e m d", "add, edit, move, delete", theme),
                help_line("Enter / Space", "toggle done", theme),
                help_line("/", "search mode", theme),
                help_line("u", "undo last action", theme),
                help_line("T", "next theme", theme),
                help_line("Y", "previous theme", theme),
                help_line("q", "quit confirm", theme),
                help_line("Esc", "close overlay", theme),
            ];
            let widget = Paragraph::new(text)
                .style(Style::default().fg(theme.text).bg(theme.panel))
                .block(
                    Block::default()
                        .title(" HELP ")
                        .borders(Borders::ALL)
                        .border_style(border_style(theme)),
                );
            frame.render_widget(widget, area);
        }
        Overlay::QuitConfirm => {
            render_scrim(frame, theme);
            let area = centered_rect(42, 20, 34, 9, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Quit Dayroll?",
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                help_line("y", "confirm", theme),
                help_line("n / Esc", "cancel", theme),
            ])
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .block(
                Block::default()
                    .title(Span::styled(
                        " QUIT ",
                        Style::default()
                            .fg(theme.danger)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(border_style(theme)),
            );
            frame.render_widget(widget, area);
        }
    }
}

pub(super) fn draw_modal(frame: &mut ratatui::Frame<'_>, modal: &ModalState, theme: &Theme) {
    match modal {
        ModalState::None => {}
        ModalState::MoveDate(state) => {
            render_scrim(frame, theme);
            let area = centered_rect(58, 30, 48, 11, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Move task date",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" target ", chip_style(theme.text, theme.info)),
                    Span::raw(" "),
                    Span::styled(state.date.to_string(), Style::default().fg(theme.text)),
                ]),
                Line::from(""),
                help_line("←/→", "day", theme),
                help_line("↑/↓", "week", theme),
                help_line("{/}", "month", theme),
                help_line("Enter", "apply", theme),
                help_line("Esc", "cancel", theme),
            ])
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .block(
                Block::default()
                    .title(" DATE PICKER ")
                    .borders(Borders::ALL)
                    .border_style(border_style(theme)),
            );
            frame.render_widget(widget, area);
        }
        ModalState::TaskForm(form) => {
            render_scrim(frame, theme);
            let area = centered_rect(72, 46, 64, 14, frame.area());
            frame.render_widget(Clear, area);

            let title_label = field_label_style(form.field == TaskFormField::Title, theme);
            let prio_label = field_label_style(form.field == TaskFormField::Priority, theme);
            let date_label = field_label_style(form.field == TaskFormField::Date, theme);
            let desc_label = field_label_style(form.field == TaskFormField::Description, theme);

            let mut lines = vec![
                Line::from(Span::styled(
                    if form.todo_id.is_some() {
                        "Edit task"
                    } else {
                        "Add task"
                    },
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
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
                help_line("Tab / Shift+Tab", "cycle field", theme),
                help_line("Enter", "save", theme),
                help_line("Esc", "cancel", theme),
            ];

            if let Some(error) = &form.error {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("error: {error}"),
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            let widget = Paragraph::new(lines)
                .style(Style::default().fg(theme.text).bg(theme.panel))
                .block(
                    Block::default()
                        .title(" TASK FORM ")
                        .borders(Borders::ALL)
                        .border_style(border_style(theme)),
                );
            frame.render_widget(widget, area);
        }
    }
}

fn render_scrim(frame: &mut ratatui::Frame<'_>, theme: &Theme) {
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        frame.area(),
    );
}

fn help_line(key: &str, label: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", key),
            chip_style(theme.text, Color::Rgb(61, 73, 88)),
        ),
        Span::raw("  "),
        Span::styled(label.to_string(), Style::default().fg(theme.text)),
    ])
}

fn help_header_line(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", "Keyboard"),
            chip_style(theme.text, Color::Rgb(80, 94, 112)),
        ),
        Span::raw("  "),
        Span::styled(
            "Info",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
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

fn field_label_style(active: bool, theme: &Theme) -> Style {
    if active {
        chip_style(theme.bg, theme.accent)
    } else {
        chip_style(theme.text, theme.bar)
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    min_width: u16,
    min_height: u16,
    area: Rect,
) -> Rect {
    let desired_w = area.width.saturating_mul(percent_x).saturating_div(100);
    let desired_h = area.height.saturating_mul(percent_y).saturating_div(100);

    let width = desired_w.max(min_width).min(area.width.saturating_sub(2));
    let height = desired_h.max(min_height).min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.max(1),
        height: height.max(1),
    }
}
