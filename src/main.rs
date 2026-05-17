mod event_handler;
mod markdown;
mod render;
mod ui_state;

use std::error::Error;
use std::io;
use std::time::Duration;

use chrono::Local;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use dayroll::app::{AppState, Overlay, UndoSlot, request_quit_overlay, toggle_help_overlay};
use dayroll::model::Priority;
use dayroll::storage::{Store, TodoStore};
use dayroll::theme::{AppConfig, load_config, save_config};
use ratatui::Terminal;

use crate::event_handler::{handle_modal_event, handle_search_key};
use crate::render::{draw_ui, visible_todos};
use crate::ui_state::{ModalState, MoveDateState, TaskFormField, TaskFormState, UiViewState};

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
    let mut config = load_config().unwrap_or_else(|_| AppConfig::default());
    let mut app = AppState::with_todos(today, todos);
    let mut selected_index = 0usize;
    let mut expanded_task: Option<uuid::Uuid> = None;
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
                .draw(|frame| {
                    draw_ui(
                        frame,
                        &app,
                        &visible_rows,
                        UiViewState {
                            selected_index,
                            expanded_task,
                            theme_name: config.theme,
                            overlay,
                        },
                        &modal,
                    )
                })
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
                        description: String::new(),
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
                            description: todo.description.clone().unwrap_or_default(),
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
                KeyCode::Char('l') => {
                    expanded_task = visible_rows.get(selected_index).and_then(|row| {
                        if row.description.is_some() {
                            Some(row.id)
                        } else {
                            None
                        }
                    });
                }
                KeyCode::Char('h') => {
                    if visible_rows.get(selected_index).map(|row| row.id) == expanded_task {
                        expanded_task = None;
                    }
                }
                KeyCode::Char('T') => {
                    config.theme = config.theme.next();
                    save_config(&config)?;
                }
                KeyCode::Char('Y') => {
                    config.theme = config.theme.previous();
                    save_config(&config)?;
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
