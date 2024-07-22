use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

use super::SurrealTrigger;

//derive Builder is only for tests, I tried adding it just for cfg_attr(test... but that
//gave me false errors in the editor (rust-analyzer) so I am just going to try including
//it always to see if that addresses these phantom errors. Nov2023.
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug, Builder)]
#[table(name = "in_the_moment_priorities")]
pub(crate) struct SurrealInTheMomentPriority {
    pub(crate) id: Option<Thing>,
    pub(crate) choice: SurrealAction,
    pub(crate) kind: SurrealPriorityKind,
    pub(crate) not_chosen: Vec<SurrealAction>,
    pub(crate) in_effect_until: Vec<SurrealTrigger>,

    #[cfg_attr(test, builder(default))]
    pub(crate) created: Datetime,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealAction {
    SetReadyAndUrgency(RecordId),
    ParentBackToAMotivation(RecordId),
    ReviewItem(RecordId),
    PickItemReviewFrequency(RecordId),
    //PickWhatShouldBeDoneFirst is not on this list, because that would be recursive, and for logging Time Spent is probably not something that we want logged in case the user just selects one to do right now
    MakeProgress(RecordId),
}

impl SurrealAction {
    pub(crate) fn get_record_id(&self) -> &RecordId {
        match self {
            SurrealAction::SetReadyAndUrgency(record_id) => record_id,
            SurrealAction::ParentBackToAMotivation(record_id) => record_id,
            SurrealAction::ReviewItem(record_id) => record_id,
            SurrealAction::PickItemReviewFrequency(record_id) => record_id,
            SurrealAction::MakeProgress(record_id) => record_id,
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealPriorityKind {
    HighestPriority,
    LowestPriority,
}
