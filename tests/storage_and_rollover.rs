use chrono::NaiveDate;

use dayroll::app::DayBuckets;
use dayroll::model::{Priority, Status, Todo};
use dayroll::storage::{Store, TodoStore};

#[test]
fn overdue_bucket_excludes_completed_items() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid date");

    let mut completed = Todo::new(
        "done yesterday",
        Priority::Medium,
        NaiveDate::from_ymd_opt(2026, 4, 17).expect("valid"),
    );
    completed.status = Status::Done;

    let pending = Todo::new(
        "still overdue",
        Priority::High,
        NaiveDate::from_ymd_opt(2026, 4, 16).expect("valid"),
    );

    let buckets = DayBuckets::for_day(today, &[completed, pending.clone()]);

    assert_eq!(buckets.overdue.len(), 1);
    assert_eq!(buckets.overdue[0].title, pending.title);
}

#[test]
fn storage_round_trip_preserves_todos() {
    let store = Store::new_in_memory();
    let seed = vec![
        Todo::new(
            "task one",
            Priority::Low,
            NaiveDate::from_ymd_opt(2026, 4, 18).expect("valid"),
        ),
        Todo::new(
            "task two",
            Priority::High,
            NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid"),
        ),
    ];

    store.save(&seed).expect("save should succeed");
    let loaded = store.load().expect("load should succeed");

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].title, "task one");
    assert_eq!(loaded[1].title, "task two");
}
