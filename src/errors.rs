/// Errors occurring during schedule parsing.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CrontextError {
    /// The schedule expression was empty or contained only whitespace.
    #[error("schedule expression cannot be empty")]
    Empty,

    /// The schedule did not begin with the keyword `every`.
    #[error("schedule must begin with 'every'")]
    MissingEvery,

    /// The interval step is invalid for cron boundaries.
    #[error("invalid step `{step}` for {unit}: {reason}")]
    InvalidStep {
        /// Time unit (e.g. "minutes", "hours", "days").
        unit: &'static str,
        /// The invalid numeric step.
        step: u32,
        /// Explanation and alternative suggestion.
        reason: String,
    },

    /// The day of month is outside the safe range of 1..=28 (for generic monthly schedules).
    #[error(
        "day of month `{day}` must be between 1 and 28 (to avoid skipped short months like February)"
    )]
    InvalidDayOfMonth {
        /// The invalid day.
        day: u32,
    },

    /// The day is out of range for the specific named month.
    #[error("day `{day}` is out of range for {month}: must be between 1 and {max_days}")]
    InvalidDayForMonth {
        /// The invalid day.
        day: u32,
        /// The month name.
        month: &'static str,
        /// Maximum valid day for this month.
        max_days: u32,
    },

    /// Time string could not be parsed into a valid hour and minute.
    #[error(
        "invalid time `{time}`: expected HH:MM (00:00 to 23:59) or 12-hour format like 6pm, 6:30pm"
    )]
    InvalidTime {
        /// The invalid time string.
        time: String,
    },

    /// Unrecognized day of the week name.
    #[error(
        "unknown day name `{name}`: expected Monday through Sunday (e.g. mon, monday, mondays)"
    )]
    InvalidDayName {
        /// The unrecognized token.
        name: String,
    },

    /// Unrecognized schedule expression.
    #[error(
        "unrecognized schedule expression `{input}`; supported forms include: every [N] minute(s), every [N] hour(s), every day [at HH:MM], every HH:MM, every [day names] [at HH:MM], every weekday/weekend [at HH:MM], every month [on the Nth [at HH:MM]]"
    )]
    InvalidExpression {
        /// The unrecognized input.
        input: String,
    },
}
