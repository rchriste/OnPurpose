pub mod hope;
pub mod item;
pub mod motivation;
pub mod to_do;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

use crate::surrealdb_layer::{
    surreal_item::SurrealItem,
    surreal_required_circumstance::{CircumstanceType, SurrealRequiredCircumstance},
};

use self::item::Item;

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum ItemType {
    ToDo,
    Hope,
    Motivation,
}

pub struct Covering<'a> {
    pub smaller: &'a Item<'a>,
    pub parent: &'a Item<'a>,
}

pub struct CoveringUntilDateTime<'a> {
    pub cover_this: &'a Item<'a>,
    pub until: DateTime<Local>,
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "processed_text")]
pub struct ProcessedText {
    pub id: Option<Thing>,
    pub text: String,
    pub when_written: Datetime,
    pub for_item: RecordId,
}

impl Item<'_> {
    pub fn find_parents<'a>(&self, linkage: &'a [Covering<'a>]) -> Vec<&'a Item<'a>> {
        linkage
            .iter()
            .filter_map(|x| {
                if x.smaller == self {
                    Some(x.parent)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Circumstance<'a> {
    pub circumstance_for: &'a SurrealItem,
    pub circumstance_type: &'a CircumstanceType,
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
