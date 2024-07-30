use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealLifeArea {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,
}

impl SurrealLifeArea {
    pub(crate) const TABLE_NAME: &'static str = "life_areas";
}