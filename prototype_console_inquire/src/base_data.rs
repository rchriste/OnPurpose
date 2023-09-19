use serde::{Serialize, Deserialize};
use surrealdb::{sql::{Thing, Datetime}, opt::RecordId};
use surrealdb_extra::table::Table;

use crate::test_data::TestData;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "next_step_item")]
pub struct NextStepItem {
    pub id: Option<Thing>,
    pub summary: String,
    pub finished: Option<Datetime>,
}

impl From<NextStepItem> for Option<Thing> {
    fn from(value: NextStepItem) -> Self {
        value.id
    }
}

impl NextStepItem {
    pub fn is_covered(&self, linkage: &Vec<LinkageWithReferences<'_>>) -> bool {
        let next_step_item = Item::NextStepItem(&self);
        linkage.iter().any(|x| x.parent == next_step_item)
    }

    pub fn is_finished(&self) -> bool {
        match self.finished {
            Some(_) => true,
            None => false,
        }
    }
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "review_item")]
pub struct ReviewItem {
    pub id: Option<Thing>,
    pub summary: String,
}

impl From<ReviewItem> for Option<Thing> {
    fn from(value: ReviewItem) -> Self {
        value.id
    }
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "reason_item")]
pub struct ReasonItem {
    pub id: Option<Thing>,
    pub summary: String,
}

impl From<ReasonItem> for Option<Thing> {
    fn from(value: ReasonItem) -> Self {
        value.id
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

#[derive(PartialEq, Eq)]
pub enum ItemOwning {
    NextStepItem(NextStepItem),
    ReviewItem(ReviewItem),
    ReasonItem(ReasonItem)
}

impl From<NextStepItem> for ItemOwning {
    fn from(value: NextStepItem) -> Self {
        Self::NextStepItem(value)
    }
}

impl From<ReviewItem> for ItemOwning {
    fn from(value: ReviewItem) -> Self {
        Self::ReviewItem(value)
    }
}

impl From<ReasonItem> for ItemOwning {
    fn from(value: ReasonItem) -> Self {
        Self::ReasonItem(value)
    }
}

impl From<ItemOwning> for Option<Thing> {
    fn from(value: ItemOwning) -> Self {
        match value {
            ItemOwning::NextStepItem(i) => i.into(),
            ItemOwning::ReviewItem(i) => i.into(),
            ItemOwning::ReasonItem(i) => i.into(),
        }
    }
}

impl<'a> From<Item<'a>> for ItemOwning {
    fn from(value: Item<'a>) -> Self {
        match value {
            Item::NextStepItem(i) => i.into(),
            Item::ReviewItem(i) => i.into(),
            Item::ReasonItem(i) => i.into(),
        }
    }
}

impl From<&NextStepItem> for ItemOwning {
    fn from(value: &NextStepItem) -> Self {
        ItemOwning::NextStepItem(value.clone())
    }
}

impl From<&ReviewItem> for ItemOwning {
    fn from(value: &ReviewItem) -> Self {
        ItemOwning::ReviewItem(value.clone())
    }
}

impl From<&ReasonItem> for ItemOwning {
    fn from(value: &ReasonItem) -> Self {
        ItemOwning::ReasonItem(value.clone())
    }
}