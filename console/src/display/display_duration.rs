use std::{fmt::Display, time::Duration};

pub(crate) struct DisplayDuration<'s> {
    duration: &'s Duration,
}

impl Display for DisplayDuration<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.duration;
        let seconds = duration.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;
        let weeks = days / 7;
        let months = weeks / 4;
        let years = months / 12;
        let seconds = seconds % 60;
        let minutes = minutes % 60;
        let hours = hours % 24;
        let days = days % 7;
        let weeks = weeks % 4;
        let months = months % 12;
        let years = years % 12;
        let mut result = String::new();
        if years > 0 {
            result.push_str(&format!("{} years", years));
        }
        if months > 0 {
            result.push_str(&format!("{} months", months));
        }
        if weeks > 0 {
            result.push_str(&format!("{} weeks", weeks));
        }
        if days > 0 {
            result.push_str(&format!("{} days", days));
        }
        if hours > 0 {
            result.push_str(&format!("{} hours", hours));
        }
        if minutes > 0 {
            result.push_str(&format!("{} minutes", minutes));
        }
        if seconds > 0 {
            result.push_str(&format!("{} seconds", seconds));
        }
        write!(f, "{}", result)
    }
}

impl<'s> DisplayDuration<'s> {
    pub(crate) fn new(duration: &'s Duration) -> Self {
        DisplayDuration { duration }
    }
}
