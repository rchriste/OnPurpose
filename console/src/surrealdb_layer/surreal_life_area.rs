use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "life_areas")] //TODO: This should be adjusted to support change history tracking
pub struct SurrealLifeArea {
    pub id: Option<Thing>,
    pub summary: String,
}
