use std::error::Error;
use std::io;
use std::time::Duration;

use chrono::{Datelike, Days, Local, NaiveDate};
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use dayroll::app::{
    AppState, DayBuckets, Overlay, UndoSlot, footer_hint, month_grid, parse_quick_add,
    request_quit_overlay, shift_month_date, toggle_help_overlay, viewport_window,
};
use dayroll::model::{Priority, Status};
use dayroll::storage::{Store, TodoStore};
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskFormField {
    Title,
    Priority,
    Date,
}

#[derive(Debug, Clone)]
struct TaskFormState {
    todo_id: Option<uuid::Uuid>,
    title: String,
    priority: Priority,
    date: NaiveDate,
    field: TaskFormField,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct MoveDateState {
    todo_id: uuid::Uuid,
    date: NaiveDate,
}

#[derive(Debug, Clone)]
enum ModalState {
    None,
    TaskForm(TaskFormState),
    MoveDate(MoveDateState),
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
    let mut modal = ModalState::None;
    let mut overlay = Overlay::None;
    let mut undo_slot = UndoSlot::new();

    enable_raw_mode().map_err(|error| format!("failed to enable raw mode: {error}"))?;
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)
        .map_err(|error| format!("failed to enter alt screen: {error}"))?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|error| format!("terminal init failed: {error}"))?;

    let run_result = (|| -> Result<(), String> {
        loop {
            let visible_rows = visible_todos(&app);
            if selected_index >= visible_rows.len() && !visible_rows.is_empty() {
                selected_index = visible_rows.len().saturating_sub(1);
            }
            if visible_rows.is_empty() {
                selected_index = 0;
            }

            terminal
                .draw(|frame| draw_ui(frame, &app, &visible_rows, selected_index, &modal, overlay))
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

            if !matches!(modal, ModalState::None) {
                handle_modal_event(key_event.code, &mut modal, &mut app, &store, &mut undo_slot)?;
                continue;
            }

            if overlay != Overlay::None {
                match overlay {
                    Overlay::Help => match key_event.code {
                        KeyCode::Char('?') | KeyCode::Esc => overlay = Overlay::None,
                        KeyCode::Char('q') => overlay = request_quit_overlay(overlay),
                        _ => {}
                    },
                    Overlay::QuitConfirm => match key_event.code {
                        KeyCode::Char('y') => break,
                        KeyCode::Char('n') | KeyCode::Esc => overlay = Overlay::None,
                        _ => {}
                    },
                    Overlay::None => {}
                }
                continue;
            }

            if handle_search_key(key_event.code, &mut app) {
                continue;
            }

            match key_event.code {
                KeyCode::Char('q') => overlay = request_quit_overlay(overlay),
                KeyCode::Esc => {
                    if app.search_active() {
                        app.cancel_search();
                    } else {
                        overlay = Overlay::QuitConfirm;
                    }
                }
                KeyCode::Char('?') => overlay = toggle_help_overlay(overlay),
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
                    modal = ModalState::TaskForm(TaskFormState {
                        todo_id: None,
                        title: String::new(),
                        priority: Priority::Medium,
                        date: app.selected_day(),
                        field: TaskFormField::Title,
                        error: None,
                    });
                }
                KeyCode::Char('e') => {
                    if let Some(todo) = visible_rows
                        .get(selected_index)
                        .and_then(|row| app.todo(row.id))
                    {
                        modal = ModalState::TaskForm(TaskFormState {
                            todo_id: Some(todo.id),
                            title: todo.title.clone(),
                            priority: todo.priority,
                            date: todo.assigned_day,
                            field: TaskFormField::Title,
                            error: None,
                        });
                    }
                }
                KeyCode::Char('m') => {
                    if let Some(todo) = visible_rows
                        .get(selected_index)
                        .and_then(|row| app.todo(row.id))
                    {
                        modal = ModalState::MoveDate(MoveDateState {
                            todo_id: todo.id,
                            date: todo.assigned_day,
                        });
                    }
                }
                KeyCode::Char('u') => {
                    if let Some(undo) = undo_slot.take() {
                        app.apply_undo(undo)?;
                        store.save(app.todos())?;
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(row) = visible_rows.get(selected_index) {
                        undo_slot.record(app.delete_todo_with_undo(row.id)?);
                        store.save(app.todos())?;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(row) = visible_rows.get(selected_index) {
                        undo_slot.record(app.toggle_done_with_undo(row.id)?);
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
        Ok(())
    })();

    let cleanup_result = cleanup_terminal(&mut terminal);

    match (run_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_error), Ok(())) => Err(run_error),
        (Ok(()), Err(cleanup_error)) => Err(cleanup_error),
        (Err(run_error), Err(cleanup_error)) => Err(format!(
            "{run_error}; terminal cleanup failed: {cleanup_error}"
        )),
    }
}

fn cleanup_terminal(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<(), String> {
    disable_raw_mode().map_err(|error| format!("failed to disable raw mode: {error}"))?;
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .map_err(|error| format!("failed leaving alt screen: {error}"))?;
    Ok(())
}

fn handle_search_key(key: KeyCode, app: &mut AppState) -> bool {
    if app.search_active() {
        match key {
            KeyCode::Esc => {
                app.clear_search();
                true
            }
            KeyCode::Backspace => {
                app.pop_search_char();
                true
            }
            KeyCode::Char(c) if !c.is_control() => {
                app.append_search_char(c);
                true
            }
            _ => true,
        }
    } else if matches!(key, KeyCode::Char('/')) {
        app.activate_search();
        true
    } else {
        false
    }
}

fn handle_modal_event(
    key: KeyCode,
    modal: &mut ModalState,
    app: &mut AppState,
    store: &Store,
    undo_slot: &mut UndoSlot,
) -> Result<(), String> {
    match modal {
        ModalState::None => Ok(()),
        ModalState::MoveDate(state) => {
            match key {
                KeyCode::Esc => *modal = ModalState::None,
                KeyCode::Enter => {
                    undo_slot.record(app.move_todo_with_undo(state.todo_id, state.date)?);
                    store.save(app.todos())?;
                    *modal = ModalState::None;
                }
                KeyCode::Left => state.date = shift_days(state.date, -1),
                KeyCode::Right => state.date = shift_days(state.date, 1),
                KeyCode::Up => state.date = shift_days(state.date, -7),
                KeyCode::Down => state.date = shift_days(state.date, 7),
                KeyCode::Char('{') | KeyCode::Char('H') => {
                    if let Ok(day) = shift_month_date(state.date, -1) {
                        state.date = day;
                    }
                }
                KeyCode::Char('}') | KeyCode::Char('L') => {
                    if let Ok(day) = shift_month_date(state.date, 1) {
                        state.date = day;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        ModalState::TaskForm(form) => {
            match key {
                KeyCode::Esc => *modal = ModalState::None,
                KeyCode::Tab => {
                    form.field = next_field(form.field);
                    form.error = None;
                }
                KeyCode::BackTab => {
                    form.field = prev_field(form.field);
                    form.error = None;
                }
                KeyCode::Enter => {
                    let title = form.title.trim().to_string();
                    if title.is_empty() {
                        form.error = Some("title cannot be empty".to_string());
                        return Ok(());
                    }

                    if let Some(id) = form.todo_id {
                        app.update_todo(id, title, form.priority, form.date)?;
                    } else {
                        let parsed = parse_quick_add(&title, form.priority, form.date)?;
                        app.add_todo(parsed.title, parsed.priority, parsed.assigned_day);
                    }

                    undo_slot.clear();
                    store.save(app.todos())?;
                    *modal = ModalState::None;
                }
                KeyCode::Backspace => {
                    if form.field == TaskFormField::Title {
                        form.title.pop();
                    }
                }
                KeyCode::Left if form.field == TaskFormField::Priority => {
                    form.priority = prev_priority(form.priority);
                }
                KeyCode::Right if form.field == TaskFormField::Priority => {
                    form.priority = next_priority(form.priority);
                }
                KeyCode::Left if form.field == TaskFormField::Date => {
                    form.date = shift_days(form.date, -1);
                }
                KeyCode::Right if form.field == TaskFormField::Date => {
                    form.date = shift_days(form.date, 1);
                }
                KeyCode::Up if form.field == TaskFormField::Date => {
                    form.date = shift_days(form.date, -7);
                }
                KeyCode::Down if form.field == TaskFormField::Date => {
                    form.date = shift_days(form.date, 7);
                }
                KeyCode::Char('{') | KeyCode::Char('H') if form.field == TaskFormField::Date => {
                    if let Ok(day) = shift_month_date(form.date, -1) {
                        form.date = day;
                    }
                }
                KeyCode::Char('}') | KeyCode::Char('L') if form.field == TaskFormField::Date => {
                    if let Ok(day) = shift_month_date(form.date, 1) {
                        form.date = day;
                    }
                }
                KeyCode::Char(c) => {
                    if form.field == TaskFormField::Title && !c.is_control() {
                        form.title.push(c);
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }
}

fn next_field(field: TaskFormField) -> TaskFormField {
    match field {
        TaskFormField::Title => TaskFormField::Priority,
        TaskFormField::Priority => TaskFormField::Date,
        TaskFormField::Date => TaskFormField::Title,
    }
}

fn prev_field(field: TaskFormField) -> TaskFormField {
    match field {
        TaskFormField::Title => TaskFormField::Date,
        TaskFormField::Priority => TaskFormField::Title,
        TaskFormField::Date => TaskFormField::Priority,
    }
}

fn next_priority(priority: Priority) -> Priority {
    match priority {
        Priority::High => Priority::Medium,
        Priority::Medium => Priority::Low,
        Priority::Low => Priority::Low,
    }
}

fn prev_priority(priority: Priority) -> Priority {
    match priority {
        Priority::High => Priority::High,
        Priority::Medium => Priority::High,
        Priority::Low => Priority::Medium,
    }
}

fn shift_days(day: NaiveDate, delta_days: i64) -> NaiveDate {
    if delta_days >= 0 {
        let abs = match u64::try_from(delta_days) {
            Ok(value) => value,
            Err(_) => return day,
        };
        match day.checked_add_days(Days::new(abs)) {
            Some(next) => next,
            None => day,
        }
    } else {
        let abs = match u64::try_from(-delta_days) {
            Ok(value) => value,
            Err(_) => return day,
        };
        match day.checked_sub_days(Days::new(abs)) {
            Some(prev) => prev,
            None => day,
        }
    }
}

fn visible_todos(app: &AppState) -> Vec<VisibleTodo> {
    let buckets = DayBuckets::for_day(app.selected_day(), app.todos());
    let filtered_buckets = buckets.filter_by_query(app.search_query());
    let mut rows = Vec::new();

    for todo in &filtered_buckets.overdue {
        rows.push(VisibleTodo {
            id: todo.id,
            label: format!("{} ({})", todo.title, todo.assigned_day),
            overdue: true,
            status: todo.status,
        });
    }

    for todo in &filtered_buckets.today {
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
    modal: &ModalState,
    overlay: Overlay,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

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
    let filter_indicator = if app.search_active() {
        if app.search_query().is_empty() {
            " [search] ".to_string()
        } else {
            format!(" [search: {}] ", app.search_query())
        }
    } else {
        String::new()
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " DAYROLL ",
            Style::default()
                .fg(COLOR_GHOST)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                " {}  pending:{} done:{}{} ",
                app.selected_day(),
                pending_count,
                done_count,
                filter_indicator
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
    let tasks = draw_tasks_widget(
        layout[2],
        visible_rows,
        selected_index,
        app.search_active(),
        app.search_query(),
    );

    let status_hint = footer_hint(overlay, app.search_active(), app.search_query());
    let status = Paragraph::new(status_hint.0)
        .style(Style::default().fg(COLOR_GHOST).bg(COLOR_VOID))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_STEEL)),
        );

    frame.render_widget(title, layout[0]);
    frame.render_widget(calendar, layout[1]);
    frame.render_widget(tasks.0, layout[2]);

    if let Some((scrollbar, mut state, area)) = tasks.1 {
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }

    frame.render_widget(status, layout[3]);

    draw_modal(frame, modal);
    draw_overlay(frame, overlay);
}

fn draw_overlay(frame: &mut ratatui::Frame<'_>, overlay: Overlay) {
    match overlay {
        Overlay::None => {}
        Overlay::Help => {
            let area = centered_rect(72, 60, frame.area());
            frame.render_widget(Clear, area);
            let text = vec![
                Line::from("Keyboard bindings"),
                Line::from(""),
                Line::from("j/k or arrows  move selection"),
                Line::from("[/] or arrows   previous/next day"),
                Line::from("{/} or H/L      previous/next month"),
                Line::from("a              add task"),
                Line::from("e              edit selected task"),
                Line::from("m              move selected task date"),
                Line::from("d              delete selected task"),
                Line::from("Enter/Space    toggle done"),
                Line::from("t              jump to today"),
                Line::from("/              enter search"),
                Line::from("search mode    type to filter, Esc clear"),
                Line::from("u              undo last move/delete/toggle"),
                Line::from("q              quit confirmation"),
                Line::from("? or Esc       close this help"),
            ];
            let widget = Paragraph::new(text).block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_CYAN)),
            );
            frame.render_widget(widget, area);
        }
        Overlay::QuitConfirm => {
            let area = centered_rect(40, 20, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from("Quit Dayroll?"),
                Line::from(""),
                Line::from("[y] yes   [n] no   [Esc] cancel"),
            ])
            .block(
                Block::default()
                    .title("Confirm Quit")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_RED)),
            );
            frame.render_widget(widget, area);
        }
    }
}

fn draw_modal(frame: &mut ratatui::Frame<'_>, modal: &ModalState) {
    match modal {
        ModalState::None => {}
        ModalState::MoveDate(state) => {
            let area = centered_rect(60, 35, frame.area());
            frame.render_widget(Clear, area);
            let text = vec![
                Line::from("Move task date"),
                Line::from(""),
                Line::from(format!("Selected: {}", state.date)),
                Line::from("←/→ day  ↑/↓ week  {/} month"),
                Line::from("Enter apply, Esc cancel"),
            ];
            let widget = Paragraph::new(text).block(
                Block::default()
                    .title("Date Picker")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_CYAN)),
            );
            frame.render_widget(widget, area);
        }
        ModalState::TaskForm(form) => {
            let area = centered_rect(70, 45, frame.area());
            frame.render_widget(Clear, area);
            let mode = if form.todo_id.is_some() {
                "Edit Task"
            } else {
                "Add Task"
            };

            let title_style = if form.field == TaskFormField::Title {
                Style::default()
                    .fg(COLOR_AMBER)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_GHOST)
            };
            let prio_style = if form.field == TaskFormField::Priority {
                Style::default()
                    .fg(COLOR_AMBER)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_GHOST)
            };
            let date_style = if form.field == TaskFormField::Date {
                Style::default()
                    .fg(COLOR_AMBER)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_GHOST)
            };

            let mut text = vec![
                Line::from(Span::styled(
                    mode,
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(format!("Title: {}", form.title), title_style)),
                Line::from(Span::styled(
                    format!("Priority: {:?}  (←/→ change)", form.priority),
                    prio_style,
                )),
                Line::from(Span::styled(
                    format!("Date: {}  (←/→ day, ↑/↓ week, {{/}} month)", form.date),
                    date_style,
                )),
                Line::from(""),
                Line::from("Tab/Shift+Tab switch field"),
                Line::from("Quick add: @tomorrow @2026-05-01 !high"),
                Line::from("Enter save, Esc cancel"),
            ];

            if let Some(error) = &form.error {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    format!("Error: {error}"),
                    Style::default().fg(COLOR_RED),
                )));
            }

            let widget = Paragraph::new(text).block(
                Block::default()
                    .title("Task Form")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_CYAN)),
            );
            frame.render_widget(widget, area);
        }
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
    search_active: bool,
    search_query: &str,
) -> (
    Paragraph<'static>,
    Option<(Scrollbar<'static>, ScrollbarState, Rect)>,
) {
    let list_height = usize::from(area.height.saturating_sub(2));
    let (start, end) = viewport_window(visible_rows.len(), selected_index, list_height);

    let mut lines = Vec::<Line<'static>>::new();
    if visible_rows.is_empty() {
        if search_active {
            let label = if search_query.is_empty() {
                "search is active — type to filter tasks".to_string()
            } else {
                format!("no matches for search: {search_query}")
            };
            lines.push(Line::from(label));
        } else {
            lines.push(Line::from("no tasks for selected day"));
        }
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

#[cfg(test)]
mod tests {
    use super::handle_search_key;
    use chrono::NaiveDate;
    use crossterm::event::KeyCode;
    use dayroll::app::AppState;

    fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
    }

    #[test]
    fn search_mode_consumes_command_letters_as_text() -> Result<(), String> {
        let mut app = AppState::new_for_date(date(2026, 4, 18)?);
        app.activate_search();

        assert!(handle_search_key(KeyCode::Char('d'), &mut app));
        assert_eq!(app.search_query(), "d");
        Ok(())
    }

    #[test]
    fn slash_enters_search_mode() -> Result<(), String> {
        let mut app = AppState::new_for_date(date(2026, 4, 18)?);

        assert!(handle_search_key(KeyCode::Char('/'), &mut app));
        assert!(app.search_active());
        assert!(app.search_query().is_empty());
        Ok(())
    }
}
