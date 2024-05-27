use std::fmt::Display;

use crate::{
    display::{
        display_enter_list_reason::DisplayEnterListReason, display_surreal_lap::DisplaySurrealLap,
    },
    surrealdb_layer::surreal_item::{InRelationToRatioType, SurrealStaging},
};

pub(crate) struct DisplayStaging<'s> {
    staging: &'s SurrealStaging,
}

impl Display for DisplayStaging<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.staging {
            SurrealStaging::Planned => write!(f, "Planned"),
            SurrealStaging::ThinkingAbout => write!(f, "Thinking about"),
            SurrealStaging::NotSet => write!(f, "Not set"),
            SurrealStaging::Released => write!(f, "Released"),
            SurrealStaging::MentallyResident { enter_list, lap } => {
                let enter_list = DisplayEnterListReason::new(enter_list);
                let lap = DisplaySurrealLap::new(lap);
                write!(f, "🧠 ⮔ {lap}, ▶️ {enter_list}",)
            }
            SurrealStaging::OnDeck { enter_list, lap } => {
                let enter_list = DisplayEnterListReason::new(enter_list);
                let lap = DisplaySurrealLap::new(lap);
                write!(f, "🔜 ⮔ {lap}, ▶️ {enter_list}",)
            }
            SurrealStaging::InRelationTo {
                other_item,
                ratio,
                start: _start,
            } => {
                write!(f, "🔗 ")?;
                match ratio {
                    InRelationToRatioType::AmountOfTimeSpent { multiplier } => {
                        write!(f, "⌚ {:?}:1", multiplier)?
                    }
                    InRelationToRatioType::IterationCount { multiplier } => {
                        write!(f, "1 / {:?}", 1f32 / multiplier)?
                    }
                };
                write!(f, " in relation to {:?}", other_item)
            }
        }
    }
}

impl<'s> DisplayStaging<'s> {
    pub(crate) fn new(staging: &'s SurrealStaging) -> Self {
        DisplayStaging { staging }
    }
}
