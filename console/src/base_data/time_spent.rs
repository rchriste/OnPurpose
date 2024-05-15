use chrono::{DateTime, TimeDelta, Utc};
use surrealdb::opt::RecordId;

use crate::surrealdb_layer::surreal_time_spent::SurrealTimeSpent;

pub(crate) struct TimeSpent<'s> {
    surreal_time_spent: &'s SurrealTimeSpent,
    when_started: DateTime<Utc>,
    when_stopped: DateTime<Utc>,
}

impl<'s> TimeSpent<'s> {
    pub(crate) fn new(surreal_time_spent: &'s SurrealTimeSpent) -> TimeSpent<'s> {
        TimeSpent {
            surreal_time_spent,
            when_started: surreal_time_spent.when_started.clone().into(),
            when_stopped: surreal_time_spent.when_stopped.clone().into(),
        }
    }

    pub(crate) fn get_started_at(&self) -> &DateTime<Utc> {
        &self.surreal_time_spent.when_started
    }

    pub(crate) fn get_time_delta(&self) -> TimeDelta {
        let when_started: DateTime<Utc> = self.surreal_time_spent.when_started.clone().into();
        self.surreal_time_spent
            .when_stopped
            .signed_duration_since(when_started)
    }

    pub(crate) fn is_within(&self, start: &DateTime<Utc>, end: &DateTime<Utc>) -> bool {
        self.when_started >= *start && self.when_stopped <= *end
    }

    pub(crate) fn worked_towards(&self) -> &Vec<RecordId> {
        &self.surreal_time_spent.working_on
    }
}
