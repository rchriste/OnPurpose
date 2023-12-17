use std::fmt::Display;

use chrono::{DateTime, Local, Utc};

use crate::surrealdb_layer::surreal_item::Staging;

pub(crate) struct DisplayStaging<'s> {
    staging: &'s Staging,
}

impl Display for DisplayStaging<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.staging {
            Staging::Planned => write!(f, "Planned"),
            Staging::ThinkingAbout => write!(f, "Thinking about"),
            Staging::NotSet => write!(f, "Not set"),
            Staging::Released => write!(f, "Released"),
            Staging::MentallyResident {
                enter_list,
                finish_first_lap,
            } => {
                let enter_list: DateTime<Utc> = enter_list.clone().into();
                let enter_list: DateTime<Local> = enter_list.into();
                let finish_first_lap: DateTime<Utc> = finish_first_lap.clone().into();
                let finish_first_lap: DateTime<Local> = finish_first_lap.into();
                write!(
                    f,
                    "MentallyResident: enter_list: {}, finish_first_lap: {}",
                    enter_list.naive_local(),
                    finish_first_lap.naive_local()
                )
            }
            Staging::OnDeck {
                enter_list,
                finish_first_lap,
            } => {
                let enter_list: DateTime<Utc> = enter_list.clone().into();
                let enter_list: DateTime<Local> = enter_list.into();
                let finish_first_lap: DateTime<Utc> = finish_first_lap.clone().into();
                let finish_first_lap: DateTime<Local> = finish_first_lap.into();
                write!(
                    f,
                    "OnDeck: enter_list: {}, finish_first_lap: {}",
                    enter_list.naive_local(),
                    finish_first_lap.naive_local()
                )
            }
        }
    }
}

impl<'s> DisplayStaging<'s> {
    pub(crate) fn new(staging: &'s Staging) -> Self {
        DisplayStaging { staging }
    }
}
