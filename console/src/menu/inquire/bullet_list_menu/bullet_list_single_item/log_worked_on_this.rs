use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};
use surrealdb::{opt::RecordId, sql::Datetime};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    display::display_duration::DisplayDuration,
    new_time_spent::NewTimeSpent,
    node::{item_lap_count::ItemLapCount, item_status::ItemStatus, Filter},
    surrealdb_layer::{
        surreal_time_spent::{SurrealBulletListPosition, SurrealDedication},
        DataLayerCommands,
    },
    systems::bullet_list::BulletListReason,
};

pub(crate) async fn log_worked_on_this(
    selected: &ItemLapCount<'_>,
    when_selected: &DateTime<Utc>,
    bullet_list_created: &DateTime<Utc>,
    now: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    ordered_bullet_list: &[BulletListReason<'_>],
) -> Result<(), ()> {
    //TODO: Change Error return type to a custom struct ExitProgram or something
    // This logs time spent on an item with the goal of in the future making it possible for the user to adjust items and balance
    // Logs the following:
    // When starting to work on item
    // -Position in list
    let position_in_list = ordered_bullet_list
        .iter()
        .position(|reason| reason.get_surreal_record_id() == selected.get_surreal_record_id());

    let bullet_list_position = if let Some(position_in_list) = position_in_list {
        // -Lap count
        let lap_count = selected.get_lap_count();

        // -Lap count of next lower and next higher so you know how much you would need to dampen or boost to work on something else
        let next_lower_lap_count: Option<f32> = ordered_bullet_list
            .get(position_in_list + 1)
            .map(|reason| reason.get_lap_count());
        let next_higher_lap_count: Option<f32> = if position_in_list == 0 {
            None
        } else {
            ordered_bullet_list
                .get(position_in_list - 1)
                .map(|reason| reason.get_lap_count())
        };

        Some(SurrealBulletListPosition {
            position_in_list: position_in_list as u64,
            lap_count,
            next_lower_lap_count,
            next_higher_lap_count,
        })
    } else {
        None
    };

    let working_on = create_working_on_list(selected.get_item_status());
    // -When started
    loop {
        let (when_started, when_stopped) = ask_when_started_and_stopped(
            send_to_data_storage_layer,
            when_selected,
            bullet_list_created,
            now,
        )
        .await?;
        // -When marked "I worked on this"
        // -How much time spent, show amount of time since started and show amount of time since last item completed, or allow user to enter a duration
        if let Some(dedication) = ask_about_dedication()? {
            let time_spent = NewTimeSpent {
                working_on,
                bullet_list_position,
                when_started,
                when_stopped,
                dedication,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::RecordTimeSpent(time_spent))
                .await
                .unwrap();
            break;
        } else {
            continue; //If the user cancels they should be able to try again
        }
    }
    Ok(())
}

fn create_working_on_list(selected: &ItemStatus<'_>) -> Vec<RecordId> {
    selected
        .get_self_and_larger_flattened(Filter::Active)
        .iter()
        .map(|x| x.get_surreal_record_id().clone())
        .collect()
}

#[derive(Clone)]
enum StartedWhen {
    WhenLastItemFinished(DateTime<Local>),
    WhenBulletListWasFirstShown(DateTime<Local>),
    WhenThisItemWasSelected(DateTime<Local>),
    CustomTime,
}

impl Display for StartedWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartedWhen::WhenThisItemWasSelected(when_selected) => {
                write!(
                    f,
                    "When this item was selected (i.e. {})",
                    when_selected.format("%a %d %b %Y %I:%M:%S%p")
                )
            }
            StartedWhen::WhenLastItemFinished(when_last_item_finished) => {
                write!(
                    f,
                    "When the last item finished (i.e. {})",
                    when_last_item_finished.format("%a %d %b %Y %I:%M:%S%p")
                )
            }
            StartedWhen::WhenBulletListWasFirstShown(when_bullet_list_was_first_shown) => {
                write!(
                    f,
                    "When the bullet list was first shown (i.e. {})",
                    when_bullet_list_was_first_shown.format("%a %d %b %Y %I:%M:%S%p")
                )
            }
            StartedWhen::CustomTime => write!(f, "Enter a Time"),
        }
    }
}

enum StoppedWhen {
    Now,
    ManualTime,
}

impl Display for StoppedWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoppedWhen::Now => write!(f, "Now"),
            StoppedWhen::ManualTime => write!(f, "Enter a Time"),
        }
    }
}

enum YesOrNo {
    Yes,
    No,
}

impl Display for YesOrNo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YesOrNo::Yes => write!(f, "Yes"),
            YesOrNo::No => write!(f, "No"),
        }
    }
}

async fn ask_when_started_and_stopped(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    when_selected: &DateTime<Utc>,
    bullet_list_created: &DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), ()> {
    let when_last_time_finished = get_when_the_last_item_finished(send_to_data_storage_layer).await;

    let mut started_when = Vec::default();

    if let Some(when_last_time_finished) = when_last_time_finished {
        started_when.push(StartedWhen::WhenLastItemFinished(
            when_last_time_finished.into(),
        ));
    }

    started_when.push(StartedWhen::WhenBulletListWasFirstShown(
        (*bullet_list_created).into(),
    ));
    started_when.push(StartedWhen::WhenThisItemWasSelected(
        (*when_selected).into(),
    ));
    started_when.push(StartedWhen::CustomTime);

    loop {
        let started_when = Select::new(
            "When did you start working on this item?",
            started_when.clone(),
        )
        .prompt();
        let when_started = match started_when {
            Ok(StartedWhen::WhenLastItemFinished(when_last_time_finished)) => {
                when_last_time_finished
            }
            Ok(StartedWhen::WhenThisItemWasSelected(when_selected)) => when_selected,
            Ok(StartedWhen::WhenBulletListWasFirstShown(when_bullet_list_was_first_shown)) => {
                when_bullet_list_was_first_shown
            }
            Ok(StartedWhen::CustomTime) => {
                match Text::new("Enter how long ago or the exact time when this item was started.")
                    .prompt()
                {
                    Ok(when_started) => match parse(&when_started) {
                        Ok(duration) => {
                            let when_started = now - duration;
                            when_started.into()
                        }
                        Err(_) => match dateparser::parse(&when_started) {
                            Ok(when_started) => when_started.into(),
                            Err(_) => {
                                println!("Invalid input. Please try again.");
                                continue;
                            }
                        },
                    },
                    Err(InquireError::OperationCanceled) => continue,
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Err(InquireError::OperationCanceled) => {
                todo!("Operation Canceled")
            }
            Err(InquireError::OperationInterrupted) => {
                return Err(());
            }
            Err(err) => todo!("{:?}", err),
        };
        println!("When started: {:?}", when_started);
        let stopped_when = vec![StoppedWhen::Now, StoppedWhen::ManualTime];
        let stopped_when =
            Select::new("When did you stop working on this item?", stopped_when).prompt();
        let when_stopped = match stopped_when {
            Ok(StoppedWhen::Now) => Local::now(),
            Ok(StoppedWhen::ManualTime) => {
                match Text::new("Enter how long ago or the exact time when this item was stopped.")
                    .prompt()
                {
                    Ok(when_stopped) => match parse(&when_stopped) {
                        Ok(duration) => {
                            let when_stopped = now - duration;
                            when_stopped.into()
                        }
                        Err(_) => match dateparser::parse(&when_stopped) {
                            Ok(when_stopped) => when_stopped.into(),
                            Err(_) => {
                                println!("Invalid input. Please try again.");
                                continue;
                            }
                        },
                    },
                    Err(InquireError::OperationCanceled) => continue,
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Err(InquireError::OperationCanceled) => continue,
            Err(InquireError::OperationInterrupted) => return Err(()),
            Err(err) => todo!("{:?}", err),
        };

        if when_started > when_stopped {
            println!("The time started is after the time stopped. Please try again.");
            continue;
        }
        let time_spent = when_stopped - when_started;
        let duration: Duration = Duration::from_secs(time_spent.num_seconds() as u64);
        let display_duration = DisplayDuration::new(&duration);
        println!("Time spent: {}", display_duration);
        if time_spent.num_hours() > 2 {
            let confirm = Select::new(
                &format!(
                    "The amount of time spent is {display_duration}. Are you sure this is correct?"
                ),
                vec![YesOrNo::Yes, YesOrNo::No],
            )
            .prompt();
            match confirm {
                Ok(YesOrNo::Yes) => {}
                Ok(YesOrNo::No) => continue,
                Err(InquireError::OperationCanceled) => continue,
                Err(InquireError::OperationInterrupted) => return Err(()),
                Err(err) => todo!("{:?}", err),
            }
        }

        return Ok((when_started.into(), when_stopped.into()));
    }
}

async fn get_when_the_last_item_finished(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Option<DateTime<Utc>> {
    let (sender, receiver) = oneshot::channel();
    send_to_data_storage_layer
        .send(DataLayerCommands::SendTimeSpentLog(sender))
        .await
        .unwrap();
    let time_spent_log = receiver.await.unwrap();
    let mut earliest: Option<Datetime> = None;
    for entry in time_spent_log {
        if let Some(e) = &earliest {
            if *e < entry.when_stopped {
                earliest = Some(entry.when_stopped);
            }
        } else {
            earliest = Some(entry.when_stopped);
        }
    }

    earliest.map(|earliest| earliest.into())
}

enum Dedication {
    Primary,
    Secondary,
}

impl Display for Dedication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dedication::Primary => write!(f, "Primary or Main Task"),
            Dedication::Secondary => write!(f, "Secondary or Background Task"),
        }
    }
}

fn ask_about_dedication() -> Result<Option<SurrealDedication>, ()> {
    let dedication = vec![Dedication::Primary, Dedication::Secondary];
    let dedication = Select::new("What is the dedication of this time spent?", dedication).prompt();
    match dedication {
        Ok(Dedication::Primary) => Ok(Some(SurrealDedication::PrimaryTask)),
        Ok(Dedication::Secondary) => Ok(Some(SurrealDedication::SecondaryTask)),
        Err(InquireError::OperationCanceled) => Ok(None),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}
