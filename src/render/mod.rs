use chrono::Local;
use dayroll::app::{AppState, DayBuckets, Overlay, footer_hint};
use dayroll::model::{Priority, Status};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui_state::{ModalState, VisibleTodo};

mod panels;
mod widgets;

pub(super) const C_BG: Color = Color::Rgb(26, 31, 38);
pub(super) const C_PANEL: Color = Color::Rgb(34, 41, 50);
pub(super) const C_BAR: Color = Color::Rgb(30, 36, 44);
pub(super) const C_BORDER: Color = Color::Rgb(95, 110, 126);
pub(super) const C_TEXT: Color = Color::Rgb(232, 238, 244);
pub(super) const C_MUTED: Color = Color::Rgb(176, 187, 198);
pub(super) const C_ACCENT: Color = Color::Rgb(233, 165, 89);
pub(super) const C_INFO: Color = Color::Rgb(142, 177, 222);
pub(super) const C_OK: Color = Color::Rgb(132, 225, 164);
pub(super) const C_WARN: Color = Color::Rgb(242, 197, 107);
pub(super) const C_DANGER: Color = Color::Rgb(236, 120, 92);

pub(super) fn border_style() -> Style {
    Style::default().fg(C_BORDER)
}

pub(super) fn bar_style() -> Style {
    Style::default().fg(C_TEXT).bg(C_BAR)
}

pub(super) fn chip_style(fg: Color, bg: Color) -> Style {
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

pub(super) fn priority_chip(priority: Priority) -> (&'static str, Style) {
    match priority {
        Priority::High => (" P1 ", chip_style(C_TEXT, Color::Rgb(118, 72, 38))),
        Priority::Medium => (" P2 ", chip_style(C_TEXT, Color::Rgb(57, 81, 108))),
        Priority::Low => (" P3 ", chip_style(C_MUTED, Color::Rgb(55, 66, 78))),
    }
}

pub(crate) fn visible_todos(app: &AppState) -> Vec<VisibleTodo> {
    let buckets = DayBuckets::for_day(app.selected_day(), app.todos());
    let filtered = buckets.filter_by_query(app.search_query());
    let mut rows = Vec::new();

    for todo in &filtered.overdue {
        rows.push(VisibleTodo {
            id: todo.id,
            label: format!("{} ({})", todo.title, todo.assigned_day),
            overdue: true,
            status: todo.status,
            priority: todo.priority,
        });
    }

    for todo in &filtered.today {
        rows.push(VisibleTodo {
            id: todo.id,
            label: todo.title.clone(),
            overdue: false,
            status: todo.status,
            priority: todo.priority,
        });
    }

    rows
}

pub(crate) fn draw_ui(
    frame: &mut ratatui::Frame<'_>,
    app: &AppState,
    visible_rows: &[VisibleTodo],
    selected_index: usize,
    modal: &ModalState,
    overlay: Overlay,
) {
    frame.render_widget(
        Block::default().style(Style::default().bg(C_BG)),
        frame.area(),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(14),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let pending = app
        .todos()
        .iter()
        .filter(|todo| todo.status == Status::Pending)
        .count();
    let done = app
        .todos()
        .iter()
        .filter(|todo| todo.status == Status::Done)
        .count();

    let search_chip = if !app.search_active() {
        (" FILTER idle ", chip_style(C_MUTED, Color::Rgb(55, 66, 78)))
    } else if app.search_query().is_empty() {
        (
            " FILTER armed ",
            chip_style(C_WARN, Color::Rgb(101, 68, 31)),
        )
    } else {
        (
            " FILTER active ",
            chip_style(C_TEXT, Color::Rgb(55, 80, 109)),
        )
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" DAYROLL ", bar_style().add_modifier(Modifier::BOLD)),
        Span::styled(search_chip.0, search_chip.1),
        Span::styled(
            format!(" day:{} ", app.selected_day()),
            chip_style(C_TEXT, Color::Rgb(58, 70, 84)),
        ),
        Span::styled(
            format!(" pending:{} ", pending),
            chip_style(C_TEXT, Color::Rgb(104, 71, 31)),
        ),
        Span::styled(
            format!(" done:{} ", done),
            chip_style(C_OK, Color::Rgb(43, 84, 61)),
        ),
        Span::styled(
            format!(" {} ", Local::now().format("%H:%M:%S")),
            bar_style().fg(C_INFO).add_modifier(Modifier::DIM),
        ),
    ]))
    .style(bar_style())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style()),
    );

    let tasks = widgets::build_nested_tasks_widget(
        layout[1],
        app.selected_day(),
        visible_rows,
        selected_index,
        app.search_active(),
        app.search_query(),
    );

    let status_hint = footer_hint(overlay, app.search_active(), app.search_query());
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" status ", chip_style(C_INFO, C_BAR)),
        Span::styled(format!(" {} ", status_hint.0), bar_style()),
        Span::styled(
            format!(" {} ", status_hint.1),
            chip_style(C_TEXT, Color::Rgb(57, 79, 106)),
        ),
    ]))
    .style(bar_style())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style()),
    );

    frame.render_widget(title, layout[0]);
    frame.render_widget(tasks.outer, layout[1]);
    frame.render_widget(tasks.today, tasks.today_area);
    frame.render_widget(tasks.overdue, tasks.overdue_area);
    frame.render_widget(tasks.calendar, tasks.calendar_area);
    if let Some((scrollbar, mut state, area)) = tasks.today_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    if let Some((scrollbar, mut state, area)) = tasks.overdue_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    frame.render_widget(status, layout[2]);

    panels::draw_modal(frame, modal);
    panels::draw_overlay(frame, overlay);
}
