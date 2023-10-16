use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "routines")]
pub struct SurrealRoutine {
    pub id: Option<Thing>,
    pub summary: String,
    pub parent: Thing,
}
