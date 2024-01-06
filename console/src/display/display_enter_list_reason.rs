use std::fmt::Display;

use chrono::{DateTime, Local, Utc};

use crate::surrealdb_layer::surreal_item::EnterListReason;

pub(crate) struct DisplayEnterListReason<'e> {
    enter_list_reason: &'e EnterListReason,
}

impl Display for DisplayEnterListReason<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.enter_list_reason {
            EnterListReason::DateTime(datetime) => {
                let datetime: DateTime<Utc> = datetime.clone().into();
                let datetime: DateTime<Local> = datetime.into();
                write!(f, "After {}", datetime.naive_local())
            }
            EnterListReason::HighestUncovered {
                earliest: _earliest,
                review_after,
            } => {
                let datetime: DateTime<Utc> = review_after.clone().into();
                let datetime: DateTime<Local> = datetime.into();
                //No need to display earliest because that is not something that the user needs to be concerned about as it is to cover an edge case
                write!(
                    f,
                    "When highest priority uncovered or review after {}",
                    datetime.naive_local()
                )
            }
        }
    }
}

impl<'s> DisplayEnterListReason<'s> {
    pub(crate) fn new(enter_list_reason: &'s EnterListReason) -> Self {
        DisplayEnterListReason { enter_list_reason }
    }
}
