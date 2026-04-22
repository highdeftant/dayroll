use chrono::NaiveDate;

use dayroll::app::{AppState, DayBuckets};
use dayroll::model::{Priority, Todo};

fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
}

#[test]
fn filter_by_query_empty_returns_all() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let overdue = Todo::new("missed task", Priority::High, date(2026, 4, 17)?);
    let due_today = Todo::new("today task", Priority::Medium, today);

    let buckets = DayBuckets::for_day(today, &[overdue.clone(), due_today.clone()]);
    let filtered = buckets.filter_by_query("");

    assert_eq!(filtered.overdue.len(), 1);
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.overdue[0].title, overdue.title);
    assert_eq!(filtered.today[0].title, due_today.title);
    Ok(())
}

#[test]
fn filter_by_query_matches_substring() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let overdue = Todo::new("finish report", Priority::High, date(2026, 4, 17)?);
    let due_today = Todo::new("today task", Priority::Medium, today);
    let another = Todo::new("write notes", Priority::Low, today);

    let buckets = DayBuckets::for_day(
        today,
        &[overdue.clone(), due_today.clone(), another.clone()],
    );

    // Filter for "task" - should match only due_today
    let filtered = buckets.filter_by_query("task");
    assert_eq!(filtered.overdue.len(), 0); // "finish report" doesn't contain "task"
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.today[0].title, due_today.title);

    // Filter for "report" - should match only overdue
    let filtered = buckets.filter_by_query("report");
    assert_eq!(filtered.overdue.len(), 1);
    assert_eq!(filtered.overdue[0].title, overdue.title);
    assert_eq!(filtered.today.len(), 0);

    // Filter for "notes" - should match only another
    let filtered = buckets.filter_by_query("notes");
    assert_eq!(filtered.overdue.len(), 0);
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.today[0].title, another.title);
    Ok(())
}

#[test]
fn filter_by_query_case_insensitive() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let task = Todo::new("URGENT Meeting", Priority::High, today);

    let buckets = DayBuckets::for_day(today, std::slice::from_ref(&task));

    // Filter with lowercase should match uppercase in title
    let filtered = buckets.filter_by_query("meet");
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.today[0].title, task.title);

    // Filter with uppercase should also match
    let filtered = buckets.filter_by_query("URGENT");
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.today[0].title, task.title);
    Ok(())
}

#[test]
fn filter_by_query_no_matches_returns_empty() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let task = Todo::new("today task", Priority::Medium, today);

    let buckets = DayBuckets::for_day(today, &[task]);

    // Filter for non-existent word
    let filtered = buckets.filter_by_query("xyznotfound123");
    assert_eq!(filtered.overdue.len(), 0);
    assert_eq!(filtered.today.len(), 0);
    Ok(())
}

#[test]
fn filter_preserves_overdue_today_separation() -> Result<(), String> {
    let today = date(2026, 4, 18)?;
    let overdue = Todo::new("past task", Priority::High, date(2026, 4, 17)?);
    let due_today = Todo::new("future task", Priority::Medium, today);

    let buckets = DayBuckets::for_day(today, &[overdue.clone(), due_today.clone()]);

    // Filter for word that appears in both titles
    let filtered = buckets.filter_by_query("task");
    assert_eq!(filtered.overdue.len(), 1);
    assert_eq!(filtered.today.len(), 1);
    assert_eq!(filtered.overdue[0].title, overdue.title);
    assert_eq!(filtered.today[0].title, due_today.title);
    Ok(())
}

#[test]
fn app_search_query_accessors() {
    let today = date(2026, 4, 18).unwrap();
    let mut state = AppState::new_for_date(today);

    // Initial query should be empty
    assert!(state.search_query().is_empty());

    // Set a search query
    state.set_search_query("filter text".to_string());
    assert_eq!(state.search_query(), "filter text");

    // Clear the search query
    state.set_search_query(String::new());
    assert!(state.search_query().is_empty());
}
