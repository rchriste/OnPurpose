use std::fmt::Display;

use crate::{
    display::{
        display_enter_list_reason::DisplayEnterListReason, display_surreal_lap::DisplaySurrealLap,
    },
    surrealdb_layer::surreal_item::Staging,
};

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
            Staging::MentallyResident { enter_list, lap } => {
                let enter_list = DisplayEnterListReason::new(enter_list);
                let lap = DisplaySurrealLap::new(lap);
                write!(f, "🧠 MentallyResident: lap: {lap}, start: {enter_list}",)
            }
            Staging::OnDeck { enter_list, lap } => {
                let enter_list = DisplayEnterListReason::new(enter_list);
                let lap = DisplaySurrealLap::new(lap);
                write!(f, "🪜 OnDeck: lap: {lap}, start: {enter_list}",)
            }
        }
    }
}

impl<'s> DisplayStaging<'s> {
    pub(crate) fn new(staging: &'s Staging) -> Self {
        DisplayStaging { staging }
    }
}
