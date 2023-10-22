pub(crate) mod hope;
pub(crate) mod item;
pub(crate) mod life_area;
pub(crate) mod motivation;
pub(crate) mod motivation_or_responsive_item;
pub(crate) mod responsive_item;
pub(crate) mod routine;
pub(crate) mod to_do;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

use crate::surrealdb_layer::{
    surreal_item::{SurrealItem, SurrealOrderedSubItem},
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
};

use self::item::Item;

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum ItemType {
    ToDo,
    Hope,
    Motivation,
}

pub(crate) struct Covering<'a> {
    pub(crate) smaller: &'a Item<'a>,
    pub(crate) parent: &'a Item<'a>,
}

pub(crate) struct CoveringUntilDateTime<'a> {
    pub(crate) cover_this: &'a Item<'a>,
    pub(crate) until: DateTime<Local>,
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "processed_text")]
pub(crate) struct ProcessedText {
    pub(crate) id: Option<Thing>,
    pub(crate) text: String,
    pub(crate) when_written: Datetime,
    pub(crate) for_item: RecordId,
}

impl Item<'_> {
    pub(crate) fn find_parents<'a>(
        &self,
        linkage: &'a [Covering<'a>],
        other_items: &'a [&'a Item<'a>],
    ) -> Vec<&'a Item<'a>> {
        let mut result: Vec<&'a Item<'a>> = linkage
            .iter()
            .filter_map(|x| {
                if x.smaller == self {
                    Some(x.parent)
                } else {
                    None
                }
            })
            .collect();

        result.extend(other_items.iter().filter_map(|other_item| {
            if other_item.is_this_a_smaller_item(self) {
                Some(*other_item)
            } else {
                None
            }
        }));
        result
    }

    pub(crate) fn is_this_a_smaller_item(&self, other_item: &Item) -> bool {
        self.surreal_item
            .smaller_items_in_priority_order
            .iter()
            .any(|x| match x {
                SurrealOrderedSubItem::SubItem { surreal_item_id } => {
                    other_item
                        .surreal_item
                        .id
                        .as_ref()
                        .expect("Should always be in DB")
                        == surreal_item_id
                }
                SurrealOrderedSubItem::Split { shared_priority: _ } => {
                    todo!("Implement this now that this variant is more than a placeholder")
                }
            })
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Circumstance<'a> {
    pub(crate) circumstance_for: &'a SurrealItem,
    pub(crate) circumstance_type: &'a CircumstanceType,
    surreal_required_circumstance: &'a SurrealRequiredCircumstance,
}

impl<'a> From<&Circumstance<'a>> for &'a SurrealRequiredCircumstance {
    fn from(value: &Circumstance<'a>) -> Self {
        value.surreal_required_circumstance
    }
}

impl<'a> From<Circumstance<'a>> for &'a SurrealRequiredCircumstance {
    fn from(value: Circumstance<'a>) -> Self {
        value.surreal_required_circumstance
    }
}
