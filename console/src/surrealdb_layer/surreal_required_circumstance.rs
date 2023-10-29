use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "required_circumstances")]
pub(crate) struct SurrealRequiredCircumstance {
    //TODO: This should be renamed to SurrealRequirement
    pub(crate) id: Option<Thing>,
    pub(crate) required_for: RecordId,
    pub(crate) circumstance_type: CircumstanceType,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum CircumstanceType {
    NotSunday,
    DuringFocusTime, //TODO: I should add a new type for SurrealRequiredMentalState and this should be part of the PreferredOrRequiredMood
}
