use chrono::NaiveDate;

use dayroll::app::DayBuckets;
use dayroll::model::{Priority, Status, Todo};
use dayroll::storage::{Store, TodoStore};

fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
}

#[test]
fn overdue_bucket_excludes_completed_items() -> Result<(), String> {
    let today = date(2026, 4, 18)?;

    let mut completed = Todo::new("done yesterday", Priority::Medium, date(2026, 4, 17)?);
    completed.status = Status::Done;

    let pending = Todo::new("still overdue", Priority::High, date(2026, 4, 16)?);

    let buckets = DayBuckets::for_day(today, &[completed, pending.clone()]);

    assert_eq!(buckets.overdue.len(), 1);
    assert_eq!(buckets.overdue[0].title, pending.title);
    Ok(())
}

#[test]
fn storage_round_trip_preserves_todos() -> Result<(), String> {
    let store = Store::new_in_memory();
    let seed = vec![
        Todo::new("task one", Priority::Low, date(2026, 4, 18)?),
        Todo::new("task two", Priority::High, date(2026, 4, 19)?),
    ];

    store.save(&seed)?;
    let loaded = store.load()?;

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].title, "task one");
    assert_eq!(loaded[1].title, "task two");
    Ok(())
}
