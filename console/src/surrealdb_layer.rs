use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Duration},
};

pub(crate) mod data_layer_commands;
pub(crate) mod surreal_in_the_moment_priority;
pub(crate) mod surreal_item;
pub(crate) mod surreal_life_area;
pub(crate) mod surreal_processed_text;
pub(crate) mod surreal_required_circumstance;
pub(crate) mod surreal_routine;
pub(crate) mod surreal_tables;
pub(crate) mod surreal_time_spent;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealTrigger {
    WallClockDateTime(Datetime),
    LoggedInvocationCount {
        starting: Datetime,
        count: u32,
        items_in_scope: SurrealItemsInScope,
    },
    LoggedAmountOfTime {
        starting: Datetime,
        duration: Duration,
        items_in_scope: SurrealItemsInScope,
    },
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealItemsInScope {
    All,
    Include(Vec<RecordId>),
    Exclude(Vec<RecordId>),
}
