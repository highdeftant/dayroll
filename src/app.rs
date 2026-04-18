use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::{Priority, Status, Todo};

#[derive(Debug, Clone)]
pub struct AppState {
    selected_day: NaiveDate,
    todos: Vec<Todo>,
}

#[derive(Debug, Clone)]
pub struct DayBuckets {
    pub overdue: Vec<Todo>,
    pub today: Vec<Todo>,
}

impl AppState {
    pub fn new_for_date(day: NaiveDate) -> Self {
        Self {
            selected_day: day,
            todos: Vec::new(),
        }
    }

    pub fn with_todos(day: NaiveDate, todos: Vec<Todo>) -> Self {
        Self {
            selected_day: day,
            todos,
        }
    }

    pub fn selected_day(&self) -> NaiveDate {
        self.selected_day
    }

    pub fn add_todo(
        &mut self,
        title: impl Into<String>,
        priority: Priority,
        assigned_day: NaiveDate,
    ) -> Uuid {
        let todo = Todo::new(title, priority, assigned_day);
        let id = todo.id;
        self.todos.push(todo);
        id
    }

    pub fn move_todo(&mut self, id: Uuid, target_day: NaiveDate) -> Result<(), String> {
        match self.todos.iter_mut().find(|todo| todo.id == id) {
            Some(todo) => {
                todo.assigned_day = target_day;
                Ok(())
            }
            None => Err("todo not found".to_string()),
        }
    }

    pub fn toggle_done(&mut self, id: Uuid) -> Result<(), String> {
        match self.todos.iter_mut().find(|todo| todo.id == id) {
            Some(todo) => {
                if todo.status == Status::Done {
                    todo.status = Status::Pending;
                    todo.completed_at = None;
                } else {
                    todo.status = Status::Done;
                    todo.completed_at = Some(chrono::Utc::now());
                }
                Ok(())
            }
            None => Err("todo not found".to_string()),
        }
    }

    pub fn select_next_day(&mut self) {
        if let Some(next) = self.selected_day.succ_opt() {
            self.selected_day = next;
        }
    }

    pub fn select_prev_day(&mut self) {
        if let Some(prev) = self.selected_day.pred_opt() {
            self.selected_day = prev;
        }
    }

    pub fn todos(&self) -> &[Todo] {
        &self.todos
    }

    pub fn todo(&self, id: Uuid) -> Option<&Todo> {
        self.todos.iter().find(|todo| todo.id == id)
    }
}

impl DayBuckets {
    pub fn for_day(day: NaiveDate, todos: &[Todo]) -> Self {
        let overdue = todos
            .iter()
            .filter(|todo| todo.status == Status::Pending && todo.assigned_day < day)
            .cloned()
            .collect::<Vec<_>>();

        let today = todos
            .iter()
            .filter(|todo| todo.assigned_day == day)
            .cloned()
            .collect::<Vec<_>>();

        Self { overdue, today }
    }
}
