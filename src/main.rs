use std::error::Error;
use std::io;
use std::time::Duration;

use chrono::{Datelike, Local, NaiveDate};
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use dayroll::app::{AppState, DayBuckets, month_grid, viewport_window};
use dayroll::model::{Priority, Status};
use dayroll::storage::{Store, TodoStore};
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

const COLOR_VOID: Color = Color::Rgb(24, 28, 34);
const COLOR_STEEL: Color = Color::Rgb(72, 82, 96);
const COLOR_GHOST: Color = Color::Rgb(214, 221, 230);
const COLOR_GREEN: Color = Color::Rgb(134, 239, 172);
const COLOR_AMBER: Color = Color::Rgb(252, 211, 77);
const COLOR_RED: Color = Color::Rgb(248, 113, 113);
const COLOR_CYAN: Color = Color::Rgb(147, 197, 253);

#[derive(Debug, Clone)]
struct VisibleTodo {
    id: uuid::Uuid,
    label: String,
    overdue: bool,
    status: Status,
}

fn main() -> Result<(), Box<dyn Error>> {
    let result = run_app();
    if let Err(error) = result {
        eprintln!("dayroll error: {error}");
        return Err(Box::new(io::Error::other(error)));
    }
    Ok(())
}

fn run_app() -> Result<(), String> {
    let today = Local::now().date_naive();
    let store = Store::new_file(Store::default_path());
    let todos = store.load()?;
    let mut app = AppState::with_todos(today, todos);
    let mut selected_index = 0usize;

    enable_raw_mode().map_err(|error| format!("failed to enable raw mode: {error}"))?;
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)
        .map_err(|error| format!("failed to enter alt screen: {error}"))?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|error| format!("terminal init failed: {error}"))?;

    loop {
        let visible_rows = visible_todos(&app);
        if selected_index >= visible_rows.len() && !visible_rows.is_empty() {
            selected_index = visible_rows.len().saturating_sub(1);
        }
        if visible_rows.is_empty() {
            selected_index = 0;
        }

        terminal
            .draw(|frame| draw_ui(frame, &app, &visible_rows, selected_index))
            .map_err(|error| format!("draw failed: {error}"))?;

        if !event::poll(Duration::from_millis(250))
            .map_err(|error| format!("event poll failed: {error}"))?
        {
            continue;
        }

        let key_event =
            match event::read().map_err(|error| format!("event read failed: {error}"))? {
                Event::Key(key) => key,
                _ => continue,
            };

        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Char(']') | KeyCode::Right => {
                app.select_next_day();
                selected_index = 0;
            }
            KeyCode::Char('[') | KeyCode::Left => {
                app.select_prev_day();
                selected_index = 0;
            }
            KeyCode::Char('}') | KeyCode::Char('L') => {
                app.select_next_month();
                selected_index = 0;
            }
            KeyCode::Char('{') | KeyCode::Char('H') => {
                app.select_prev_month();
                selected_index = 0;
            }
            KeyCode::Char('t') => {
                let now = Local::now().date_naive();
                app.set_selected_day(now);
                selected_index = 0;
            }
            KeyCode::Char('a') => {
                let title = format!("New task {}", Local::now().format("%H:%M:%S"));
                app.add_todo(title, Priority::Medium, app.selected_day());
                store.save(app.todos())?;
            }
            KeyCode::Char('m') => {
                if let Some(row) = visible_rows.get(selected_index) {
                    if let Some(todo) = app.todo(row.id) {
                        let next_day = match todo.assigned_day.succ_opt() {
                            Some(day) => day,
                            None => todo.assigned_day,
                        };
                        app.move_todo(row.id, next_day)?;
                        store.save(app.todos())?;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(row) = visible_rows.get(selected_index) {
                    app.toggle_done(row.id)?;
                    store.save(app.todos())?;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if selected_index + 1 < visible_rows.len() {
                    selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if selected_index > 0 {
                    selected_index = selected_index.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    disable_raw_mode().map_err(|error| format!("failed to disable raw mode: {error}"))?;
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .map_err(|error| format!("failed leaving alt screen: {error}"))?;
    Ok(())
}

fn visible_todos(app: &AppState) -> Vec<VisibleTodo> {
    let buckets = DayBuckets::for_day(app.selected_day(), app.todos());
    let mut rows = Vec::new();

    for todo in &buckets.overdue {
        rows.push(VisibleTodo {
            id: todo.id,
            label: format!("{} ({})", todo.title, todo.assigned_day),
            overdue: true,
            status: todo.status,
        });
    }

    for todo in &buckets.today {
        rows.push(VisibleTodo {
            id: todo.id,
            label: todo.title.clone(),
            overdue: false,
            status: todo.status,
        });
    }

    rows
}

fn draw_ui(
    frame: &mut ratatui::Frame<'_>,
    app: &AppState,
    visible_rows: &[VisibleTodo],
    selected_index: usize,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(20)])
        .split(layout[1]);

    let pending_count = app
        .todos()
        .iter()
        .filter(|todo| todo.status == Status::Pending)
        .count();
    let done_count = app
        .todos()
        .iter()
        .filter(|todo| todo.status == Status::Done)
        .count();

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " DAYROLL ",
            Style::default()
                .fg(COLOR_GHOST)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                " {}  pending:{} done:{} ",
                app.selected_day(),
                pending_count,
                done_count
            ),
            Style::default().fg(COLOR_GREEN),
        ),
    ]))
    .style(Style::default().bg(COLOR_VOID))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_STEEL)),
    );

    let calendar = draw_calendar_widget(app.selected_day());

    let tasks = draw_tasks_widget(middle[1], visible_rows, selected_index);

    let status = Paragraph::new(
        "[a] add [space/enter] toggle [m] move +1d [[/]] day [{/}] month [t] today [q] quit",
    )
    .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_STEEL)),
    );

    frame.render_widget(title, layout[0]);
    frame.render_widget(calendar, middle[0]);
    frame.render_widget(tasks.0, middle[1]);

    if let Some((scrollbar, mut state, area)) = tasks.1 {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    frame.render_widget(status, layout[2]);
}

fn draw_calendar_widget(selected_day: NaiveDate) -> Paragraph<'static> {
    let mut lines = Vec::<Line<'static>>::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} {}",
            month_name(selected_day.month()),
            selected_day.year()
        ),
        Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("Mo Tu We Th Fr Sa Su"));

    match month_grid(selected_day) {
        Ok(cells) => {
            for week in 0..6 {
                let mut spans = Vec::<Span<'static>>::new();
                for day_col in 0..7 {
                    let idx = week * 7 + day_col;
                    match cells.get(idx).and_then(|cell| *cell) {
                        Some(date) => {
                            let text = format!("{:>2}", date.day());
                            let style = if date == selected_day {
                                Style::default()
                                    .fg(COLOR_VOID)
                                    .bg(COLOR_AMBER)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(COLOR_GHOST)
                            };
                            spans.push(Span::styled(text, style));
                        }
                        None => spans.push(Span::styled("  ", Style::default().fg(COLOR_STEEL))),
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
                Style::default().fg(COLOR_RED),
            )));
        }
    }

    Paragraph::new(lines)
        .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
        .block(
            Block::default()
                .title("Calendar")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_STEEL)),
        )
}

fn draw_tasks_widget(
    area: Rect,
    visible_rows: &[VisibleTodo],
    selected_index: usize,
) -> (
    Paragraph<'static>,
    Option<(Scrollbar<'static>, ScrollbarState, Rect)>,
) {
    let list_height = usize::from(area.height.saturating_sub(2));
    let (start, end) = viewport_window(visible_rows.len(), selected_index, list_height);

    let mut lines = Vec::<Line<'static>>::new();
    if visible_rows.is_empty() {
        lines.push(Line::from("no tasks for selected day"));
    } else {
        for (row_idx, row) in visible_rows.iter().enumerate().take(end).skip(start) {
            let marker = if row_idx == selected_index { ">" } else { " " };
            let bucket = if row.overdue { "O" } else { "T" };
            let style = match row.status {
                Status::Done => Style::default().fg(COLOR_GREEN),
                Status::Pending => {
                    if row.overdue {
                        Style::default().fg(COLOR_RED)
                    } else {
                        Style::default().fg(COLOR_GHOST)
                    }
                }
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", marker), Style::default().fg(COLOR_AMBER)),
                Span::styled(
                    format!("[{} {}] {}", bucket, status_symbol(row.status), row.label),
                    style,
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_STEEL)),
        );

    let scrollbar = if visible_rows.len() > list_height && list_height > 0 {
        let state = ScrollbarState::new(visible_rows.len())
            .position(start)
            .viewport_content_length(list_height);
        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(COLOR_CYAN))
            .track_style(Style::default().fg(COLOR_STEEL));
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

fn status_symbol(status: Status) -> &'static str {
    if status == Status::Done { "✓" } else { " " }
}
