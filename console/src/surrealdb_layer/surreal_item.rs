use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb_extra::table::Table;

use crate::base_data::{Item, ItemType};

use super::surreal_required_circumstance::SurrealRequiredCircumstance;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "item")] //TODO: This should be renamed items
pub struct SurrealItem {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
    pub item_type: ItemType,
}

impl SurrealItem {
    pub fn make_item<'a>(&'a self, requirements: &'a [SurrealRequiredCircumstance]) -> Item<'a> {
        let my_requirements = requirements
            .iter()
            .filter(|x| {
                &x.required_for
                    == self
                        .id
                        .as_ref()
                        .expect("Item should already be in the database and have an id")
            })
            .collect();

        Item {
            id: self
                .id
                .as_ref()
                .expect("Item should already be in the database and have an id"),
            summary: &self.summary,
            finished: &self.finished,
            item_type: &self.item_type,
            required_circumstances: my_requirements,
            surreal_item: self,
        }
    }
}
