use std::{fmt::Display, time::Duration};

use crate::{
    display::{
        display_duration::DisplayDuration, display_enter_list_reason::DisplayEnterListReason,
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
                let lap: Duration = lap.clone().into();
                let lap = DisplayDuration::new(&lap);
                write!(f, "MentallyResident: enter_list: {enter_list}, lap: {lap}",)
            }
            Staging::OnDeck { enter_list, lap } => {
                let enter_list = DisplayEnterListReason::new(enter_list);
                let lap: Duration = lap.clone().into();
                let lap = DisplayDuration::new(&lap);
                write!(f, "OnDeck: enter_list: {enter_list}, lap: {lap}",)
            }
        }
    }
}

impl<'s> DisplayStaging<'s> {
    pub(crate) fn new(staging: &'s Staging) -> Self {
        DisplayStaging { staging }
    }
}
