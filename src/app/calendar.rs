use chrono::{Datelike, NaiveDate};

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
