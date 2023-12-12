use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};

use crate::{menu::YesOrNo, surrealdb_layer::surreal_item::Staging};

pub(crate) async fn on_deck_query() -> Result<Staging, InquireError> {
    let (return_to, work_on_again_before) = prompt_for_two_times()?;

    Ok(Staging::OnDeck {
        began_waiting: return_to.into(),
        can_wait_until: work_on_again_before.into(),
    })
}

pub(crate) async fn mentally_resident_query() -> Result<Staging, InquireError> {
    let (return_to, work_on_again_before) = prompt_for_two_times()?;

    Ok(Staging::MentallyResident {
        last_worked_on: return_to.into(),
        work_on_again_before: work_on_again_before.into(),
    })
}

fn prompt_for_two_times() -> Result<(DateTime<Utc>, DateTime<Utc>), InquireError> {
    let now = Local::now();
    loop {
        let return_to_string =
            Text::new("Wait how long before returning the item to the list?").prompt()?;
        let return_to_duration = parse(&return_to_string).unwrap();
        let return_to = now + return_to_duration;
        let deadline_string = Text::new("Then how long until the item expires?").prompt()?;
        let deadline_duration = parse(&deadline_string).unwrap();
        let work_on_again_before = now + return_to_duration + deadline_duration;
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
