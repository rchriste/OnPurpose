use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb_extra::table::Table;

use crate::new_time_spent::NewTimeSpent;

#[derive(PartialEq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "time_spent_log")]
pub(crate) struct SurrealTimeSpent {
    pub(crate) id: Option<Thing>,
    pub(crate) version: u32,
    pub(crate) working_on: Vec<Thing>,
    pub(crate) bullet_list_position: Option<SurrealBulletListPosition>,
    pub(crate) when_started: Datetime,
    pub(crate) when_stopped: Datetime,
    pub(crate) dedication: SurrealDedication,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealDedication {
    PrimaryTask,
    SecondaryTask,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealBulletListPosition {
    pub(crate) position_in_list: u64,
    pub(crate) lap_count: f32,
    pub(crate) next_lower_lap_count: Option<f32>,
    pub(crate) next_higher_lap_count: Option<f32>,
}

impl From<NewTimeSpent> for SurrealTimeSpent {
    fn from(new_time_spent: NewTimeSpent) -> Self {
        SurrealTimeSpent {
            id: None,
            version: 0,
            working_on: new_time_spent.working_on,
            bullet_list_position: new_time_spent.bullet_list_position,
            when_started: new_time_spent.when_started.into(),
            when_stopped: new_time_spent.when_stopped.into(),
            dedication: new_time_spent.dedication,
        }
    }
}

#[derive(PartialEq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "time_spent_log")]
pub(crate) struct SurrealTimeSpentPreviousVersion {
    pub(crate) id: Option<Thing>,
    pub(crate) version: u32,
    pub(crate) working_on: Vec<Thing>,
    pub(crate) position_in_list: u64,
    pub(crate) lap_count: f32,
    pub(crate) next_lower_lap_count: Option<f32>,
    pub(crate) next_higher_lap_count: Option<f32>,
    pub(crate) when_started: Datetime,
    pub(crate) when_stopped: Datetime,
    pub(crate) dedication: SurrealDedication,
}
