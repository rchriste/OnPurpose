use chrono::{DateTime, Utc};
use surrealdb::opt::RecordId;

use crate::surrealdb_layer::surreal_time_spent::{SurrealBulletListPosition, SurrealDedication};

pub(crate) struct NewTimeSpent {
    pub(crate) working_on: Vec<RecordId>,
    pub(crate) bullet_list_position: Option<SurrealBulletListPosition>,
    pub(crate) when_started: DateTime<Utc>,
    pub(crate) when_stopped: DateTime<Utc>,
    pub(crate) dedication: SurrealDedication,
}
