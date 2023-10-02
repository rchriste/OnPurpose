use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "requirements")]
pub struct SurrealRequirement {
    //TODO: Rename to SurrealCircumstance or SurrealRequiredCircumstance
    pub id: Option<Thing>,
    pub requirement_for: RecordId,
    pub requirement_type: RequirementType,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum RequirementType {
    NotSunday,
}
