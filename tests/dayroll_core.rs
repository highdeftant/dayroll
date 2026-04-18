use chrono::NaiveDate;

use dayroll::app::{AppState, DayBuckets};
use dayroll::model::{Priority, Todo};

#[test]
fn opens_on_today() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");
    let state = AppState::new_for_date(today);
    assert_eq!(state.selected_day(), today);
}

#[test]
fn day_view_separates_overdue_and_today() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");
    let overdue = Todo::new(
        "missed",
        Priority::High,
        NaiveDate::from_ymd_opt(2026, 4, 17).expect("valid"),
    );
    let due_today = Todo::new("today", Priority::Medium, today);

    let buckets = DayBuckets::for_day(today, &[overdue.clone(), due_today.clone()]);

    assert_eq!(buckets.overdue.len(), 1);
    assert_eq!(buckets.today.len(), 1);
    assert_eq!(buckets.overdue[0].title, overdue.title);
    assert_eq!(buckets.today[0].title, due_today.title);
}

#[test]
fn move_todo_to_another_day_changes_assigned_day() {
    let original = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");
    let target = NaiveDate::from_ymd_opt(2026, 4, 22).expect("valid date");

    let mut state = AppState::new_for_date(original);
    let id = state.add_todo("reschedule", Priority::Low, original);

    state.move_todo(id, target).expect("move should succeed");

    let moved = state.todo(id).expect("todo exists");
    assert_eq!(moved.assigned_day, target);
}

#[test]
fn toggling_done_sets_completed_at() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");
    let mut state = AppState::new_for_date(today);
    let id = state.add_todo("complete me", Priority::Medium, today);

    state.toggle_done(id).expect("toggle should succeed");

    let todo = state.todo(id).expect("todo exists");
    assert!(todo.completed_at.is_some());
}

#[test]
fn day_navigation_moves_selection() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");
    let mut state = AppState::new_for_date(today);

    state.select_next_day();
    assert_eq!(
        state.selected_day(),
        NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid")
    );

    state.select_prev_day();
    assert_eq!(state.selected_day(), today);
}
