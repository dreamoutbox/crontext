/// A parsed schedule consisting of a standard 5-field cron expression and a human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    /// Standard 5-field cron string: `minute hour day-of-month month day-of-week`.
    pub cron: String,
    /// Human-friendly description of the schedule.
    pub description: String,
}

impl std::fmt::Display for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.cron, self.description)
    }
}
