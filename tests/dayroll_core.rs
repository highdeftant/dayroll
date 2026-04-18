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
