use chrono::{Days, NaiveDate};
use crossterm::event::KeyCode;
use dayroll::app::{AppState, UndoSlot, parse_quick_add, shift_month_date};
use dayroll::model::Priority;
use dayroll::storage::{Store, TodoStore};

use crate::ui_state::{DescriptionEditorState, ModalState, TaskFormField};

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
        ModalState::DescriptionEditor(state) => {
            match key {
                KeyCode::Esc => *modal = ModalState::TaskForm(state.parent.clone()),
                KeyCode::F(2) => {
                    let mut parent = state.parent.clone();
                    parent.description = state.draft.clone();
                    parent.field = TaskFormField::Description;
                    parent.error = None;
                    *modal = ModalState::TaskForm(parent);
                }
                KeyCode::Enter => state.draft.push('\n'),
                KeyCode::Backspace => {
                    state.draft.pop();
                }
                KeyCode::Tab => state.draft.push_str("    "),
                KeyCode::Char(c) if !c.is_control() => state.draft.push(c),
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
                KeyCode::Enter if form.field == TaskFormField::Description => {
                    let parent = form.clone();
                    *modal = ModalState::DescriptionEditor(DescriptionEditorState {
                        draft: parent.description.clone(),
                        parent,
                    });
                }
                KeyCode::Enter => {
                    let title = form.title.trim().to_string();
                    if title.is_empty() {
                        form.error = Some("title cannot be empty".to_string());
                        return Ok(());
                    }

                    if let Some(id) = form.todo_id {
                        let parsed = match parse_quick_add(&title, form.priority, form.date) {
                            Ok(parsed) => parsed,
                            Err(error) => {
                                form.error = Some(error);
                                return Ok(());
                            }
                        };
                        let description = if form.description.trim().is_empty() {
                            None
                        } else {
                            Some(form.description.clone())
                        };
                        app.update_todo_with_description(
                            id,
                            parsed.title,
                            parsed.priority,
                            parsed.assigned_day,
                            description,
                            parsed.due_time,
                        )?;
                    } else {
                        let parsed = match parse_quick_add(&title, form.priority, form.date) {
                            Ok(parsed) => parsed,
                            Err(error) => {
                                form.error = Some(error);
                                return Ok(());
                            }
                        };
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
                            parsed.due_time,
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
    use dayroll::app::{AppState, UndoSlot};
    use dayroll::model::Priority;
    use dayroll::storage::Store;

    use crate::ui_state::{DescriptionEditorState, ModalState, TaskFormField, TaskFormState};

    use super::{handle_modal_event, handle_search_key};

    fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
    }

    fn task_form(day: NaiveDate) -> TaskFormState {
        TaskFormState {
            todo_id: None,
            title: "draft title".to_string(),
            priority: Priority::Medium,
            date: day,
            description: String::new(),
            field: TaskFormField::Title,
            error: None,
        }
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

    #[test]
    fn invalid_quick_add_token_stays_in_form_and_sets_error() -> Result<(), String> {
        let day = date(2026, 4, 18)?;
        let store = Store::new_in_memory();
        let mut undo_slot = UndoSlot::new();
        let mut app = AppState::new_for_date(day);
        let mut form = task_form(day);
        form.title = "pay bill @2026-99-99".to_string();
        let mut modal = ModalState::TaskForm(form);

        handle_modal_event(KeyCode::Enter, &mut modal, &mut app, &store, &mut undo_slot)?;

        match modal {
            ModalState::TaskForm(form) => {
                assert_eq!(form.title, "pay bill @2026-99-99");
                assert!(form.error.is_some());
            }
            other => panic!("expected task form, got {other:?}"),
        }
        assert!(app.todos().is_empty());
        Ok(())
    }

    #[test]
    fn enter_on_description_field_opens_description_editor() -> Result<(), String> {
        let day = date(2026, 4, 18)?;
        let store = Store::new_in_memory();
        let mut undo_slot = UndoSlot::new();
        let mut app = AppState::new_for_date(day);
        let mut form = task_form(day);
        form.description = "# Notes\n- item".to_string();
        form.field = TaskFormField::Description;
        let mut modal = ModalState::TaskForm(form.clone());

        handle_modal_event(KeyCode::Enter, &mut modal, &mut app, &store, &mut undo_slot)?;

        match modal {
            ModalState::DescriptionEditor(DescriptionEditorState { parent, draft }) => {
                assert_eq!(parent.description, form.description);
                assert_eq!(draft, form.description);
            }
            other => panic!("expected description editor, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn description_editor_save_updates_parent_form() -> Result<(), String> {
        let day = date(2026, 4, 18)?;
        let store = Store::new_in_memory();
        let mut undo_slot = UndoSlot::new();
        let mut app = AppState::new_for_date(day);
        let parent = task_form(day);
        let mut modal = ModalState::DescriptionEditor(DescriptionEditorState {
            draft: "# Heading".to_string(),
            parent,
        });

        handle_modal_event(KeyCode::Enter, &mut modal, &mut app, &store, &mut undo_slot)?;
        handle_modal_event(
            KeyCode::Char('-'),
            &mut modal,
            &mut app,
            &store,
            &mut undo_slot,
        )?;
        handle_modal_event(KeyCode::F(2), &mut modal, &mut app, &store, &mut undo_slot)?;

        match modal {
            ModalState::TaskForm(form) => {
                assert_eq!(form.field, TaskFormField::Description);
                assert_eq!(form.description, "# Heading\n-");
                assert!(form.error.is_none());
            }
            other => panic!("expected task form, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn description_editor_escape_restores_parent_form_without_saving() -> Result<(), String> {
        let day = date(2026, 4, 18)?;
        let store = Store::new_in_memory();
        let mut undo_slot = UndoSlot::new();
        let mut app = AppState::new_for_date(day);
        let mut parent = task_form(day);
        parent.description = "kept".to_string();
        parent.field = TaskFormField::Description;
        let mut modal = ModalState::DescriptionEditor(DescriptionEditorState {
            draft: "changed".to_string(),
            parent,
        });

        handle_modal_event(KeyCode::Esc, &mut modal, &mut app, &store, &mut undo_slot)?;

        match modal {
            ModalState::TaskForm(form) => {
                assert_eq!(form.field, TaskFormField::Description);
                assert_eq!(form.description, "kept");
            }
            other => panic!("expected task form, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn enter_on_title_field_saves_task_instead_of_opening_notes() -> Result<(), String> {
        let day = date(2026, 4, 18)?;
        let store = Store::new_in_memory();
        let mut undo_slot = UndoSlot::new();
        let mut app = AppState::new_for_date(day);
        let mut form = task_form(day);
        form.title = "pay rent".to_string();
        form.description = "# note".to_string();
        form.field = TaskFormField::Title;
        let mut modal = ModalState::TaskForm(form);

        handle_modal_event(KeyCode::Enter, &mut modal, &mut app, &store, &mut undo_slot)?;

        assert!(matches!(modal, ModalState::None));
        assert_eq!(app.todos().len(), 1);
        let todo = &app.todos()[0];
        assert_eq!(todo.title, "pay rent");
        assert_eq!(todo.description.as_deref(), Some("# note"));
        Ok(())
    }
}
