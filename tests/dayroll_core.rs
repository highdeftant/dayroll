use chrono::NaiveDate;

use dayroll::app::{AppState, DayBuckets};
use dayroll::model::{Priority, Todo};

fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
}

#[test]
fn opens_on_today() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let state = AppState::new_for_date(today);
    assert_eq!(state.selected_day(), today);
    Ok(())
}

#[test]
fn day_view_separates_overdue_and_today() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let overdue = Todo::new("missed", Priority::High, date(2026, 4, 17)?);
    let due_today = Todo::new("today", Priority::Medium, today);

    let buckets = DayBuckets::for_day(today, &[overdue.clone(), due_today.clone()]);

    assert_eq!(buckets.overdue.len(), 1);
    assert_eq!(buckets.today.len(), 1);
    assert_eq!(buckets.overdue[0].title, overdue.title);
    assert_eq!(buckets.today[0].title, due_today.title);
    Ok(())
}

#[test]
fn move_todo_to_another_day_changes_assigned_day() -> Result<(), String> {
    let original = date(2026, 4, 18)?;
    let target = date(2026, 4, 22)?;

    let mut state = AppState::new_for_date(original);
    let id = state.add_todo("reschedule", Priority::Low, original);

    state.move_todo(id, target)?;

    let moved = state
        .todo(id)
        .ok_or_else(|| "todo not found after move".to_string())?;
    assert_eq!(moved.assigned_day, target);
    Ok(())
}

#[test]
fn toggling_done_sets_completed_at() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(today);
    let id = state.add_todo("complete me", Priority::Medium, today);

    state.toggle_done(id)?;

    let todo = state
        .todo(id)
        .ok_or_else(|| "todo missing after toggle".to_string())?;
    assert!(todo.completed_at.is_some());
    Ok(())
}

#[test]
fn day_navigation_moves_selection() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(today);

    state.select_next_day();
    assert_eq!(state.selected_day(), date(2026, 4, 19)?);

    state.select_prev_day();
    assert_eq!(state.selected_day(), today);
    Ok(())
}

#[test]
fn month_navigation_clamps_to_month_length() -> Result<(), String> {
    let jan_31 = date(2026, 1, 31)?;
    let mut state = AppState::new_for_date(jan_31);

    state.select_next_month();
    assert_eq!(state.selected_day(), date(2026, 2, 28)?);

    state.select_prev_month();
    assert_eq!(state.selected_day(), date(2026, 1, 28)?);
    Ok(())
}

#[test]
fn month_grid_has_42_cells_and_contains_selected_day() -> Result<(), String> {
    let selected = date(2026, 4, 18)?;
    let grid = dayroll::app::month_grid(selected)?;

    assert_eq!(grid.len(), 42);
    assert!(grid.iter().flatten().any(|d| *d == selected));
    Ok(())
}

#[test]
fn viewport_window_keeps_selection_visible() {
    let (start, end) = dayroll::app::viewport_window(40, 22, 10);
    assert!(22 >= start && 22 < end);
    assert_eq!(end.saturating_sub(start), 10);
}

#[test]
fn delete_removes_todo_by_id() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);
    let id = state.add_todo("remove me", Priority::Low, day);
    assert_eq!(state.todos().len(), 1);

    state.delete_todo(id)?;
    assert_eq!(state.todos().len(), 0);
    assert!(state.todo(id).is_none());
    Ok(())
}

#[test]
fn delete_missing_id_returns_error() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);

    let err = state.delete_todo(uuid::Uuid::nil());
    assert!(err.is_err());
    Ok(())
}

#[test]
fn edit_updates_title_priority_and_date() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);
    let id = state.add_todo("old", Priority::Low, day);
    let target = date(2026, 4, 23)?;

    state.update_todo(id, "new".to_string(), Priority::High, target)?;

    let todo = state
        .todo(id)
        .ok_or_else(|| "todo missing after edit".to_string())?;
    assert_eq!(todo.title, "new");
    assert_eq!(todo.priority, Priority::High);
    assert_eq!(todo.assigned_day, target);
    Ok(())
}

#[test]
fn shift_month_date_clamps() -> Result<(), String> {
    let jan_31 = date(2026, 1, 31)?;
    let feb = dayroll::app::shift_month_date(jan_31, 1)?;
    assert_eq!(feb, date(2026, 2, 28)?);
    Ok(())
}

#[test]
fn help_overlay_toggles_on_and_off() {
    let shown = dayroll::app::toggle_help_overlay(dayroll::app::Overlay::None);
    assert_eq!(shown, dayroll::app::Overlay::Help);

    let hidden = dayroll::app::toggle_help_overlay(shown);
    assert_eq!(hidden, dayroll::app::Overlay::None);
}

#[test]
fn quit_request_opens_confirmation_from_normal_mode() {
    let overlay = dayroll::app::request_quit_overlay(dayroll::app::Overlay::None);
    assert_eq!(overlay, dayroll::app::Overlay::QuitConfirm);
}

#[test]
fn footer_hint_is_short_in_normal_mode() {
    let hint = dayroll::app::footer_hint(dayroll::app::Overlay::None, false);
    assert!(hint.contains("[?] help"));
    assert!(!hint.contains("delete"));
}

#[test]
fn search_mode_starts_inactive_and_can_be_cleared() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);
    assert!(!state.search_active());

    state.activate_search();
    assert!(state.search_active());

    state.append_search_char('r');
    state.append_search_char('e');
    assert_eq!(state.search_query(), "re");

    state.clear_search();
    assert!(!state.search_active());
    assert!(state.search_query().is_empty());
    Ok(())
}

#[test]
fn active_search_footer_hint_explains_escape() {
    let hint = dayroll::app::footer_hint(dayroll::app::Overlay::None, true);
    assert!(hint.contains("search"));
    assert!(hint.contains("Esc"));
}

#[test]
fn undo_restore_after_delete_reinserts_task() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);
    let first_id = state.add_todo("first", Priority::Low, day);
    let second_id = state.add_todo("second", Priority::High, day);

    let undo = state.delete_todo_with_undo(first_id)?;
    assert!(state.todo(first_id).is_none());

    state.apply_undo(undo)?;
    assert_eq!(state.todos().len(), 2);
    assert_eq!(state.todos()[0].id, first_id);
    assert_eq!(state.todos()[1].id, second_id);
    Ok(())
}

#[test]
fn undo_restore_after_move_returns_original_day() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let target = date(2026, 4, 22)?;
    let mut state = AppState::new_for_date(day);
    let id = state.add_todo("move me", Priority::Medium, day);

    let undo = state.move_todo_with_undo(id, target)?;
    assert_eq!(
        state
            .todo(id)
            .ok_or_else(|| "missing moved todo".to_string())?
            .assigned_day,
        target
    );

    state.apply_undo(undo)?;
    assert_eq!(
        state
            .todo(id)
            .ok_or_else(|| "missing restored todo".to_string())?
            .assigned_day,
        day
    );
    Ok(())
}

#[test]
fn undo_restore_after_toggle_returns_previous_status() -> Result<(), String> {
    let day = date(2026, 4, 18)?;
    let mut state = AppState::new_for_date(day);
    let id = state.add_todo("toggle me", Priority::Medium, day);

    let undo = state.toggle_done_with_undo(id)?;
    assert!(
        state
            .todo(id)
            .ok_or_else(|| "missing toggled todo".to_string())?
            .completed_at
            .is_some()
    );

    state.apply_undo(undo)?;
    let todo = state
        .todo(id)
        .ok_or_else(|| "missing restored todo".to_string())?;
    assert!(todo.completed_at.is_none());
    assert_eq!(todo.status, dayroll::model::Status::Pending);
    Ok(())
}

#[test]
fn quick_add_parses_priority_and_relative_day_tokens() -> Result<(), String> {
    let default_day = date(2026, 4, 18)?;
    let parsed =
        dayroll::app::parse_quick_add("pay rent @tomorrow !high", Priority::Low, default_day)?;
    assert_eq!(parsed.title, "pay rent");
    assert_eq!(parsed.priority, Priority::High);
    assert_eq!(parsed.assigned_day, date(2026, 4, 19)?);
    Ok(())
}

#[test]
fn quick_add_parses_iso_date_token() -> Result<(), String> {
    let default_day = date(2026, 4, 18)?;
    let parsed =
        dayroll::app::parse_quick_add("oil change @2026-05-01", Priority::Medium, default_day)?;
    assert_eq!(parsed.title, "oil change");
    assert_eq!(parsed.priority, Priority::Medium);
    assert_eq!(parsed.assigned_day, date(2026, 5, 1)?);
    Ok(())
}

#[test]
fn quick_add_rejects_bad_date_token() -> Result<(), String> {
    let default_day = date(2026, 4, 18)?;
    let err = dayroll::app::parse_quick_add("pay bill @2026-99-99", Priority::Medium, default_day);
    assert!(err.is_err());
    Ok(())
}
