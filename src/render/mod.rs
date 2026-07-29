use chrono::{NaiveDate, NaiveTime};
use dayroll::app::{AppState, board_todos, footer_hint};
use dayroll::model::{Area, Priority};
use dayroll::theme::{Theme, theme_by_name};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui_state::{ModalState, UiViewState, VisibleTodo};

mod panels;
mod widgets;

pub(super) fn border_style(theme: &Theme) -> Style {
    Style::default().fg(theme.border)
}

pub(super) fn bar_style(theme: &Theme) -> Style {
    Style::default().fg(theme.text).bg(theme.bar)
}

pub(super) fn chip_style(fg: Color, bg: Color) -> Style {
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

fn format_task_label(title: &str, assigned_day: NaiveDate, due_time: Option<NaiveTime>) -> String {
    let date = match due_time {
        Some(time) => format!("{} {}", assigned_day, time.format("%H:%M")),
        None => assigned_day.to_string(),
    };
    format!("{title} ({date})")
}

fn format_area_chip(area: Area) -> (&'static str, Style) {
    match area {
        Area::Projects => (
            " projects ",
            chip_style(Color::Rgb(224, 184, 91), Color::Rgb(42, 44, 50)),
        ),
        Area::Home => (
            " home ",
            chip_style(Color::Rgb(138, 186, 255), Color::Rgb(42, 44, 50)),
        ),
        Area::Admin => (
            " admin ",
            chip_style(Color::Rgb(177, 140, 255), Color::Rgb(42, 44, 50)),
        ),
        Area::Personal => (
            " personal ",
            chip_style(Color::Rgb(122, 201, 172), Color::Rgb(42, 44, 50)),
        ),
        Area::Waiting => (
            " waiting ",
            chip_style(Color::Rgb(248, 177, 122), Color::Rgb(42, 44, 50)),
        ),
        Area::Inbox => (
            " inbox ",
            chip_style(Color::Rgb(175, 175, 175), Color::Rgb(42, 44, 50)),
        ),
    }
}

pub(super) fn priority_chip(priority: Priority, theme: &Theme) -> (&'static str, Style) {
    match priority {
        Priority::High => (" P1 ", chip_style(theme.text, Color::Rgb(118, 72, 38))),
        Priority::Medium => (" P2 ", chip_style(theme.text, Color::Rgb(57, 81, 108))),
        Priority::Low => (" P3 ", chip_style(theme.muted, Color::Rgb(55, 66, 78))),
    }
}

pub(crate) fn visible_todos(app: &AppState) -> Vec<VisibleTodo> {
    let mut rows = Vec::new();

    for todo in board_todos(app.todos(), app.search_query()) {
        rows.push(VisibleTodo {
            id: todo.id,
            label: format_task_label(&todo.title, todo.assigned_day, todo.due_time),
            description: todo.description.clone(),
            area: todo.area,
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
    view: UiViewState,
    modal: &ModalState,
) {
    let theme = theme_by_name(view.theme_name);

    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        frame.area(),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(14), Constraint::Length(3)])
        .split(frame.area());

    let now = chrono::Local::now();
    let tasks = widgets::build_nested_tasks_widget(widgets::TasksWidgetInput {
        area: layout[0],
        selected_day: app.selected_day(),
        now_time: &now.format("%H:%M:%S").to_string(),
        visible_rows,
        selected_index: view.selected_index,
        expanded_task: view.expanded_task,
        search_active: app.search_active(),
        theme: &theme,
        theme_name: view.theme_name,
    });

    let status_hint = footer_hint(view.overlay, app.search_active(), app.search_query());
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" status ", chip_style(theme.info, theme.bar)),
        Span::styled(format!(" {} ", status_hint.0), bar_style(&theme)),
        Span::styled(
            format!(" {} ", status_hint.1),
            chip_style(theme.text, Color::Rgb(57, 79, 106)),
        ),
    ]))
    .style(bar_style(&theme))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(&theme)),
    );

    frame.render_widget(tasks.outer, layout[0]);
    frame.render_widget(tasks.backlog, tasks.backlog_area);
    frame.render_widget(tasks.doing, tasks.doing_area);
    frame.render_widget(tasks.blocked, tasks.blocked_area);
    frame.render_widget(tasks.done, tasks.done_area);
    frame.render_widget(tasks.calendar, tasks.calendar_area);
    if let Some((scrollbar, mut state, area)) = tasks.backlog_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    if let Some((scrollbar, mut state, area)) = tasks.doing_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    if let Some((scrollbar, mut state, area)) = tasks.blocked_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    if let Some((scrollbar, mut state, area)) = tasks.done_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    frame.render_widget(status, layout[1]);

    panels::draw_modal(frame, modal, &theme);
    panels::draw_overlay(frame, view.overlay, &theme);
}
