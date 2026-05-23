use std::collections::VecDeque;

use chrono::{Datelike, Days, NaiveDate};
use uuid::Uuid;

use crate::model::{Priority, Status, Todo};

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

#[derive(Debug, Clone)]
pub struct QuickAddParsed {
    pub title: String,
    pub priority: Priority,
    pub assigned_day: NaiveDate,
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
        self.add_todo_with_description(title, priority, assigned_day, None)
    }

    pub fn add_todo_with_description(
        &mut self,
        title: impl Into<String>,
        priority: Priority,
        assigned_day: NaiveDate,
        description: Option<String>,
    ) -> Uuid {
        let mut todo = Todo::new(title, priority, assigned_day);
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
        }
    }

    pub fn update_todo(
        &mut self,
        id: Uuid,
        title: String,
        priority: Priority,
        assigned_day: NaiveDate,
    ) -> Result<(), String> {
        self.update_todo_with_description(id, title, priority, assigned_day, None)
    }

    pub fn update_todo_with_description(
        &mut self,
        id: Uuid,
        title: String,
        priority: Priority,
        assigned_day: NaiveDate,
        description: Option<String>,
    ) -> Result<(), String> {
        let todo = self
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| "todo not found".to_string())?;

        todo.title = title;
        todo.priority = priority;
        todo.assigned_day = assigned_day;
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
        if let Ok(next) = shift_month(self.selected_day, 1) {
            self.selected_day = next;
        }
    }

    pub fn select_prev_month(&mut self) {
        if let Ok(prev) = shift_month(self.selected_day, -1) {
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

pub fn month_grid(selected_day: NaiveDate) -> Result<Vec<Option<NaiveDate>>, String> {
    let year = selected_day.year();
    let month = selected_day.month();
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("invalid month start: {year}-{month}"))?;
    let offset_u32 = first.weekday().num_days_from_sunday();
    let offset = usize::try_from(offset_u32)
        .map_err(|_| format!("weekday offset overflow: {offset_u32}"))?;
    let total_days = days_in_month(year, month)?;

    let mut cells = vec![None; 42];
    for day in 1..=total_days {
        let day_usize = usize::try_from(day).map_err(|_| format!("day overflow: {day}"))?;
        let idx = offset.saturating_add(day_usize.saturating_sub(1));
        if idx >= cells.len() {
            return Err("month grid overflow".to_string());
        }
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("invalid date in month grid: {year}-{month}-{day}"))?;
        cells[idx] = Some(date);
    }

    Ok(cells)
}

pub fn viewport_window(
    total_rows: usize,
    selected_index: usize,
    window_rows: usize,
) -> (usize, usize) {
    if window_rows == 0 {
        return (0, 0);
    }
    if total_rows <= window_rows {
        return (0, total_rows);
    }

    let half = window_rows / 2;
    let mut start = selected_index.saturating_sub(half);
    if start.saturating_add(window_rows) > total_rows {
        start = total_rows.saturating_sub(window_rows);
    }
    let end = start.saturating_add(window_rows);
    (start, end)
}

fn shift_month(day: NaiveDate, delta_months: i32) -> Result<NaiveDate, String> {
    let year_i64 = i64::from(day.year());
    let month_zero_based = i64::from(day.month0());
    let delta_i64 = i64::from(delta_months);
    let total_months = year_i64
        .checked_mul(12)
        .and_then(|value| value.checked_add(month_zero_based))
        .and_then(|value| value.checked_add(delta_i64))
        .ok_or_else(|| "month arithmetic overflow".to_string())?;

    let new_year_i64 = total_months.div_euclid(12);
    let new_month0_i64 = total_months.rem_euclid(12);
    let new_year =
        i32::try_from(new_year_i64).map_err(|_| format!("year out of range: {new_year_i64}"))?;
    let new_month0 = u32::try_from(new_month0_i64)
        .map_err(|_| format!("month out of range: {new_month0_i64}"))?;
    let new_month = new_month0.saturating_add(1);

    let max_day = days_in_month(new_year, new_month)?;
    let day_clamped = day.day().min(max_day);

    NaiveDate::from_ymd_opt(new_year, new_month, day_clamped).ok_or_else(|| {
        format!("failed to build shifted date: {new_year}-{new_month}-{day_clamped}")
    })
}

pub fn shift_month_date(day: NaiveDate, delta_months: i32) -> Result<NaiveDate, String> {
    shift_month(day, delta_months)
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

pub fn parse_quick_add(
    input: &str,
    default_priority: Priority,
    default_day: NaiveDate,
) -> Result<QuickAddParsed, String> {
    let mut priority = default_priority;
    let mut assigned_day = default_day;
    let mut title_tokens = Vec::<String>::new();

    for token in input.split_whitespace() {
        let normalized = token.to_ascii_lowercase();
        if let Some(parsed_priority) = parse_priority_token(&normalized) {
            priority = parsed_priority;
            continue;
        }

        if let Some(date_token) = token.strip_prefix('@') {
            assigned_day = parse_date_token(date_token, default_day)?;
            continue;
        }

        title_tokens.push(token.to_string());
    }

    let title = title_tokens.join(" ").trim().to_string();
    if title.is_empty() {
        return Err("title cannot be empty".to_string());
    }

    Ok(QuickAddParsed {
        title,
        priority,
        assigned_day,
    })
}

fn parse_priority_token(token: &str) -> Option<Priority> {
    match token {
        "!h" | "!high" => Some(Priority::High),
        "!m" | "!med" | "!medium" => Some(Priority::Medium),
        "!l" | "!low" => Some(Priority::Low),
        _ => None,
    }
}

fn parse_date_token(token: &str, base_day: NaiveDate) -> Result<NaiveDate, String> {
    let normalized = token.to_ascii_lowercase();
    match normalized.as_str() {
        "today" => Ok(base_day),
        "tomorrow" => base_day
            .checked_add_days(Days::new(1))
            .ok_or_else(|| format!("failed to compute tomorrow from {base_day}")),
        _ => NaiveDate::parse_from_str(token, "%Y-%m-%d")
            .map_err(|_| format!("invalid date token: @{token}")),
    }
}

fn days_in_month(year: i32, month: u32) -> Result<u32, String> {
    if month == 0 || month > 12 {
        return Err(format!("invalid month: {month}"));
    }

    if month == 12 {
        let next_year = year
            .checked_add(1)
            .ok_or_else(|| format!("year overflow: {year}"))?;
        let this = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| format!("invalid date: {year}-{month}-1"))?;
        let next = NaiveDate::from_ymd_opt(next_year, 1, 1)
            .ok_or_else(|| format!("invalid date: {next_year}-1-1"))?;
        let days_i64 = (next - this).num_days();
        return u32::try_from(days_i64)
            .map_err(|_| format!("days conversion overflow for {year}-{month}"));
    }

    let this = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("invalid date: {year}-{month}-1"))?;
    let next_month = month.saturating_add(1);
    let next = NaiveDate::from_ymd_opt(year, next_month, 1)
        .ok_or_else(|| format!("invalid date: {year}-{next_month}-1"))?;
    let days_i64 = (next - this).num_days();
    u32::try_from(days_i64).map_err(|_| format!("days conversion overflow for {year}-{month}"))
}
