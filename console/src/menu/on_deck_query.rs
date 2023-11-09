use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};

use crate::{menu::YesOrNo, surrealdb_layer::surreal_item::Staging};

pub(crate) async fn on_deck_query() -> Result<Staging, InquireError> {
    let now = Local::now();
    let wait_until = loop {
        let wait_string = Text::new("Can wait for how long?").prompt()?;
        let wait_duration = parse(&wait_string).unwrap();
        let wait_until = now + wait_duration;
        println!("Can wait until {}?", wait_until);
        let result = Select::new("", YesOrNo::make_list()).prompt()?;
        match result {
            YesOrNo::Yes => break wait_until,
            YesOrNo::No => continue,
        }
    };

    let now: DateTime<Utc> = now.into();
    let wait_until: DateTime<Utc> = wait_until.into();
    Ok(Staging::OnDeck {
        began_waiting: now.into(),
        can_wait_until: wait_until.into(),
    })
}
