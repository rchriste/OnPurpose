use std::fmt::Display;

use crate::{
    display::display_duration::DisplayDuration, surrealdb_layer::surreal_item::SurrealLap,
};

pub(crate) struct DisplaySurrealLap<'s> {
    surreal_lap: &'s SurrealLap,
}

impl Display for DisplaySurrealLap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.surreal_lap {
            SurrealLap::AlwaysTimer(duration) => {
                let duration = DisplayDuration::new(duration);
                write!(f, "Always Timer Duration: {duration}")
            }
            SurrealLap::WorkedOnCounter { stride } => {
                write!(f, "Stride Counter: 1 / {stride}")
            }
        }
    }
}

impl<'s> DisplaySurrealLap<'s> {
    pub(crate) fn new(surreal_lap: &'s SurrealLap) -> Self {
        DisplaySurrealLap { surreal_lap }
    }
}
