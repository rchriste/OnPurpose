use chrono::{DateTime, Utc};
use surrealdb::opt::RecordId;

use crate::surrealdb_layer::surreal_time_spent::SurrealDedication;

pub(crate) struct NewTimeSpent {
    pub(crate) working_on: Vec<RecordId>,
    pub(crate) position_in_list: u64,
    pub(crate) lap_count: f32,
    pub(crate) next_lower_lap_count: Option<f32>,
    pub(crate) next_higher_lap_count: Option<f32>,
    pub(crate) when_started: DateTime<Utc>,
    pub(crate) when_stopped: DateTime<Utc>,
    pub(crate) dedication: SurrealDedication,
}
