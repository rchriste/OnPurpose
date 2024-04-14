use std::fmt::Display;

use chrono::Utc;
use duration_str::parse;
use inquire::{InquireError, Select, Text};

use crate::{
    display::{
        display_enter_list_reason::DisplayEnterListReason, display_surreal_lap::DisplaySurrealLap,
    },
    menu::YesOrNo,
    surrealdb_layer::surreal_item::{EnterListReason, Staging, SurrealLap},
};

#[derive(Debug, Clone)]
enum EnterListReasonSelection {
    Immediately,
    DateTime,
    HighestUncovered,
}

impl Display for EnterListReasonSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnterListReasonSelection::Immediately => write!(f, "Immediately"),
            EnterListReasonSelection::DateTime => write!(f, "Wait an amount of time"),
            EnterListReasonSelection::HighestUncovered => {
                write!(f, "Once this is highest priority uncovered")
            }
        }
    }
}

impl EnterListReasonSelection {
    fn make_list_on_deck() -> Vec<Self> {
        vec![
            EnterListReasonSelection::DateTime,
            EnterListReasonSelection::Immediately,
            EnterListReasonSelection::HighestUncovered,
        ]
    }

    fn make_list_mentally_resident() -> Vec<Self> {
        vec![
            EnterListReasonSelection::DateTime,
            EnterListReasonSelection::Immediately,
        ]
    }
}

pub(crate) async fn on_deck_query() -> Result<Staging, InquireError> {
    let (enter_list, lap) = prompt_for_two_times(EnterListReasonSelection::make_list_on_deck())?;

    Ok(Staging::OnDeck { enter_list, lap })
}

pub(crate) async fn mentally_resident_query() -> Result<Staging, InquireError> {
    let (enter_list, lap) =
        prompt_for_two_times(EnterListReasonSelection::make_list_mentally_resident())?;

    Ok(Staging::MentallyResident { enter_list, lap })
}

fn prompt_for_two_times(
    list: Vec<EnterListReasonSelection>,
) -> Result<(EnterListReason, SurrealLap), InquireError> {
    let now = Utc::now();
    loop {
        let selection =
            Select::new("When should this item enter the list?", list.clone()).prompt()?;
        let enter_list_reason = match selection {
            EnterListReasonSelection::Immediately => EnterListReason::DateTime(now.into()),
            EnterListReasonSelection::DateTime => {
                let return_to_string =
                    Text::new("Wait how long before returning the item to the list?").prompt()?;
                match parse(&return_to_string) {
                    Ok(return_to_duration) => {
                        let return_to = now + return_to_duration;
                        EnterListReason::DateTime(return_to.into())
                    }
                    Err(_) => match dateparser::parse(&return_to_string) {
                        Ok(return_to) => {
                            if return_to < now {
                                println!("Cannot give a time in the past. Please try again.");
                                continue;
                            } else {
                                EnterListReason::DateTime(return_to.into())
                            }
                        }
                        Err(_) => {
                            println!("Invalid input. Please try again.");
                            continue;
                        }
                    },
                }
            }
            EnterListReasonSelection::HighestUncovered => {
                let review_after =
                    Text::new("What is the maximum amount of time to wait?").prompt()?;
                match parse(&review_after) {
                    Ok(review_after_duration) => {
                        let review_after = now + review_after_duration;
                        EnterListReason::HighestUncovered {
                            earliest: now.into(),
                            review_after: review_after.into(),
                        }
                    }
                    Err(_) => match dateparser::parse(&review_after) {
                        Ok(review_after) => EnterListReason::HighestUncovered {
                            earliest: now.into(),
                            review_after: review_after.into(),
                        },
                        Err(_) => {
                            println!("Invalid input. Please try again.");
                            continue;
                        }
                    },
                }
            }
        };
        if let Some(lap) = prompt_for_surreal_lap()? {
            let result = Select::new(
                &format!(
                    "Wait until: {}\n Lap: {}?",
                    DisplayEnterListReason::new(&enter_list_reason),
                    DisplaySurrealLap::new(&lap)
                ),
                YesOrNo::make_list(),
            )
            .prompt()?;
            match result {
                YesOrNo::Yes => return Ok((enter_list_reason, lap)),
                YesOrNo::No => continue,
            }
        } else {
            continue;
        }
    }
}

enum LapSelection {
    WallClockTimer,
    WorkedOnCounter,
}

impl Display for LapSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LapSelection::WallClockTimer => write!(f, "Wall clock timer"),
            LapSelection::WorkedOnCounter => write!(f, "Times worked on counter"),
        }
    }
}

pub(crate) fn prompt_for_surreal_lap() -> Result<Option<SurrealLap>, InquireError> {
    let selection = Select::new(
        "How should the lap be measured?",
        vec![LapSelection::WallClockTimer, LapSelection::WorkedOnCounter],
    )
    .prompt()?;
    match selection {
        LapSelection::WallClockTimer => {
            let deadline_string = Text::new("Lap length?").prompt()?;
            match parse(deadline_string) {
                Ok(lap) => Ok(Some(SurrealLap::AlwaysTimer(lap.into()))),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    Ok(None)
                }
            }
        }
        LapSelection::WorkedOnCounter => {
            let stride = Text::new("Stride?").prompt()?;
            match stride.parse::<u32>() {
                Ok(stride) => Ok(Some(SurrealLap::WorkedOnCounter { stride })),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    Ok(None)
                }
            }
        }
    }
}
