use std::collections::VecDeque;

use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::model::{Priority, Status, Todo};

mod calendar;
mod quick_add;

pub use calendar::{month_grid, shift_month_date, viewport_window};
pub use quick_add::{QuickAddParsed, parse_quick_add};

#[derive(Debug, Clone)]
pub struct AppState {
    selected_day: NaiveDate,
    todos: Vec<Todo>,
    search_query: String,
    search_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Help,
    QuitConfirm,
}

#[derive(Debug, Clone)]
pub struct DayBuckets {
    pub overdue: Vec<Todo>,
    pub today: Vec<Todo>,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    Delete {
        todo: Todo,
        index: usize,
    },
    Move {
        id: Uuid,
        previous_day: NaiveDate,
    },
    Toggle {
        id: Uuid,
        previous_status: Status,
        previous_completed_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    SetStatus {
        id: Uuid,
        previous_status: Status,
        previous_completed_at: Option<chrono::DateTime<chrono::Utc>>,
    },
}

#[derive(Debug, Clone)]
pub struct UndoSlot {
    pending: VecDeque<UndoAction>,
    capacity: usize,
}

impl Default for UndoSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoSlot {
    pub const DEFAULT_CAPACITY: usize = 32;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn record(&mut self, action: UndoAction) {
        self.pending.push_back(action);
        while self.pending.len() > self.capacity {
            self.pending.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn take(&mut self) -> Option<UndoAction> {
        self.pending.pop_back()
    }
}

impl AppState {
    pub fn new_for_date(day: NaiveDate) -> Self {
        Self {
            selected_day: day,
            todos: Vec::new(),
            search_query: String::new(),
            search_active: false,
        }
    }

    pub fn with_todos(day: NaiveDate, todos: Vec<Todo>) -> Self {
        Self {
            selected_day: day,
            todos,
            search_query: String::new(),
            search_active: false,
        }
    }

    pub fn selected_day(&self) -> NaiveDate {
        self.selected_day
    }

    pub fn set_selected_day(&mut self, day: NaiveDate) {
        self.selected_day = day;
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn search_active(&self) -> bool {
        self.search_active
    }

    pub fn activate_search(&mut self) {
        self.search_active = true;
    }

    pub fn append_search_char(&mut self, character: char) {
        self.search_active = true;
        self.search_query.push(character);
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.search_active = !self.search_query.is_empty();
    }

    pub fn cancel_search(&mut self) {
        self.clear_search();
    }

    pub fn add_todo(
        &mut self,
        title: impl Into<String>,
        priority: Priority,
        assigned_day: NaiveDate,
    ) -> Uuid {
        self.add_todo_with_description(title, priority, assigned_day, None, None)
    }

    pub fn add_todo_with_description(
        &mut self,
        title: impl Into<String>,
        priority: Priority,
        assigned_day: NaiveDate,
        description: Option<String>,
        due_time: Option<NaiveTime>,
    ) -> Uuid {
        let mut todo = Todo::new(title, priority, assigned_day).with_due_time(due_time);
        if let Some(description_text) = description {
            todo = todo.with_description(description_text);
        }
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

    pub fn move_todo_with_undo(
        &mut self,
        id: Uuid,
        target_day: NaiveDate,
    ) -> Result<UndoAction, String> {
        let previous_day = self
            .todos
            .iter()
            .find(|todo| todo.id == id)
            .map(|todo| todo.assigned_day)
            .ok_or_else(|| "todo not found".to_string())?;
        self.move_todo(id, target_day)?;
        Ok(UndoAction::Move { id, previous_day })
    }

    pub fn toggle_done(&mut self, id: Uuid) -> Result<(), String> {
        match self.todos.iter_mut().find(|todo| todo.id == id) {
            Some(todo) => {
                if todo.status == Status::Done {
                    todo.status = Status::Backlog;
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

    pub fn toggle_done_with_undo(&mut self, id: Uuid) -> Result<UndoAction, String> {
        let todo = self
            .todos
            .iter()
            .find(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;
        let undo = UndoAction::Toggle {
            id,
            previous_status: todo.status,
            previous_completed_at: todo.completed_at,
        };
        self.toggle_done(id)?;
        Ok(undo)
    }

    pub fn set_status(&mut self, id: Uuid, status: Status) -> Result<(), String> {
        match self.todos.iter_mut().find(|todo| todo.id == id) {
            Some(todo) => {
                todo.status = status;
                todo.completed_at = if status == Status::Done {
                    Some(chrono::Utc::now())
                } else {
                    None
                };
                Ok(())
            }
            None => Err("todo not found".to_string()),
        }
    }

    pub fn set_status_with_undo(&mut self, id: Uuid, status: Status) -> Result<UndoAction, String> {
        let todo = self
            .todos
            .iter()
            .find(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;
        let undo = UndoAction::SetStatus {
            id,
            previous_status: todo.status,
            previous_completed_at: todo.completed_at,
        };
        self.set_status(id, status)?;
        Ok(undo)
    }

    pub fn delete_todo(&mut self, id: Uuid) -> Result<(), String> {
        let pos = self
            .todos
            .iter()
            .position(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;
        self.todos.remove(pos);
        Ok(())
    }

    pub fn delete_todo_with_undo(&mut self, id: Uuid) -> Result<UndoAction, String> {
        let pos = self
            .todos
            .iter()
            .position(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;
        let todo = self
            .todos
            .get(pos)
            .cloned()
            .ok_or_else(|| "todo not found".to_string())?;
        self.todos.remove(pos);
        Ok(UndoAction::Delete { todo, index: pos })
    }

    pub fn apply_undo(&mut self, undo: UndoAction) -> Result<(), String> {
        match undo {
            UndoAction::Delete { todo, index } => {
                let insert_at = index.min(self.todos.len());
                self.todos.insert(insert_at, todo);
                Ok(())
            }
            UndoAction::Move { id, previous_day } => self.move_todo(id, previous_day),
            UndoAction::Toggle {
                id,
                previous_status,
                previous_completed_at,
            } => {
                let todo = self
                    .todos
                    .iter_mut()
                    .find(|todo| todo.id == id)
                    .ok_or_else(|| "todo not found".to_string())?;
                todo.status = previous_status;
                todo.completed_at = previous_completed_at;
                Ok(())
            }
            UndoAction::SetStatus {
                id,
                previous_status,
                previous_completed_at,
            } => {
                let todo = self
                    .todos
                    .iter_mut()
                    .find(|todo| todo.id == id)
                    .ok_or_else(|| "todo not found".to_string())?;
                todo.status = previous_status;
                todo.completed_at = previous_completed_at;
                Ok(())
            }
        }
    }

    pub fn update_todo(
        &mut self,
        id: Uuid,
        title: String,
        priority: Priority,
        assigned_day: NaiveDate,
    ) -> Result<(), String> {
        self.update_todo_with_description(id, title, priority, assigned_day, None, None)
    }

    pub fn update_todo_with_description(
        &mut self,
        id: Uuid,
        title: String,
        priority: Priority,
        assigned_day: NaiveDate,
        description: Option<String>,
        due_time: Option<NaiveTime>,
    ) -> Result<(), String> {
        let todo = self
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;

        todo.title = title;
        todo.priority = priority;
        todo.assigned_day = assigned_day;
        todo.due_time = due_time;
        todo.description = description.and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        });
        Ok(())
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

    pub fn select_next_month(&mut self) {
        if let Ok(next) = shift_month_date(self.selected_day, 1) {
            self.selected_day = next;
        }
    }

    pub fn select_prev_month(&mut self) {
        if let Ok(prev) = shift_month_date(self.selected_day, -1) {
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
        Self::for_day_as_of(day, day, todos)
    }

    pub fn for_day_as_of(day: NaiveDate, as_of: NaiveDate, todos: &[Todo]) -> Self {
        let overdue_cutoff = std::cmp::min(day, as_of);

        let overdue = todos
            .iter()
            .filter(|todo| {
                !matches!(todo.status, Status::Done) && todo.assigned_day < overdue_cutoff
            })
            .cloned()
            .collect::<Vec<_>>();

        let today = todos
            .iter()
            .filter(|todo| todo.assigned_day == day)
            .cloned()
            .collect::<Vec<_>>();

        Self { overdue, today }
    }

    pub fn filter_by_query(&self, query: &str) -> Self {
        let query = query.trim().to_lowercase();

        let mut filtered_overdue = self
            .overdue
            .iter()
            .filter_map(|todo| {
                fuzzy_title_score(&todo.title, &query).map(|score| (todo.clone(), score))
            })
            .collect::<Vec<_>>();

        let mut filtered_today = self
            .today
            .iter()
            .filter_map(|todo| {
                fuzzy_title_score(&todo.title, &query).map(|score| (todo.clone(), score))
            })
            .collect::<Vec<_>>();

        rank_todos_by_score(&mut filtered_overdue);
        rank_todos_by_score(&mut filtered_today);

        Self {
            overdue: filtered_overdue.into_iter().map(|(todo, _)| todo).collect(),
            today: filtered_today.into_iter().map(|(todo, _)| todo).collect(),
        }
    }
}

pub fn search_todos(todos: &[Todo], query: &str) -> Vec<Todo> {
    let query = query.trim().to_lowercase();
    let mut scored = todos
        .iter()
        .filter_map(|todo| {
            fuzzy_title_score(&todo.title, &query).map(|score| (todo.clone(), score))
        })
        .collect::<Vec<_>>();

    rank_todos_by_score(&mut scored);

    scored.into_iter().map(|(todo, _)| todo).collect()
}

pub fn board_todos(todos: &[Todo], query: &str) -> Vec<Todo> {
    let mut lanes = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for todo in search_todos(todos, query) {
        if is_archived_completed(&todo) {
            continue;
        }
        lanes[status_lane_index(todo.status)].push(todo);
    }

    lanes.into_iter().flatten().collect()
}

pub fn archived_completed_todos(todos: &[Todo], query: &str) -> Vec<Todo> {
    search_todos(todos, query)
        .into_iter()
        .filter(is_archived_completed)
        .collect()
}

fn is_archived_completed(todo: &Todo) -> bool {
    matches!(todo.status, Status::Done)
        && todo
            .completed_at
            .is_some_and(|completed_at| completed_at < Utc::now() - Duration::days(7))
}

pub fn status_lane_index(status: Status) -> usize {
    match status {
        Status::Backlog => 0,
        Status::Doing => 1,
        Status::Blocked => 2,
        Status::Done => 3,
    }
}

fn fuzzy_title_score(title: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(1);
    }

    let title = title.to_lowercase();

    // highest: exact title match
    if title == query {
        return Some(500);
    }

    // strong: token-prefix match
    if title
        .split(|c: char| !c.is_alphanumeric())
        .any(|token| !token.is_empty() && token.starts_with(query))
    {
        return Some(300);
    }

    // medium: contiguous substring match (earlier index scores higher)
    if let Some(index) = title.find(query) {
        return Some(200usize.saturating_sub(index));
    }

    // weakest: subsequence match
    let mut query_chars = query.chars();
    let mut current = query_chars.next();
    for ch in title.chars() {
        if let Some(qch) = current {
            if ch == qch {
                current = query_chars.next();
            }
        } else {
            break;
        }
    }

    if current.is_none() { Some(100) } else { None }
}

fn rank_todos_by_score(scored: &mut [(Todo, usize)]) {
    scored.sort_by(|(a_todo, a_score), (b_todo, b_score)| {
        b_score
            .cmp(a_score)
            .then_with(|| a_todo.title.len().cmp(&b_todo.title.len()))
            .then_with(|| a_todo.title.cmp(&b_todo.title))
    });
}

pub fn toggle_help_overlay(current: Overlay) -> Overlay {
    if current == Overlay::Help {
        Overlay::None
    } else {
        Overlay::Help
    }
}

pub fn request_quit_overlay(current: Overlay) -> Overlay {
    match current {
        Overlay::None => Overlay::QuitConfirm,
        Overlay::Help => Overlay::None,
        Overlay::QuitConfirm => Overlay::QuitConfirm,
    }
}

pub fn footer_hint(overlay: Overlay, search_active: bool, search_query: &str) -> (String, bool) {
    match overlay {
        Overlay::None => {
            if search_active {
                let query_display = if search_query.is_empty() {
                    "[search: __] [Esc] clear".to_string()
                } else {
                    format!("[search: {search_query}_] [Esc] clear")
                };
                (query_display, true)
            } else {
                (
                    "[?] help [/] search [u] undo [q] quit [j/k] move [enter] done".to_string(),
                    false,
                )
            }
        }
        Overlay::Help => ("[Esc/?] close help".to_string(), false),
        Overlay::QuitConfirm => ("[y] quit [n/Esc] cancel".to_string(), false),
    }
}
