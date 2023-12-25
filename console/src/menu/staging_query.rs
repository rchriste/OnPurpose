use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};

use crate::{menu::YesOrNo, surrealdb_layer::surreal_item::Staging};

pub(crate) async fn on_deck_query() -> Result<Staging, InquireError> {
    let (enter_list, finish_first_lap) = prompt_for_two_times()?;

    Ok(Staging::OnDeck {
        enter_list: enter_list.into(),
        finish_first_lap: finish_first_lap.into(),
    })
}

pub(crate) async fn mentally_resident_query() -> Result<Staging, InquireError> {
    let (enter_list, finish_first_lap) = prompt_for_two_times()?;

    Ok(Staging::MentallyResident {
        enter_list: enter_list.into(),
        finish_first_lap: finish_first_lap.into(),
    })
}

fn prompt_for_two_times() -> Result<(DateTime<Utc>, DateTime<Utc>), InquireError> {
    let now = Local::now();
    loop {
        let return_to_string =
            Text::new("Wait how long before returning the item to the list?").prompt()?;
        let return_to = match parse(&return_to_string) {
            Ok(return_to_duration) => now + return_to_duration,
            Err(_) => match dateparser::parse(&return_to_string) {
                Ok(return_to) => return_to.into(),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    continue;
                }
            },
        };
        let deadline_string = Text::new("Lap priority length?").prompt()?;
        let work_on_again_before = match parse(&deadline_string) {
            Ok(deadline_duration) => return_to + deadline_duration,
            Err(_) => match dateparser::parse(&deadline_string) {
                Ok(work_on_again_before) => work_on_again_before.into(),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    continue;
                }
            },
        };
        let result = Select::new(
            &format!(
                "Wait until {}\n Expires at {}?",
                return_to, work_on_again_before
            ),
            YesOrNo::make_list(),
        )
        .prompt()?;
        match result {
            YesOrNo::Yes => return Ok((return_to.into(), work_on_again_before.into())),
            YesOrNo::No => continue,
        }
    }
}
