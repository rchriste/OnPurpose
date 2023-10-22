use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "routines")]
pub(crate) struct SurrealRoutine {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,
    pub(crate) parent: Thing,
}
