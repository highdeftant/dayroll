use chrono::{Datelike, NaiveDate};
use dayroll::app::{month_grid, viewport_window};
use dayroll::model::Status;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::markdown::render_markdown;
use crate::ui_state::VisibleTodo;

use super::{
    C_ACCENT, C_DANGER, C_INFO, C_MUTED, C_OK, C_PANEL, C_TEXT, C_WARN, border_style, chip_style,
    priority_chip,
};

pub(super) struct NestedTasksWidget<'a> {
    pub(super) outer: Block<'a>,
    pub(super) today: Paragraph<'a>,
    pub(super) today_area: Rect,
    pub(super) overdue: Paragraph<'a>,
    pub(super) overdue_area: Rect,
    pub(super) calendar: Paragraph<'a>,
    pub(super) calendar_area: Rect,
    pub(super) today_scrollbar: Option<(Scrollbar<'a>, ScrollbarState, Rect)>,
    pub(super) overdue_scrollbar: Option<(Scrollbar<'a>, ScrollbarState, Rect)>,
}

pub(super) fn build_nested_tasks_widget(
    area: Rect,
    current_day: NaiveDate,
    selected_day: NaiveDate,
    now_time: &str,
    visible_rows: &[VisibleTodo],
    selected_index: usize,
    search_active: bool,
) -> NestedTasksWidget<'static> {
    let overdue_count = visible_rows.iter().filter(|row| row.overdue).count();
    let done = visible_rows
        .iter()
        .filter(|row| row.status == Status::Done)
        .count();
    let pending = visible_rows.len().saturating_sub(done);

    let search_chip = if !search_active {
        (
            " FILTER idle ",
            chip_style(C_MUTED, ratatui::style::Color::Rgb(55, 66, 78)),
        )
    } else {
        (
            " FILTER active ",
            chip_style(C_TEXT, ratatui::style::Color::Rgb(55, 80, 109)),
        )
    };

    let outer = Block::default()
        .title(Line::from(vec![Span::styled(
            " 日録 // DAYROLL ",
            Style::default()
                .fg(C_TEXT)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        )]))
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", current_day),
                Style::default().fg(C_INFO),
            ))
            .centered(),
        )
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", now_time),
                Style::default().fg(C_INFO).add_modifier(Modifier::DIM),
            ))
            .right_aligned(),
        )
        .title_bottom(Line::from(Span::styled(search_chip.0, search_chip.1)))
        .borders(Borders::ALL)
        .border_style(border_style());

    let inner = outer.inner(area).inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let horizontal = if inner.width >= 80 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(12), Constraint::Length(10)])
            .split(inner)
    };

    let queue_area = horizontal[0];
    let calendar_area = horizontal[1];

    let queue_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(queue_area);

    let today_area = queue_split[0];
    let overdue_area = queue_split[1];

    let today_rows: Vec<(usize, &VisibleTodo)> = visible_rows
        .iter()
        .enumerate()
        .filter(|(_, row)| !row.overdue)
        .collect();
    let overdue_rows: Vec<(usize, &VisibleTodo)> = visible_rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.overdue)
        .collect();

    let empty_message = if search_active {
        "search active"
    } else {
        "no tasks"
    };

    let (today, today_scrollbar) = draw_section_panel(
        today_area,
        " Tasks ",
        &today_rows,
        selected_index,
        empty_message,
        Some(format!(
            " todo:{} done:{} overdue:{} ",
            pending, done, overdue_count
        )),
    );
    let (overdue, overdue_scrollbar) = draw_section_panel(
        overdue_area,
        " Overdue ",
        &overdue_rows,
        selected_index,
        empty_message,
        None,
    );

    let calendar = draw_calendar_panel(selected_day);

    NestedTasksWidget {
        outer,
        today,
        today_area,
        overdue,
        overdue_area,
        calendar,
        calendar_area,
        today_scrollbar,
        overdue_scrollbar,
    }
}

fn draw_section_panel(
    area: Rect,
    title: &'static str,
    rows: &[(usize, &VisibleTodo)],
    selected_index: usize,
    empty_message: &str,
    metrics: Option<String>,
) -> (
    Paragraph<'static>,
    Option<(Scrollbar<'static>, ScrollbarState, Rect)>,
) {
    let list_height = usize::from(area.height.saturating_sub(2));
    let (start, end) = viewport_window(
        rows.len(),
        selected_index_in_rows(rows, selected_index),
        list_height,
    );

    let mut lines = Vec::<Line<'static>>::new();
    if rows.is_empty() {
        lines.push(Line::from(Span::styled(
            empty_message.to_string(),
            Style::default().fg(C_MUTED),
        )));
    } else {
        for (global_idx, row) in rows.iter().take(end).skip(start) {
            let selected = *global_idx == selected_index;
            let marker_style = if selected {
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(C_MUTED)
            };

            let status_dot_style = match row.overdue {
                true => Style::default()
                    .fg(pulsing_dot_color(C_DANGER))
                    .add_modifier(Modifier::BOLD),
                false => match row.status {
                    Status::Done => Style::default()
                        .fg(pulsing_dot_color(C_OK))
                        .add_modifier(Modifier::BOLD),
                    Status::Pending => Style::default()
                        .fg(pulsing_dot_color(C_WARN))
                        .add_modifier(Modifier::BOLD),
                },
            };

            let mut rendered = render_markdown(&row.label)
                .lines
                .first()
                .map(|line| line.spans.clone())
                .unwrap_or_else(|| vec![Span::raw(row.label.clone())]);
            for span in &mut rendered {
                span.style = Style::default().fg(C_TEXT).patch(span.style);
            }

            let prio = priority_chip(row.priority);
            let mut row_spans = vec![
                Span::styled(if selected { "▶ " } else { "  " }, marker_style),
                Span::styled("●", status_dot_style),
                Span::raw(" "),
                Span::styled(prio.0, prio.1),
                Span::raw(" "),
            ];
            row_spans.extend(rendered);
            lines.push(Line::from(row_spans));
        }
    }

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(C_TEXT).bg(C_PANEL))
        .block(
            Block::default()
                .title(title)
                .title_top(
                    metrics
                        .map(|value| {
                            Line::from(Span::styled(
                                value,
                                Style::default().fg(C_INFO).add_modifier(Modifier::DIM),
                            ))
                            .right_aligned()
                        })
                        .unwrap_or_else(|| Line::from("")),
                )
                .borders(Borders::ALL)
                .border_style(border_style()),
        );

    let selected_local = selected_index_in_rows(rows, selected_index);
    let scrollbar = if rows.len() > list_height && list_height > 0 {
        let state = ScrollbarState::new(rows.len())
            .position(start.min(selected_local))
            .viewport_content_length(list_height);
        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(C_ACCENT))
            .track_style(Style::default().fg(C_MUTED));
        let sb_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });
        Some((sb, state, sb_area))
    } else {
        None
    };

    (paragraph, scrollbar)
}

fn selected_index_in_rows(rows: &[(usize, &VisibleTodo)], selected_index: usize) -> usize {
    rows.iter()
        .position(|(global_idx, _)| *global_idx == selected_index)
        .unwrap_or(0)
}

fn draw_calendar_panel(selected_day: NaiveDate) -> Paragraph<'static> {
    let mut lines = Vec::<Line<'static>>::new();
    lines.push(Line::from(vec![Span::styled(
        format!(
            "{} {}",
            month_name(selected_day.month()),
            selected_day.year()
        ),
        Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::styled(
        selected_day.format("%a %Y-%m-%d").to_string(),
        Style::default().fg(C_INFO),
    )));
    lines.push(Line::from(Span::styled(
        " Mo Tu We Th Fr Sa Su",
        Style::default().fg(C_MUTED),
    )));

    match month_grid(selected_day) {
        Ok(cells) => {
            for week in 0..6 {
                let mut spans = Vec::<Span<'static>>::new();
                for day_col in 0..7 {
                    let idx = week * 7 + day_col;
                    match cells.get(idx).and_then(|cell| *cell) {
                        Some(date) => {
                            let style = if date == selected_day {
                                Style::default()
                                    .fg(Color::Rgb(18, 22, 28))
                                    .bg(C_ACCENT)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(C_TEXT)
                            };
                            spans.push(Span::styled(format!("{:>2}", date.day()), style));
                        }
                        None => spans.push(Span::styled("  ", Style::default().fg(C_MUTED))),
                    }
                    if day_col < 6 {
                        spans.push(Span::raw(" "));
                    }
                }
                lines.push(Line::from(spans));
            }
        }
        Err(error) => {
            lines.push(Line::from(Span::styled(
                error,
                Style::default().fg(C_DANGER),
            )));
        }
    }

    Paragraph::new(lines)
        .style(Style::default().fg(C_TEXT).bg(C_PANEL))
        .block(
            Block::default()
                .title(" Calendar ")
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn pulsing_dot_color(base: Color) -> Color {
    let dark = darker_variant(base, 0.45);
    let alpha = pulse_alpha();
    lerp_color(dark, base, alpha)
}

fn pulse_alpha() -> f32 {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0u128, |duration| duration.as_millis());
    let cycle_ms = 4200u128;
    let half = cycle_ms / 2;
    let phase = ms % cycle_ms;

    if phase <= half {
        phase as f32 / half as f32
    } else {
        (cycle_ms - phase) as f32 / half as f32
    }
}

fn darker_variant(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            ((r as f32) * factor) as u8,
            ((g as f32) * factor) as u8,
            ((b as f32) * factor) as u8,
        ),
        _ => Color::Black,
    }
}

fn lerp_color(from: Color, to: Color, alpha: f32) -> Color {
    let t = alpha.clamp(0.0, 1.0);
    match (from, to) {
        (Color::Rgb(fr, fg, fb), Color::Rgb(tr, tg, tb)) => {
            Color::Rgb(lerp_u8(fr, tr, t), lerp_u8(fg, tg, t), lerp_u8(fb, tb, t))
        }
        _ => to,
    }
}

fn lerp_u8(from: u8, to: u8, alpha: f32) -> u8 {
    let from_f = from as f32;
    let to_f = to as f32;
    (from_f + (to_f - from_f) * alpha).round() as u8
}
