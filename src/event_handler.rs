use chrono::{Days, NaiveDate};
use crossterm::event::KeyCode;
use dayroll::app::{AppState, UndoSlot, parse_quick_add, shift_month_date};
use dayroll::model::Priority;
use dayroll::storage::{Store, TodoStore};

use crate::ui_state::{ModalState, TaskFormField};

pub(crate) fn handle_search_key(key: KeyCode, app: &mut AppState) -> bool {
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

pub(crate) fn handle_modal_event(
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
                        let description = if form.description.trim().is_empty() {
                            None
                        } else {
                            Some(form.description.clone())
                        };
                        app.update_todo_with_description(
                            id,
                            title,
                            form.priority,
                            form.date,
                            description,
                        )?;
                    } else {
                        let parsed = parse_quick_add(&title, form.priority, form.date)?;
                        let description = if form.description.trim().is_empty() {
                            None
                        } else {
                            Some(form.description.clone())
                        };
                        app.add_todo_with_description(
                            parsed.title,
                            parsed.priority,
                            parsed.assigned_day,
                            description,
                        );
                    }

                    undo_slot.clear();
                    store.save(app.todos())?;
                    *modal = ModalState::None;
                }
                KeyCode::Backspace => match form.field {
                    TaskFormField::Title => {
                        form.title.pop();
                    }
                    TaskFormField::Description => {
                        form.description.pop();
                    }
                    _ => {}
                },
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
                KeyCode::Char(c) => match form.field {
                    TaskFormField::Title if !c.is_control() => form.title.push(c),
                    TaskFormField::Description if !c.is_control() => form.description.push(c),
                    _ => {}
                },
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
        TaskFormField::Date => TaskFormField::Description,
        TaskFormField::Description => TaskFormField::Title,
    }
}

fn prev_field(field: TaskFormField) -> TaskFormField {
    match field {
        TaskFormField::Title => TaskFormField::Description,
        TaskFormField::Priority => TaskFormField::Title,
        TaskFormField::Date => TaskFormField::Priority,
        TaskFormField::Description => TaskFormField::Date,
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use crossterm::event::KeyCode;
    use dayroll::app::AppState;

    use super::handle_search_key;

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
