use dayroll::app::{AppState, DayBuckets, footer_hint};
use dayroll::model::Priority;
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

pub(super) fn priority_chip(priority: Priority, theme: &Theme) -> (&'static str, Style) {
    match priority {
        Priority::High => (" P1 ", chip_style(theme.text, Color::Rgb(118, 72, 38))),
        Priority::Medium => (" P2 ", chip_style(theme.text, Color::Rgb(57, 81, 108))),
        Priority::Low => (" P3 ", chip_style(theme.muted, Color::Rgb(55, 66, 78))),
    }
}

pub(crate) fn visible_todos(app: &AppState) -> Vec<VisibleTodo> {
    let buckets = DayBuckets::for_day_as_of(
        app.selected_day(),
        chrono::Local::now().date_naive(),
        app.todos(),
    );
    let filtered = buckets.filter_by_query(app.search_query());
    let mut rows = Vec::new();

    for todo in &filtered.overdue {
        rows.push(VisibleTodo {
            id: todo.id,
            label: format!("{} ({})", todo.title, todo.assigned_day),
            description: todo.description.clone(),
            overdue: true,
            status: todo.status,
            priority: todo.priority,
        });
    }

    for todo in &filtered.today {
        rows.push(VisibleTodo {
            id: todo.id,
            label: todo.title.clone(),
            description: todo.description.clone(),
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
    frame.render_widget(tasks.today, tasks.today_area);
    frame.render_widget(tasks.overdue, tasks.overdue_area);
    frame.render_widget(tasks.calendar, tasks.calendar_area);
    if let Some((scrollbar, mut state, area)) = tasks.today_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    if let Some((scrollbar, mut state, area)) = tasks.overdue_scrollbar {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
    frame.render_widget(status, layout[1]);

    panels::draw_modal(frame, modal, &theme);
    panels::draw_overlay(frame, view.overlay, &theme);
}
