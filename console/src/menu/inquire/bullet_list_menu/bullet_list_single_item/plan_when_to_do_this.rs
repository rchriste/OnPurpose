use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    node::item_lap_count::ItemLapCount,
    surrealdb_layer::{
        surreal_item::{SurrealScheduled, SurrealScheduledPriority},
        DataLayerCommands,
    },
};

enum StartWhenOption {
    ExactTime,
    TimeRange,
}

impl Display for StartWhenOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartWhenOption::ExactTime => write!(f, "Exact Time"),
            StartWhenOption::TimeRange => write!(f, "Time Range"),
        }
    }
}

pub(crate) enum StartWhen {
    ExactTime(DateTime<Utc>),
    TimeRange(DateTime<Utc>, DateTime<Utc>),
}

pub(crate) enum ScheduledPriority {
    Always,
    RoutineActive,
}

impl From<ScheduledPriority> for SurrealScheduledPriority {
    fn from(value: ScheduledPriority) -> Self {
        match value {
            ScheduledPriority::Always => SurrealScheduledPriority::Always,
            ScheduledPriority::RoutineActive => SurrealScheduledPriority::WhenRoutineIsActive,
        }
    }
}

impl Display for ScheduledPriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduledPriority::Always => write!(f, "Always"),
            ScheduledPriority::RoutineActive => write!(f, "If Routine Is Active"),
        }
    }
}

pub(crate) async fn plan_when_to_do_this(
    item_lap_count: &ItemLapCount<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let start_when = vec![StartWhenOption::ExactTime, StartWhenOption::TimeRange];
    let start_when = Select::new("When do you want to start this item?", start_when).prompt();
    let start_when = match start_when {
        Ok(StartWhenOption::ExactTime) => {
            let exact_start = Text::new("Enter the exact time you want to start this item")
                .prompt()
                .unwrap();
            let exact_start = match parse(&exact_start) {
                Ok(exact_start) => {
                    let now = Utc::now();
                    now + exact_start
                }
                Err(_) => match dateparser::parse(&exact_start) {
                    Ok(exact_start) => exact_start,
                    Err(e) => {
                        todo!("Error: {:?}", e)
                    }
                },
            };
            StartWhen::ExactTime(exact_start)
        }
        Ok(StartWhenOption::TimeRange) => {
            let range_start = Text::new("Enter the start of the range").prompt().unwrap();
            let range_start = match parse(&range_start) {
                Ok(range_start) => {
                    let now = Utc::now();
                    now + range_start
                }
                Err(_) => match dateparser::parse(&range_start) {
                    Ok(range_start) => range_start,
                    Err(e) => {
                        todo!("Error: {:?}", e)
                    }
                },
            };
            let range_end = Text::new("Enter the end of the range").prompt().unwrap();
            let range_end = match parse(&range_end) {
                Ok(range_end) => {
                    let now = Utc::now();
                    now + range_end
                }
                Err(_) => match dateparser::parse(&range_end) {
                    Ok(range_end) => range_end,
                    Err(e) => {
                        todo!("Error: {:?}", e)
                    }
                },
            };
            StartWhen::TimeRange(range_start, range_end)
        }
        Err(InquireError::OperationCanceled) => {
            println!("Operation canceled");
            return Ok(());
        }
        Err(InquireError::OperationInterrupted) => return Err(()),
        Err(e) => {
            todo!("Error: {:?}", e)
        }
    };

    let timeboxed = Text::new("Timebox how much time for this item")
        .prompt()
        .unwrap();
    let timeboxed = match parse(&timeboxed) {
        Ok(timeboxed) => timeboxed,
        Err(e) => {
            todo!("Error: {:?}", e)
        }
    };

    // Is this scheduled for if the routine is active or scheduled for always?
    let scheduled_priority = vec![ScheduledPriority::Always, ScheduledPriority::RoutineActive];
    let scheduled_priority = Select::new(
        "What is the priority of this scheduled item?",
        scheduled_priority,
    )
    .prompt()
    .unwrap();

    let surreal_scheduled = match start_when {
        StartWhen::ExactTime(exact_start) => SurrealScheduled::ScheduledExact {
            start: exact_start.into(),
            duration: timeboxed.into(),
            priority: scheduled_priority.into(),
        },
        StartWhen::TimeRange(range_start, range_end) => SurrealScheduled::ScheduledRange {
            start_range: (range_start.into(), range_end.into()),
            duration: timeboxed.into(),
            priority: scheduled_priority.into(),
        },
    };
    // For now because it is easier to code and because dealing with active conflicts does need to happen regardless
    // I just add the item and then let the system for dealing with conflicts that comes later deal with overbooking
    // but in the future we might consider checking and dealing with conflicts here.
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateScheduled(
            item_lap_count.get_surreal_record_id().clone(),
            surreal_scheduled,
        ))
        .await
        .unwrap();
    Ok(())
}
