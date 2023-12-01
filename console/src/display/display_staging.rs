use std::fmt::Display;

use chrono::{DateTime, Local, Utc};

use crate::surrealdb_layer::surreal_item::Staging;

pub(crate) struct DisplayStaging<'s> {
    staging: &'s Staging,
}

impl Display for DisplayStaging<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.staging {
            Staging::Intension => write!(f, "Intension"),
            Staging::NotSet => write!(f, "NotSet"),
            Staging::Released => write!(f, "Released"),
            Staging::MentallyResident {
                last_worked_on,
                work_on_again_before,
            } => {
                let last_worked_on: DateTime<Utc> = last_worked_on.clone().into();
                let last_worked_on: DateTime<Local> = last_worked_on.into();
                let work_on_again_before: DateTime<Utc> = work_on_again_before.clone().into();
                let work_on_again_before: DateTime<Local> = work_on_again_before.into();
                write!(
                    f,
                    "MentallyResident: last_worked_on: {}, work_on_again_before: {}",
                    last_worked_on.naive_local(),
                    work_on_again_before.naive_local()
                )
            }
            Staging::OnDeck {
                began_waiting,
                can_wait_until,
            } => {
                let began_waiting: DateTime<Utc> = began_waiting.clone().into();
                let began_waiting: DateTime<Local> = began_waiting.into();
                let can_wait_until: DateTime<Utc> = can_wait_until.clone().into();
                let can_wait_until: DateTime<Local> = can_wait_until.into();
                write!(
                    f,
                    "OnDeck: began_waiting: {}, can_wait_until: {}",
                    began_waiting.naive_local(),
                    can_wait_until.naive_local()
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
