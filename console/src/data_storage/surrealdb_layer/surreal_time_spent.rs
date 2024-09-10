use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use crate::new_time_spent::NewTimeSpent;

use super::{surreal_in_the_moment_priority::SurrealAction, surreal_item::SurrealUrgency};

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealTimeSpent {
    pub(crate) id: Option<Thing>,
    pub(crate) version: u32,
    pub(crate) working_on: Vec<SurrealAction>,
    pub(crate) urgency: Option<SurrealUrgency>,
    pub(crate) when_started: Datetime,
    pub(crate) when_stopped: Datetime,
    pub(crate) dedication: Option<SurrealDedication>,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealDedication {
    PrimaryTask,
    BackgroundTask,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealDedicationOld {
    PrimaryTask,
    SecondaryTask,
}

impl From<NewTimeSpent> for SurrealTimeSpent {
    fn from(new_time_spent: NewTimeSpent) -> Self {
        SurrealTimeSpent {
            id: None,
            version: 0,
            working_on: new_time_spent.working_on,
            when_started: new_time_spent.when_started.into(),
            when_stopped: new_time_spent.when_stopped.into(),
            dedication: new_time_spent.dedication,
            urgency: new_time_spent.urgency,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealBulletListPositionOld {
    pub(crate) position_in_list: u64,
    pub(crate) lap_count: f32,
    pub(crate) next_lower_lap_count: Option<f32>,
    pub(crate) next_higher_lap_count: Option<f32>,
}

impl SurrealTimeSpent {
    pub(crate) const TABLE_NAME: &'static str = "time_spent_log";
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealTimeSpentOldVersion {
    pub(crate) id: Option<Thing>,
    pub(crate) version: u32,
    pub(crate) working_on: Vec<Thing>,
    pub(crate) bullet_list_position: Option<SurrealBulletListPositionOld>,
    pub(crate) when_started: Datetime,
    pub(crate) when_stopped: Datetime,
    pub(crate) dedication: SurrealDedicationOld,
}

impl From<SurrealTimeSpentOldVersion> for SurrealTimeSpent {
    fn from(old_version: SurrealTimeSpentOldVersion) -> Self {
        SurrealTimeSpent {
            id: old_version.id,
            version: old_version.version,
            working_on: old_version
                .working_on
                .into_iter()
                .map(SurrealAction::MakeProgress)
                .collect(),
            when_started: old_version.when_started,
            when_stopped: old_version.when_stopped,
            dedication: match old_version.dedication {
                SurrealDedicationOld::PrimaryTask => Some(SurrealDedication::PrimaryTask),
                SurrealDedicationOld::SecondaryTask => Some(SurrealDedication::BackgroundTask),
            },
            urgency: None,
        }
    }
}

impl SurrealTimeSpentOldVersion {
    pub(crate) const TABLE_NAME: &'static str = "time_spent_log";
}
