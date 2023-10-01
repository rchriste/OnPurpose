use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb_extra::table::Table;

use crate::base_data::{Item, ItemType};

use super::surreal_requirement::SurrealRequirement;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "item")] //TODO: This should be renamed items
pub struct SurrealItem {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
    pub item_type: ItemType,
}

pub trait SurrealItemVecExtensions {
    fn make_items<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Vec<Item<'a>>;
}

impl SurrealItemVecExtensions for [SurrealItem] {
    fn make_items<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Vec<Item<'a>> {
        self.iter().map(|x| x.make_item(requirements)).collect()
    }
}

impl SurrealItem {
    pub fn make_item<'a>(&'a self, requirements: &'a [SurrealRequirement]) -> Item<'a> {
        let my_requirements = requirements
            .iter()
            .filter(|x| {
                &x.requirement_for
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
            requirements: my_requirements,
            surreal_item: self,
        }
    }
}
