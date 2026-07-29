use chrono::{Days, NaiveDate, NaiveTime};

use crate::model::Priority;

#[derive(Debug, Clone)]
pub struct QuickAddParsed {
    pub title: String,
    pub priority: Priority,
    pub assigned_day: NaiveDate,
    pub due_time: Option<NaiveTime>,
}

pub fn parse_quick_add(
    input: &str,
    default_priority: Priority,
    default_day: NaiveDate,
) -> Result<QuickAddParsed, String> {
    let mut priority = default_priority;
    let mut assigned_day = default_day;
    let mut due_time = None;
    let mut title_tokens = Vec::<String>::new();

    for token in input.split_whitespace() {
        let normalized = token.to_ascii_lowercase();
        if let Some(parsed_priority) = parse_priority_token(&normalized) {
            priority = parsed_priority;
            continue;
        }

        if let Some(token_body) = token.strip_prefix('@') {
            if token_body.is_empty() {
                title_tokens.push(token.to_string());
                continue;
            }

            if let Ok(parsed_time) = parse_time_token(token_body) {
                due_time = Some(parsed_time);
                continue;
            }

            if let Ok(parsed_date) = try_parse_date_token(token_body, default_day) {
                assigned_day = parsed_date;
                continue;
            }

            if appears_to_be_time_like(token_body) {
                return Err(format!("invalid time token: @{token}"));
            }

            if appears_to_be_date_like(token_body) {
                return Err(format!("invalid date token: @{token}"));
            }

            title_tokens.push(token.to_string());
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
        due_time,
    })
}

fn parse_time_token(token: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(token, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(token, "%-H:%M"))
        .map_err(|_| format!("invalid time token: @{token}"))
}

fn appears_to_be_time_like(token: &str) -> bool {
    token.contains(':') && token.len() >= 3
}

fn appears_to_be_date_like(token: &str) -> bool {
    token.contains('-')
}

fn parse_priority_token(token: &str) -> Option<Priority> {
    match token {
        "!h" | "!high" => Some(Priority::High),
        "!m" | "!med" | "!medium" => Some(Priority::Medium),
        "!l" | "!low" => Some(Priority::Low),
        _ => None,
    }
}

fn try_parse_date_token(token: &str, base_day: NaiveDate) -> Result<NaiveDate, String> {
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
