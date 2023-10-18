use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb_extra::table::Table;

use crate::base_data::{item::Item, ItemType};

use super::surreal_required_circumstance::SurrealRequiredCircumstance;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "item")] //TODO: This should be renamed items
pub struct SurrealItem {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
    pub item_type: ItemType,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    pub smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,
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

        Item::new(self, my_requirements)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum SurrealOrderedSubItem {
    SubItem {
        surreal_item_id: Thing,
    },
    Split {
        shared_priority: Vec<SurrealPriorityGoal>,
    },
}

//Each of these variants should be containing data but I don't want the data layer to get too far ahead of the prototype UI
//so I want to wait until I can try it out before working out these details so just this for now.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum SurrealPriorityGoal {
    AbsoluteInvocationCount,
    AbsoluteAmountOfTime,
    RelativePercentageOfTime,
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "item")] //TODO: Remove this after the upgrade is complete
pub struct SurrealItemOldVersion {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
    pub item_type: ItemType,
}

impl From<SurrealItemOldVersion> for SurrealItem {
    fn from(old: SurrealItemOldVersion) -> Self {
        SurrealItem {
            id: old.id,
            summary: old.summary,
            finished: old.finished,
            item_type: old.item_type,
            smaller_items_in_priority_order: Vec::default(),
        }
    }
}
