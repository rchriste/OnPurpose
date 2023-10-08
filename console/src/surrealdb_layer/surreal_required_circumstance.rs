use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "required_circumstances")]
pub struct SurrealRequiredCircumstance {
    pub id: Option<Thing>,
    pub required_for: RecordId,
    pub circumstance_type: CircumstanceType,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum CircumstanceType {
    NotSunday,
    DuringFocusTime,
}
