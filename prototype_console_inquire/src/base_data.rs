use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

use crate::test_data::TestData;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "next_step_item")]
pub struct ToDo {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
}

impl From<ToDo> for Option<Thing> {
    fn from(value: ToDo) -> Self {
        value.id
    }
}

impl ToDo {
    pub fn is_covered(&self, linkage: &[LinkageWithReferences<'_>]) -> bool {
        let next_step_item = Item::ToDo(self);
        let mut covered_by = linkage.iter().filter(|x| x.parent == next_step_item);
        //Now see if the items that are covering are finished or active
        covered_by.any(|x| !x.smaller.is_finished())
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "review_item")]
pub struct Hope {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
}

impl From<Hope> for Option<Thing> {
    fn from(value: Hope) -> Self {
        value.id
    }
}

impl Hope {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "reason_item")]
pub struct Reason {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
}

impl From<Reason> for Option<Thing> {
    fn from(value: Reason) -> Self {
        value.id
    }
}

impl Reason {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

pub struct LinkageWithReferences<'a> {
    pub smaller: Item<'a>,
    pub parent: Item<'a>,
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "linkage")]
pub struct LinkageWithRecordIds {
    pub id: Option<Thing>,
    pub smaller: RecordId,
    pub parent: RecordId,
}

impl<'a> From<LinkageWithReferences<'a>> for LinkageWithRecordIds {
    fn from(value: LinkageWithReferences<'a>) -> Self {
        LinkageWithRecordIds {
            id: None,
            smaller: value
                .smaller
                .get_id()
                .as_ref()
                .expect("Should already be in the DB")
                .clone(),
            parent: value
                .parent
                .get_id()
                .as_ref()
                .expect("Should already be in the DB")
                .clone(),
        }
    }
}

pub fn convert_linkage_with_record_ids_to_references<'a>(
    linkage_with_record_ids: &[LinkageWithRecordIds],
    test_data: &'a TestData,
) -> Vec<LinkageWithReferences<'a>> {
    linkage_with_record_ids
        .iter()
        .map(|x| LinkageWithReferences {
            smaller: test_data.lookup_from_record_id(&x.smaller).unwrap(),
            parent: test_data.lookup_from_record_id(&x.parent).unwrap(),
        })
        .collect()
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "processed_text")]
pub struct ProcessedText {
    pub id: Option<Thing>,
    pub text: String,
    pub when_written: Datetime,
    pub for_item: RecordId,
}

#[derive(PartialEq, Eq)]
pub enum Item<'a> {
    ToDo(&'a ToDo),
    Hope(&'a Hope),
    Reason(&'a Reason),
}

impl<'a> From<&'a ToDo> for Item<'a> {
    fn from(value: &'a ToDo) -> Self {
        Item::ToDo(value)
    }
}

impl<'a> From<&'a Hope> for Item<'a> {
    fn from(value: &'a Hope) -> Self {
        Item::Hope(value)
    }
}

impl<'a> From<&'a Reason> for Item<'a> {
    fn from(value: &'a Reason) -> Self {
        Item::Reason(value)
    }
}

impl<'a> Item<'a> {
    pub fn from_to_do(to_do: &'a ToDo) -> Item<'a> {
        Item::ToDo(to_do)
    }

    pub fn from_hope(hope: &'a Hope) -> Item<'a> {
        Item::Hope(hope)
    }

    pub fn from_reason_item(reason: &'a Reason) -> Item<'a> {
        Item::Reason(reason)
    }

    pub fn find_parents(&self, linkage: &'a [LinkageWithReferences<'a>]) -> Vec<&'a Item<'a>> {
        linkage
            .iter()
            .filter_map(|x| {
                if &x.smaller == self {
                    Some(&x.parent)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_id(&'a self) -> &'a Option<Thing> {
        match self {
            Item::ToDo(to_do) => &to_do.id,
            Item::Hope(hope) => &hope.id,
            Item::Reason(reason_item) => &reason_item.id,
        }
    }

    pub fn is_finished(&'a self) -> bool {
        match self {
            Item::ToDo(i) => i.is_finished(),
            Item::Hope(i) => i.is_finished(),
            Item::Reason(i) => i.is_finished(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ItemOwning {
    ToDo(ToDo),
    Hope(Hope),
    Reason(Reason),
}

impl From<ToDo> for ItemOwning {
    fn from(value: ToDo) -> Self {
        Self::ToDo(value)
    }
}

impl From<Hope> for ItemOwning {
    fn from(value: Hope) -> Self {
        Self::Hope(value)
    }
}

impl From<Reason> for ItemOwning {
    fn from(value: Reason) -> Self {
        Self::Reason(value)
    }
}

impl From<ItemOwning> for Option<Thing> {
    fn from(value: ItemOwning) -> Self {
        match value {
            ItemOwning::ToDo(i) => i.into(),
            ItemOwning::Hope(i) => i.into(),
            ItemOwning::Reason(i) => i.into(),
        }
    }
}

impl<'a> From<Item<'a>> for ItemOwning {
    fn from(value: Item<'a>) -> Self {
        match value {
            Item::ToDo(i) => i.into(),
            Item::Hope(i) => i.into(),
            Item::Reason(i) => i.into(),
        }
    }
}

impl From<&ToDo> for ItemOwning {
    fn from(value: &ToDo) -> Self {
        ItemOwning::ToDo(value.clone())
    }
}

impl From<&Hope> for ItemOwning {
    fn from(value: &Hope) -> Self {
        ItemOwning::Hope(value.clone())
    }
}

impl From<&Reason> for ItemOwning {
    fn from(value: &Reason) -> Self {
        ItemOwning::Reason(value.clone())
    }
}
