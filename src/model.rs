use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub assigned_day: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub description: Option<String>, // Markdown description
    pub due_time: Option<NaiveTime>,
}

impl Todo {
    pub fn new(title: impl Into<String>, priority: Priority, assigned_day: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            status: Status::Pending,
            priority,
            assigned_day,
            created_at: Utc::now(),
            completed_at: None,
            description: None,
            due_time: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        let desc = desc.into();
        if desc.trim().is_empty() {
            self.description = None;
        } else {
            self.description = Some(desc);
        }
        self
    }

    pub fn with_due_time(mut self, due_time: Option<NaiveTime>) -> Self {
        self.due_time = due_time;
        self
    }
}
