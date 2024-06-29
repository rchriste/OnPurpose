use std::{fmt::Display, time::Duration};

pub(crate) struct DisplayDurationOneUnit<'s> {
    duration: &'s Duration,
}

impl Display for DisplayDurationOneUnit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.duration;
        let millis = duration.as_millis() % 1000;
        let seconds = duration.as_secs() as f32;
        let minutes = seconds / 60.0;
        let hours = minutes / 60.0;
        let days = hours / 24.0;
        let weeks = days / 7.0;
        let months = weeks / 4.0;
        let years = months / 12.0;
        if years > 0.9 {
            if years.fract() >= 0.1 {
                write!(f, "{:.1} years", years)
            } else {
                write!(f, "{:.0} years", years)
            }
        } else if months > 0.9 {
            if months.fract() >= 0.1 {
                write!(f, "{:.1} months", months)
            } else {
                write!(f, "{:.0} months", months)
            }
        } else if weeks > 0.9 {
            if weeks.fract() >= 0.1 {
                write!(f, "{:.1} weeks", weeks)
            } else {
                write!(f, "{:.0} weeks", weeks)
            }
        } else if days > 0.9 {
            if days.fract() >= 0.1 {
                write!(f, "{:.1} days", days)
            } else {
                write!(f, "{:.0} days", days)
            }
        } else if hours > 0.9 {
            if hours.fract() >= 0.1 {
                write!(f, "{:.1} hours", hours)
            } else {
                write!(f, "{:.0} hours", hours)
            }
        } else if minutes > 0.9 {
            if minutes.fract() >= 0.1 {
                write!(f, "{:.1} minutes", minutes)
            } else {
                write!(f, "{:.0} minutes", minutes)
            }
        } else if seconds > 0.9 {
            if seconds.fract() >= 0.1 {
                write!(f, "{:.1} seconds", seconds)
            } else {
                write!(f, "{:.0} seconds", seconds)
            }
        } else if millis > 0 {
            write!(f, "{} ms", millis)
        } else {
            write!(f, "0 ms")
        }
    }
}

impl<'s> DisplayDurationOneUnit<'s> {
    pub(crate) fn new(duration: &'s Duration) -> Self {
        DisplayDurationOneUnit { duration }
    }
}
