//! Human-friendly schedule parsing for cron expressions.
//!
//! Parses natural language schedules like `every 15 minutes`, `every day at 03:30`,
//! `every monday`, and `every mon, wed and fri at 06:30` into standard 5-field cron strings.

pub mod errors;
pub mod schedule;

use errors::CrontextError;
use schedule::Schedule;

/// Parse a human-friendly schedule expression into a [`Schedule`].
///
/// # Errors
///
/// Returns [`CrontextError`] if the expression is invalid, does not start with `every`,
/// or uses interval steps that do not divide cleanly into cron unit boundaries.
pub fn parse(input: &str) -> Result<Schedule, CrontextError> {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return Err(CrontextError::Empty);
    }

    if !normalized.starts_with("every") {
        return Err(CrontextError::MissingEvery);
    }

    let remainder = normalized["every".len()..].trim();
    if remainder.is_empty() {
        return Err(CrontextError::Empty);
    }

    parse_remainder(remainder)
}

/// Normalize whitespace and lowercase input string.
fn normalize(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Parse the remainder of the schedule string after the leading `every`.
fn parse_remainder(rem: &str) -> Result<Schedule, CrontextError> {
    // 1. Bare clock time: "every 12:00", "every 6pm", "every 03:30"
    if !rem.contains(' ') && (rem.contains(':') || rem.ends_with("am") || rem.ends_with("pm")) {
        let (min, hour) = parse_time(rem)?;
        return Ok(Schedule {
            cron: format!("{min} {hour} * * *"),
            description: format!("every day at {hour:02}:{min:02}"),
        });
    }

    // 2. Minute intervals
    if let Some(schedule) = try_parse_minutes(rem)? {
        return Ok(schedule);
    }

    // 3. Hour intervals
    if let Some(schedule) = try_parse_hours(rem)? {
        return Ok(schedule);
    }

    // 4. Daily intervals
    if let Some(schedule) = try_parse_days(rem)? {
        return Ok(schedule);
    }

    // 5. Monthly intervals
    if let Some(schedule) = try_parse_months(rem)? {
        return Ok(schedule);
    }

    // 6. Yearly: "1st of february", "15th of december at 09:00"
    if let Some(schedule) = try_parse_yearly(rem)? {
        return Ok(schedule);
    }

    // 7. Weekday / Weekend shortcuts
    if let Some(schedule) = try_parse_weekday_or_weekend(rem)? {
        return Ok(schedule);
    }

    // 8. Named days of week (e.g. "friday at 18:00", "mon, wed and fri at 06:30")
    if let Some(schedule) = try_parse_days_of_week(rem)? {
        return Ok(schedule);
    }

    Err(CrontextError::InvalidExpression {
        input: rem.to_owned(),
    })
}

/// Try to parse minute expressions: "minute", "1 minute", "15 minutes".
fn try_parse_minutes(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    if rem == "minute" || rem == "1 minute" || rem == "1 minutes" {
        return Ok(Some(Schedule {
            cron: "* * * * *".to_owned(),
            description: "every minute".to_owned(),
        }));
    }

    let tokens: Vec<&str> = rem.split_whitespace().collect();
    if tokens.len() == 2
        && (tokens[1] == "minute" || tokens[1] == "minutes")
        && let Ok(n) = tokens[0].parse::<u32>()
    {
        if n == 0 {
            return Err(CrontextError::InvalidStep {
                unit: "minutes",
                step: 0,
                reason: "step must be at least 1".to_owned(),
            });
        }
        if n == 1 {
            return Ok(Some(Schedule {
                cron: "* * * * *".to_owned(),
                description: "every minute".to_owned(),
            }));
        }
        if 60 % n != 0 {
            return Err(CrontextError::InvalidStep {
                unit: "minutes",
                step: n,
                reason: format!(
                    "every {n} minutes does not divide 60, use a factor of 60 (e.g. 1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30) or a raw cron expression"
                ),
            });
        }
        return Ok(Some(Schedule {
            cron: format!("*/{n} * * * *"),
            description: format!("every {n} minutes"),
        }));
    }

    Ok(None)
}

/// Try to parse hour expressions: "hour", "1 hour", "12 hours".
fn try_parse_hours(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    if rem == "hour" || rem == "1 hour" || rem == "1 hours" {
        return Ok(Some(Schedule {
            cron: "0 * * * *".to_owned(),
            description: "every hour".to_owned(),
        }));
    }

    let tokens: Vec<&str> = rem.split_whitespace().collect();
    if tokens.len() == 2
        && (tokens[1] == "hour" || tokens[1] == "hours")
        && let Ok(n) = tokens[0].parse::<u32>()
    {
        if n == 0 {
            return Err(CrontextError::InvalidStep {
                unit: "hours",
                step: 0,
                reason: "step must be at least 1".to_owned(),
            });
        }
        if n == 1 {
            return Ok(Some(Schedule {
                cron: "0 * * * *".to_owned(),
                description: "every hour".to_owned(),
            }));
        }
        if 24 % n != 0 {
            return Err(CrontextError::InvalidStep {
                unit: "hours",
                step: n,
                reason: format!(
                    "every {n} hours does not divide 24, use a factor of 24 (e.g. 1, 2, 3, 4, 6, 8, 12) or a raw cron expression"
                ),
            });
        }
        return Ok(Some(Schedule {
            cron: format!("0 */{n} * * *"),
            description: format!("every {n} hours"),
        }));
    }

    Ok(None)
}

/// Try to parse day expressions: "day", "1 day", "day at 03:30", "1 day at 03:30", "2 days".
fn try_parse_days(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    if rem == "day" || rem == "1 day" || rem == "1 days" {
        return Ok(Some(Schedule {
            cron: "0 0 * * *".to_owned(),
            description: "every day at 00:00".to_owned(),
        }));
    }

    // "day at <time>", "1 day at <time>", "1 days at <time>"
    for prefix in ["day at ", "1 day at ", "1 days at "] {
        if let Some(time_str) = rem.strip_prefix(prefix) {
            let (min, hour) = parse_time(time_str.trim())?;
            return Ok(Some(Schedule {
                cron: format!("{min} {hour} * * *"),
                description: format!("every day at {hour:02}:{min:02}"),
            }));
        }
    }

    // Invalid day steps like "2 days", "5 days"
    let tokens: Vec<&str> = rem.split_whitespace().collect();
    if !tokens.is_empty()
        && let Ok(n) = tokens[0].parse::<u32>()
        && tokens.len() >= 2
        && (tokens[1] == "day" || tokens[1] == "days")
        && n != 1
    {
        return Err(CrontextError::InvalidStep {
            unit: "days",
            step: n,
            reason: format!(
                "every {n} days is not supported (cron step resets at month boundaries); use `every 1 day` or a raw cron expression"
            ),
        });
    }

    Ok(None)
}

/// Try to parse month expressions: "month", "month on the 1st at 03:00".
fn try_parse_months(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    if rem == "month" || rem == "1 month" || rem == "1 months" {
        return Ok(Some(Schedule {
            cron: "0 0 1 * *".to_owned(),
            description: "every month on the 1st at 00:00".to_owned(),
        }));
    }

    let prefix = if let Some(rest) = rem.strip_prefix("month ") {
        rest
    } else if let Some(rest) = rem.strip_prefix("1 month ") {
        rest
    } else {
        return Ok(None);
    };

    let rest = prefix.trim();
    let rest = rest.strip_prefix("on ").unwrap_or(rest).trim();
    let rest = rest.strip_prefix("the ").unwrap_or(rest).trim();

    // Now rest is like "1st at 03:00" or "1st" or "15th at 18:00"
    let (dom_part, time_part) = match rest.split_once(" at ") {
        Some((d, t)) => (d.trim(), Some(t.trim())),
        None => (rest.trim(), None),
    };

    let day = parse_day_of_month(dom_part)?;
    let (min, hour) = match time_part {
        Some(t) => parse_time(t)?,
        None => (0, 0),
    };

    let ordinal = ordinal_suffix(day);
    Ok(Some(Schedule {
        cron: format!("{min} {hour} {day} * *"),
        description: format!("every month on the {ordinal} at {hour:02}:{min:02}"),
    }))
}

/// Inner implementation split out so the closure trick above doesn't obscure the real logic.
fn try_parse_yearly(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    let (core, time_part) = match rem.split_once(" at ") {
        Some((c, t)) => (c.trim(), Some(t.trim())),
        None => (rem.trim(), None),
    };

    // Expect "<ordinal> of <month>"
    let (ord_str, month_str) = match core.split_once(" of ") {
        Some(pair) => pair,
        None => return Ok(None),
    };

    // Validate ordinal token looks like a day (digits + optional suffix)
    let (month_num, max_days, month_display) = match parse_month_name(month_str.trim()) {
        Some(m) => m,
        None => return Ok(None), // not a month name — don't error, just skip
    };

    let day = parse_day_for_named_month(ord_str.trim(), max_days, month_display)?;
    let (min, hour) = match time_part {
        Some(t) => parse_time(t)?,
        None => (0, 0),
    };

    let ordinal = ordinal_suffix(day);
    Ok(Some(Schedule {
        cron: format!("{min} {hour} {day} {month_num} *"),
        description: format!("every {month_display} {ordinal} at {hour:02}:{min:02}"),
    }))
}

/// Parse a month name (full or 3-letter abbreviation) into `(month_num, max_days, display_name)`.
/// Returns `None` for unrecognized strings so the caller can fall through.
fn parse_month_name(s: &str) -> Option<(u8, u32, &'static str)> {
    match s {
        "jan" | "january" => Some((1, 31, "January")),
        "feb" | "february" => Some((2, 28, "February")),
        "mar" | "march" => Some((3, 31, "March")),
        "apr" | "april" => Some((4, 30, "April")),
        "may" => Some((5, 31, "May")),
        "jun" | "june" => Some((6, 30, "June")),
        "jul" | "july" => Some((7, 31, "July")),
        "aug" | "august" => Some((8, 31, "August")),
        "sep" | "september" => Some((9, 30, "September")),
        "oct" | "october" => Some((10, 31, "October")),
        "nov" | "november" => Some((11, 30, "November")),
        "dec" | "december" => Some((12, 31, "December")),
        _ => None,
    }
}

/// Strip ordinal suffix (st/nd/rd/th) and parse, validating against `max_days`.
fn parse_day_for_named_month(
    s: &str,
    max_days: u32,
    month: &'static str,
) -> Result<u32, CrontextError> {
    let num_str = s
        .strip_suffix("st")
        .or_else(|| s.strip_suffix("nd"))
        .or_else(|| s.strip_suffix("rd"))
        .or_else(|| s.strip_suffix("th"))
        .unwrap_or(s);

    let day = num_str
        .parse::<u32>()
        .map_err(|_| CrontextError::InvalidExpression {
            input: s.to_owned(),
        })?;

    if day < 1 || day > max_days {
        return Err(CrontextError::InvalidDayForMonth {
            day,
            month,
            max_days,
        });
    }

    Ok(day)
}

fn try_parse_weekday_or_weekend(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    if rem == "weekday" || rem == "weekdays" {
        return Ok(Some(Schedule {
            cron: "0 0 * * 1-5".to_owned(),
            description: "every weekday at 00:00".to_owned(),
        }));
    }

    if let Some(rest) = rem
        .strip_prefix("weekday at ")
        .or_else(|| rem.strip_prefix("weekdays at "))
    {
        let (min, hour) = parse_time(rest.trim())?;
        return Ok(Some(Schedule {
            cron: format!("{min} {hour} * * 1-5"),
            description: format!("every weekday at {hour:02}:{min:02}"),
        }));
    }

    if rem == "weekend" || rem == "weekends" {
        return Ok(Some(Schedule {
            cron: "0 0 * * 6,0".to_owned(),
            description: "every weekend at 00:00".to_owned(),
        }));
    }

    if let Some(rest) = rem
        .strip_prefix("weekend at ")
        .or_else(|| rem.strip_prefix("weekends at "))
    {
        let (min, hour) = parse_time(rest.trim())?;
        return Ok(Some(Schedule {
            cron: format!("{min} {hour} * * 6,0"),
            description: format!("every weekend at {hour:02}:{min:02}"),
        }));
    }

    Ok(None)
}

/// Try to parse named days of the week: "monday", "friday at 18:00", "mon, wed and fri at 06:30".
fn try_parse_days_of_week(rem: &str) -> Result<Option<Schedule>, CrontextError> {
    let (days_part, time_part) = match rem.split_once(" at ") {
        Some((d, t)) => (d.trim(), Some(t.trim())),
        None => (rem.trim(), None),
    };

    let (min, hour) = match time_part {
        Some(t) => parse_time(t)?,
        None => (0, 0),
    };

    // Split days by commas, whitespace, and "and"
    let mut day_nums = Vec::new();
    let mut day_names = Vec::new();

    // Replace commas with spaces to treat as uniform delimiters
    let cleaned = days_part.replace(',', " ");
    for token in cleaned.split_whitespace() {
        let token = token.trim();
        if token.is_empty() || token == "and" {
            continue;
        }

        let day_num = parse_day_name(token)?;
        day_nums.push(day_num);
        day_names.push(token);
    }

    if day_nums.is_empty() {
        return Ok(None);
    }

    // Sort days with standard display order: Mon(1)..Sat(6), Sun(0)
    day_nums.sort_by_key(|&d| if d == 0 { 7 } else { d });
    day_nums.dedup();

    let cron_days = day_nums
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let desc_days = day_nums
        .iter()
        .map(|&d| day_name_display(d))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(Some(Schedule {
        cron: format!("{min} {hour} * * {cron_days}"),
        description: format!("every {desc_days} at {hour:02}:{min:02}"),
    }))
}

/// Parse a day name into cron day number (0=Sunday, 1=Monday, ..., 6=Saturday).
fn parse_day_name(s: &str) -> Result<u8, CrontextError> {
    match s {
        "mon" | "monday" | "mondays" => Ok(1),
        "tue" | "tues" | "tuesday" | "tuesdays" => Ok(2),
        "wed" | "wednesday" | "wednesdays" => Ok(3),
        "thu" | "thur" | "thurs" | "thursday" | "thursdays" => Ok(4),
        "fri" | "friday" | "fridays" => Ok(5),
        "sat" | "saturday" | "saturdays" => Ok(6),
        "sun" | "sunday" | "sundays" => Ok(0),
        _ => Err(CrontextError::InvalidDayName { name: s.to_owned() }),
    }
}

/// Get short canonical name for day number for description display.
fn day_name_display(day: u8) -> &'static str {
    match day {
        1 => "monday",
        2 => "tuesday",
        3 => "wednesday",
        4 => "thursday",
        5 => "friday",
        6 => "saturday",
        0 => "sunday",
        _ => "unknown",
    }
}

/// Parse day of month number (1..=28).
fn parse_day_of_month(s: &str) -> Result<u32, CrontextError> {
    let trimmed = s.trim();
    let num_str = trimmed
        .strip_suffix("st")
        .or_else(|| trimmed.strip_suffix("nd"))
        .or_else(|| trimmed.strip_suffix("rd"))
        .or_else(|| trimmed.strip_suffix("th"))
        .unwrap_or(trimmed);

    let day = num_str
        .parse::<u32>()
        .map_err(|_| CrontextError::InvalidExpression {
            input: s.to_owned(),
        })?;

    if !(1..=28).contains(&day) {
        return Err(CrontextError::InvalidDayOfMonth { day });
    }

    Ok(day)
}

/// Helper to render ordinal suffix for a day number.
fn ordinal_suffix(day: u32) -> String {
    let suffix = match day {
        1 | 21 => "1st",
        2 | 22 => "2nd",
        3 | 23 => "3rd",
        _ => "th",
    };
    if suffix == "th" {
        format!("{day}th")
    } else {
        suffix.to_owned()
    }
}

/// Parse time string in 24h format (`HH:MM`, `H:MM`) or 12h format (`6pm`, `6:30pm`, `12am`).
/// Returns `(minute, hour)`.
fn parse_time(s: &str) -> Result<(u32, u32), CrontextError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(CrontextError::InvalidTime { time: s.to_owned() });
    }

    // 12-hour am/pm format check
    if s.ends_with("am") || s.ends_with("pm") {
        let is_pm = s.ends_with("pm");
        let bare = &s[..s.len() - 2].trim();

        let (h_raw, m_raw) = if let Some((h_str, m_str)) = bare.split_once(':') {
            let h = h_str
                .parse::<u32>()
                .map_err(|_| CrontextError::InvalidTime { time: s.to_owned() })?;
            let m = m_str
                .parse::<u32>()
                .map_err(|_| CrontextError::InvalidTime { time: s.to_owned() })?;
            (h, m)
        } else {
            let h = bare
                .parse::<u32>()
                .map_err(|_| CrontextError::InvalidTime { time: s.to_owned() })?;
            (h, 0)
        };

        if !(1..=12).contains(&h_raw) || m_raw >= 60 {
            return Err(CrontextError::InvalidTime { time: s.to_owned() });
        }

        let hour = match (is_pm, h_raw) {
            (false, 12) => 0,
            (false, h) => h,
            (true, 12) => 12,
            (true, h) => h + 12,
        };

        return Ok((m_raw, hour));
    }

    // 24-hour HH:MM or H:MM format
    if let Some((h_str, m_str)) = s.split_once(':') {
        let h = h_str
            .parse::<u32>()
            .map_err(|_| CrontextError::InvalidTime { time: s.to_owned() })?;
        let m = m_str
            .parse::<u32>()
            .map_err(|_| CrontextError::InvalidTime { time: s.to_owned() })?;

        if h >= 24 || m >= 60 {
            return Err(CrontextError::InvalidTime { time: s.to_owned() });
        }

        return Ok((m, h));
    }

    Err(CrontextError::InvalidTime { time: s.to_owned() })
}

#[cfg(test)]
mod lib_test;
