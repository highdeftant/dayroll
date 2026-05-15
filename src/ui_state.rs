use chrono::NaiveDate;
use dayroll::model::{Priority, Status};

#[derive(Debug, Clone)]
pub(crate) struct VisibleTodo {
    pub(crate) id: uuid::Uuid,
    pub(crate) label: String,
    pub(crate) overdue: bool,
    pub(crate) status: Status,
    pub(crate) priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskFormField {
    Title,
    Priority,
    Date,
    Description,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskFormState {
    pub(crate) todo_id: Option<uuid::Uuid>,
    pub(crate) title: String,
    pub(crate) priority: Priority,
    pub(crate) date: NaiveDate,
    pub(crate) description: String, // Markdown description
    pub(crate) field: TaskFormField,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MoveDateState {
    pub(crate) todo_id: uuid::Uuid,
    pub(crate) date: NaiveDate,
}

#[derive(Debug, Clone)]
pub(crate) enum ModalState {
    None,
    TaskForm(TaskFormState),
    MoveDate(MoveDateState),
}
