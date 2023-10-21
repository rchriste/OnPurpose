use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Thing};
use surrealdb_extra::table::Table;

use super::surreal_item::Responsibility;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "specific_to_to_dos")]
pub struct SurrealSpecificToToDo {
    pub id: Option<Thing>,
    pub for_item: RecordId,
    pub order: Order,
    pub responsibility: Responsibility, //TODO: This needs to be completely migrated to just a SurrealItem
}

impl SurrealSpecificToToDo {
    pub fn new_defaults(for_item: RecordId) -> Self {
        Self {
            id: Option::<Thing>::default(),
            for_item,
            order: Order::default(),
            responsibility: Responsibility::default(),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum Order {
    //This is maybe something that should be tagged on the actual covering action rather than the to do itself
    #[default]
    NextStep,
    DoNotForget,
}
