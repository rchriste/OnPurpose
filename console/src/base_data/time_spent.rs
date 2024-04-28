use chrono::{DateTime, Utc};

use crate::surrealdb_layer::surreal_time_spent::SurrealTimeSpent;

pub(crate) struct TimeSpent<'s> {
    surreal_time_spent: &'s SurrealTimeSpent,
}

impl<'s> TimeSpent<'s> {
    pub(crate) fn new(surreal_time_spent: &'s SurrealTimeSpent) -> TimeSpent<'s> {
        TimeSpent { surreal_time_spent }
    }

    pub(crate) fn get_started_at(&self) -> &DateTime<Utc> {
        &self.surreal_time_spent.when_started
    }
}