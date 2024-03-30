use async_recursion::async_recursion;
use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use surrealdb::{opt::RecordId, sql::Datetime};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    base_data::item::Item,
    menu::staging_query::{mentally_resident_query, on_deck_query},
    new_time_spent::NewTimeSpent,
    node::{item_status::ItemStatus, Filter},
    surrealdb_layer::{
        surreal_item::{Responsibility, Staging},
        surreal_time_spent::SurrealDedication,
        DataLayerCommands,
    },
    systems::bullet_list::BulletListReason,
};
use inquire::{InquireError, Select, Text};
use std::fmt::Display;

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum StagingMenuSelection {
    NotSet,
    MentallyResident,
    OnDeck,
    Planned,
    ThinkingAbout,
    Released,
    MakeItemReactive,
}

impl Display for StagingMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StagingMenuSelection::NotSet => write!(f, "Not Set"),
            StagingMenuSelection::MentallyResident => write!(f, "Mentally Resident"),
            StagingMenuSelection::OnDeck => write!(f, "On Deck"),
            StagingMenuSelection::Planned => write!(f, "Planned"),
            StagingMenuSelection::ThinkingAbout => write!(f, "Thinking About"),
            StagingMenuSelection::Released => write!(f, "Released"),
            StagingMenuSelection::MakeItemReactive => write!(f, "Make Item Reactive"),
        }
    }
}

impl StagingMenuSelection {
    /// Returns a tuple of the list and the default index or recommended default selection
    pub(crate) fn make_list(default_selection: Option<StagingMenuSelection>) -> (Vec<Self>, usize) {
        let choices = vec![
            StagingMenuSelection::MentallyResident,
            StagingMenuSelection::OnDeck,
            StagingMenuSelection::Planned,
            StagingMenuSelection::ThinkingAbout,
            StagingMenuSelection::Released,
            StagingMenuSelection::NotSet,
            StagingMenuSelection::MakeItemReactive,
        ];
        let default_index = match default_selection {
            Some(default_selection) => choices
                .iter()
                .position(|choice| choice == &default_selection)
                .unwrap(),
            None => 1,
        };

        (choices, default_index)
    }
}

pub(crate) async fn log_worked_on_this(
    selected: &ItemStatus<'_>,
    when_selected: DateTime<Utc>,
    now: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    ordered_bullet_list: &[BulletListReason<'_>],
) -> Result<(), ()> {
    // This logs time spent on an item with the goal of in the future making it possible for the user to adjust items and balance
    // Logs the following:
    // When starting to work on item
    // -Position in list
    let position_in_list = ordered_bullet_list
        .iter()
        .position(|reason| reason.get_surreal_record_id() == selected.get_surreal_record_id())
        .unwrap();

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

    let working_on = create_working_on_list(selected);
    // -When started
    let (when_started, when_stopped) =
        ask_when_started_and_stopped(send_to_data_storage_layer, when_selected, now).await?;
    // -When marked "I worked on this"
    // -How much time spent, show amount of time since started and show amount of time since last item completed, or allow user to enter a duration
    let dedication = ask_about_dedication()?;
    let time_spent = NewTimeSpent {
        working_on,
        position_in_list: position_in_list as u64,
        lap_count,
        next_lower_lap_count,
        next_higher_lap_count,
        when_started,
        when_stopped,
        dedication,
    };
    send_to_data_storage_layer
        .send(DataLayerCommands::RecordTimeSpent(time_spent))
        .await
        .unwrap();
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
    WhenThisItemWasSelected(DateTime<Local>),
    WhenLastItemFinished(DateTime<Local>),
    ManualTime,
}

impl Display for StartedWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartedWhen::WhenThisItemWasSelected(when_selected) => {
                write!(f, "When this item was selected (i.e. {})", when_selected)
            }
            StartedWhen::WhenLastItemFinished(when_last_item_finished) => {
                write!(
                    f,
                    "When the last item finished (i.e. {})",
                    when_last_item_finished
                )
            }
            StartedWhen::ManualTime => write!(f, "Manual Time"),
        }
    }
}

enum StoppedWhen {
    Now(DateTime<Local>),
    ManualTime,
}

impl Display for StoppedWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoppedWhen::Now(now) => write!(f, "Now (i.e. {})", now),
            StoppedWhen::ManualTime => write!(f, "Manual Time"),
        }
    }
}

async fn ask_when_started_and_stopped(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    when_selected: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), ()> {
    let when_last_time_finished = get_when_the_last_item_finished(send_to_data_storage_layer).await;

    let mut started_when = Vec::default();

    if let Some(when_last_time_finished) = when_last_time_finished {
        started_when.push(StartedWhen::WhenLastItemFinished(
            when_last_time_finished.into(),
        ));
    }

    started_when.push(StartedWhen::WhenThisItemWasSelected(when_selected.into()));
    started_when.push(StartedWhen::ManualTime);

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
            Ok(StartedWhen::ManualTime) => {
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
        let stopped_when = vec![StoppedWhen::Now(now.into()), StoppedWhen::ManualTime];
        let stopped_when =
            Select::new("When did you stop working on this item?", stopped_when).prompt();
        let when_stopped = match stopped_when {
            Ok(StoppedWhen::Now(now)) => now,
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
            Err(InquireError::OperationCanceled) => {
                todo!("Operation Canceled")
            }
            Err(InquireError::OperationInterrupted) => {
                return Err(());
            }
            Err(err) => todo!("{:?}", err),
        };

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

fn ask_about_dedication() -> Result<SurrealDedication, ()> {
    let dedication = vec![Dedication::Primary, Dedication::Secondary];
    let dedication = Select::new("What is the dedication of this time spent?", dedication).prompt();
    match dedication {
        Ok(Dedication::Primary) => Ok(SurrealDedication::PrimaryTask),
        Ok(Dedication::Secondary) => Ok(SurrealDedication::SecondaryTask),
        Err(InquireError::OperationCanceled) => {
            todo!("Operation Canceled")
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("{:?}", err),
    }
}

#[async_recursion]
pub(crate) async fn present_set_staging_menu(
    selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    default_selection: Option<StagingMenuSelection>,
) -> Result<(), ()> {
    let staging = loop {
        let (list, starting_cursor) = StagingMenuSelection::make_list(default_selection);

        let selection = Select::new("Select from the below list|", list)
            .with_starting_cursor(starting_cursor)
            .prompt();
        let staging = match selection {
            Ok(StagingMenuSelection::NotSet) => Staging::NotSet,
            Ok(StagingMenuSelection::MentallyResident) => {
                let result = mentally_resident_query().await;
                match result {
                    Ok(mentally_resident) => mentally_resident,
                    Err(InquireError::OperationCanceled) => {
                        return present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        )
                        .await
                    }
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Ok(StagingMenuSelection::OnDeck) => {
                let result = on_deck_query().await;
                match result {
                    Ok(staging) => staging,
                    Err(InquireError::OperationCanceled) => {
                        return present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        )
                        .await
                    }
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Ok(StagingMenuSelection::Planned) => Staging::Planned,
            Ok(StagingMenuSelection::ThinkingAbout) => Staging::ThinkingAbout,
            Ok(StagingMenuSelection::Released) => Staging::Released,
            Ok(StagingMenuSelection::MakeItemReactive) => {
                send_to_data_storage_layer
                    .send(DataLayerCommands::UpdateItemResponsibility(
                        selected.get_surreal_record_id().clone(),
                        Responsibility::ReactiveBeAvailableToAct,
                    ))
                    .await
                    .unwrap();
                return Ok(());
            }
            Err(InquireError::OperationInterrupted) => return Err(()),
            Err(InquireError::OperationCanceled) => {
                // Just continue because we don't know exactly what to go back to
                return Ok(());
            }
            Err(err) => todo!("{:?}", err),
        };
        break staging;
    };

    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemStaging(
            selected.get_surreal_record_id().clone(),
            staging,
        ))
        .await
        .unwrap();
    Ok(())
}
