use std::error::Error;
use std::io;
use std::time::Duration;

use chrono::Local;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use dayroll::app::{AppState, DayBuckets};
use dayroll::model::Priority;
use dayroll::storage::{Store, TodoStore};
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const COLOR_VOID: Color = Color::Rgb(24, 28, 34);
const COLOR_STEEL: Color = Color::Rgb(72, 82, 96);
const COLOR_GHOST: Color = Color::Rgb(214, 221, 230);
const COLOR_GREEN: Color = Color::Rgb(134, 239, 172);
const COLOR_AMBER: Color = Color::Rgb(252, 211, 77);
const COLOR_RED: Color = Color::Rgb(248, 113, 113);

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
        let visible = visible_todo_ids(&app);
        if selected_index >= visible.len() && !visible.is_empty() {
            selected_index = visible.len().saturating_sub(1);
        }
        if visible.is_empty() {
            selected_index = 0;
        }

        terminal
            .draw(|frame| draw_ui(frame, &app, selected_index))
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
            KeyCode::Char('t') => {
                let now = Local::now().date_naive();
                app = AppState::with_todos(now, app.todos().to_vec());
                selected_index = 0;
            }
            KeyCode::Char('a') => {
                let title = format!("New task {}", Local::now().format("%H:%M:%S"));
                app.add_todo(title, Priority::Medium, app.selected_day());
                store.save(app.todos())?;
            }
            KeyCode::Char('m') => {
                if let Some(todo_id) = visible.get(selected_index) {
                    if let Some(todo) = app.todo(*todo_id) {
                        let next_day = match todo.assigned_day.succ_opt() {
                            Some(day) => day,
                            None => todo.assigned_day,
                        };
                        app.move_todo(*todo_id, next_day)?;
                        store.save(app.todos())?;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(todo_id) = visible.get(selected_index) {
                    app.toggle_done(*todo_id)?;
                    store.save(app.todos())?;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if selected_index + 1 < visible.len() {
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

fn visible_todo_ids(app: &AppState) -> Vec<uuid::Uuid> {
    let buckets = DayBuckets::for_day(app.selected_day(), app.todos());
    let mut ids = Vec::new();
    for todo in buckets.overdue {
        ids.push(todo.id);
    }
    for todo in buckets.today {
        ids.push(todo.id);
    }
    ids
}

fn draw_ui(frame: &mut ratatui::Frame<'_>, app: &AppState, selected_index: usize) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let buckets = DayBuckets::for_day(app.selected_day(), app.todos());

    let pending_count = app
        .todos()
        .iter()
        .filter(|todo| todo.status == dayroll::model::Status::Pending)
        .count();
    let done_count = app
        .todos()
        .iter()
        .filter(|todo| todo.status == dayroll::model::Status::Done)
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

    let mut lines = Vec::<Line<'_>>::new();
    lines.push(Line::from(Span::styled(
        "OVERDUE",
        Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
    )));

    let mut row = 0usize;
    for todo in &buckets.overdue {
        let selected = row == selected_index;
        let marker = if selected { ">" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", marker), Style::default().fg(COLOR_AMBER)),
            Span::raw(format!(
                "[{}] {} ({})",
                status_symbol(todo.status),
                todo.title,
                todo.assigned_day
            )),
        ]));
        row += 1;
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "TODAY",
        Style::default()
            .fg(COLOR_GREEN)
            .add_modifier(Modifier::BOLD),
    )));

    for todo in &buckets.today {
        let selected = row == selected_index;
        let marker = if selected { ">" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", marker), Style::default().fg(COLOR_AMBER)),
            Span::raw(format!("[{}] {}", status_symbol(todo.status), todo.title)),
        ]));
        row += 1;
    }

    if row == 0 {
        lines.push(Line::from("  no tasks yet for this day"));
    }

    let list = Paragraph::new(lines)
        .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
        .block(
            Block::default()
                .title("Calendar Day View")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_STEEL)),
        );

    let status = Paragraph::new(
        "[a] add [space/enter] toggle done [m] move +1 day [[/]] day nav [t] today [q] quit",
    )
    .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_STEEL)),
    );

    frame.render_widget(title, layout[0]);
    frame.render_widget(list, layout[1]);
    frame.render_widget(status, layout[2]);
}

fn status_symbol(status: dayroll::model::Status) -> &'static str {
    if status == dayroll::model::Status::Done {
        "✓"
    } else {
        " "
    }
}
