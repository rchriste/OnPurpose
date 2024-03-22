use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

use crate::base_data::covering::Covering;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "coverings")]
pub(crate) struct SurrealCovering {
    pub(crate) id: Option<Thing>,
    /// RecordId of the smaller item that is doing the covering
    pub(crate) smaller: RecordId, //covering
    /// RecordId of the larger item that is being covered
    pub(crate) parent: RecordId, //being_covered
}

impl<'a> From<Covering<'a>> for SurrealCovering {
    fn from(value: Covering<'a>) -> Self {
        SurrealCovering {
            id: None,
            smaller: value.smaller.get_id().clone(),
            parent: value.parent.get_id().clone(),
        }
    }
}
