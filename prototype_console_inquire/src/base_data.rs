use serde::{Serialize, Deserialize};
use surrealdb::{sql::Thing, opt::RecordId};
use surrealdb_extra::table::Table;

use crate::test_data::TestData;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone)]
#[table(name = "next_step_item")]
pub struct NextStepItem {
    pub id: Option<Thing>,
    pub summary: String,
}

impl NextStepItem {
    pub fn is_covered(self: &NextStepItem, linkage: &Vec<LinkageWithReferences<'_>>) -> bool {
        let next_step_item = Item::NextStepItem(&self);
        linkage.iter().any(|x| x.parent == next_step_item)
    }
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone)]
#[table(name = "review_item")]
pub struct ReviewItem {
    pub id: Option<Thing>,
    pub summary: String,
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone)]
#[table(name = "reason_item")]
pub struct ReasonItem {
    pub id: Option<Thing>,
    pub summary: String,
}

pub struct LinkageWithReferences<'a> {
    pub smaller: Item<'a>,
    pub parent: Item<'a>,
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone)]
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
            smaller: value.smaller.get_id().as_ref().expect("Should already be in the DB").clone(), 
            parent: value.parent.get_id().as_ref().expect("Should already be in the DB").clone()
        }
    }
}

pub fn convert_linkage_with_record_ids_to_references<'a>(linkage_with_record_ids: &Vec<LinkageWithRecordIds>, test_data: &'a TestData) -> Vec<LinkageWithReferences<'a>>
{
    linkage_with_record_ids.iter().map(|x|
        LinkageWithReferences 
        { 
            smaller: test_data.lookup_from_record_id(&x.smaller).unwrap(), 
            parent: test_data.lookup_from_record_id(&x.parent).unwrap()
        }
    ).collect()
}

#[derive(PartialEq, Eq)]
pub enum Item<'a> {
    NextStepItem(&'a NextStepItem),
    ReviewItem(&'a ReviewItem),
    ReasonItem(&'a ReasonItem)
}

impl<'a> Item<'a> {
    pub fn from_next_step(next_step_item: &'a NextStepItem) -> Item<'a>
    {
        Item::NextStepItem(next_step_item)
    }

    pub fn from_review_item(review_item: &'a ReviewItem) -> Item<'a>
    {
        Item::ReviewItem(review_item)
    }

    pub fn from_reason_item(reason_item: &'a ReasonItem) -> Item<'a>
    {
        Item::ReasonItem(reason_item)
    }

    pub fn find_parents(&self, linkage: &'a Vec<LinkageWithReferences<'a>>) -> Vec<&'a Item<'a>>
    {
        linkage.iter().filter_map(|x| {
            if &x.smaller == self { Some(&x.parent) }
            else { None }
        }).collect()
    }

    pub fn get_id(&'a self) -> &'a Option<Thing>
    {
        match self {
            Item::NextStepItem(next_step) => &next_step.id,
            Item::ReviewItem(review_item) => &review_item.id,
            Item::ReasonItem(reason_item) => &reason_item.id,
        }
    }
}