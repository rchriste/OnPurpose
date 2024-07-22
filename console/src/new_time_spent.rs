use chrono::{DateTime, Utc};

use crate::surrealdb_layer::{
    surreal_in_the_moment_priority::SurrealAction, surreal_item::SurrealUrgency,
    surreal_time_spent::SurrealDedication,
};

pub(crate) struct NewTimeSpent {
    pub(crate) working_on: Vec<SurrealAction>,
    pub(crate) urgency: Option<SurrealUrgency>,
    pub(crate) when_started: DateTime<Utc>,
    pub(crate) when_stopped: DateTime<Utc>,
    pub(crate) dedication: Option<SurrealDedication>,
}
