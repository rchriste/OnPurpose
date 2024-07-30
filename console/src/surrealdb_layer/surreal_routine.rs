use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealRoutine {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,
    pub(crate) parent: Thing,
}

impl SurrealRoutine {
    pub(crate) const TABLE_NAME: &'static str = "routines";
}
