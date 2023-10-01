use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

use crate::base_data::Covering;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "coverings")]
pub struct SurrealCovering {
    pub id: Option<Thing>,
    pub smaller: RecordId,
    pub parent: RecordId,
}

impl<'a> From<Covering<'a>> for SurrealCovering {
    fn from(value: Covering<'a>) -> Self {
        SurrealCovering {
            id: None,
            smaller: value.smaller.id.clone(),
            parent: value.parent.id.clone(),
        }
    }
}