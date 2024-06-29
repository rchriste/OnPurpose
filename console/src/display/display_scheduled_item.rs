use std::fmt::Display;

use better_term::Style;
use chrono::{DateTime, Local, Utc};

use crate::{
    display::display_duration_one_unit::DisplayDurationOneUnit,
    surrealdb_layer::surreal_item::{SurrealScheduled, SurrealScheduledPriority},
    systems::upcoming::scheduled_item::ScheduledItem,
};

pub(crate) struct DisplayScheduledItem<'s> {
    scheduled_item: &'s ScheduledItem<'s>,
    now: DateTime<Local>,
}

impl<'s> Display for DisplayScheduledItem<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //scheduled_start in bold then +duration also in bold then start range
        //tab then name in italics
        match self.scheduled_item.get_scheduled() {
            SurrealScheduled::NotScheduled => panic!("Programming error, not a scheduled item"),
            SurrealScheduled::ScheduledExact {
                start: _start,
                duration,
                priority,
            } => {
                let scheduled_start = self.scheduled_item.get_scheduled_start();
                let scheduled_start = scheduled_start.with_timezone(&Local);
                writeln!(
                    f,
                    "{}={} lasting {}{} {}{}",
                    Style::default().bold(),
                    duration_or_time(&self.now, &scheduled_start),
                    DisplayDurationOneUnit::new(duration),
                    Style::default().italic(),
                    format_priority(priority),
                    Style::default()
                )?;
            }
            SurrealScheduled::ScheduledRange {
                start_range,
                duration,
                priority,
            } => {
                let scheduled_start = self.scheduled_item.get_scheduled_start();
                let scheduled_start = scheduled_start.with_timezone(&Local);
                let start_end_of_range = start_range.1.with_timezone(&Local);
                let delay_up_to = start_end_of_range - scheduled_start;
                let delay_up_to = delay_up_to.to_std().unwrap();
                let delay_up_to = DisplayDurationOneUnit::new(&delay_up_to);
                writeln!(
                    f,
                    "{}~{} delay up to {} lasting {}{} {}{}",
                    Style::default().bold(),
                    duration_or_time(&self.now, &scheduled_start),
                    delay_up_to,
                    DisplayDurationOneUnit::new(duration),
                    Style::default().italic(),
                    format_priority(priority),
                    Style::default()
                )?;
            }
        }
        writeln!(f, "\t{}", self.scheduled_item.get_summary())
    }
}

impl<'s> DisplayScheduledItem<'s> {
    pub(crate) fn new(scheduled_item: &'s ScheduledItem<'s>, now: &DateTime<Utc>) -> Self {
        let now = now.with_timezone(&Local);
        Self {
            scheduled_item,
            now,
        }
    }
}

fn duration_or_time(now: &DateTime<Local>, time: &DateTime<Local>) -> String {
    //Show duration if start is within two hours, otherwise show start time
    let duration = *time - now;
    if duration.num_minutes() < 1 {
        "Now".to_string()
    } else if duration.num_hours() < 2 {
        let duration = duration.to_std().unwrap();
        format!("{}", DisplayDurationOneUnit::new(&duration))
    } else {
        format!("{}", time.format("%a %d %b %Y %I:%M%p"))
    }
}

fn format_priority(priority: &SurrealScheduledPriority) -> String {
    match priority {
        SurrealScheduledPriority::Always => "Always".to_string(),
        SurrealScheduledPriority::WhenRoutineIsActive => "When Routine Scheduled".to_string(),
    }
}
