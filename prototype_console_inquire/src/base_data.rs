use serde::{Serialize, Deserialize};
use surrealdb::sql::Thing;
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone)]
#[table(name = "next_step_item")]
pub struct NextStepItem {
    pub id: Option<Thing>,
    pub summary: String,
}

impl NextStepItem {
    pub fn is_covered(self: &NextStepItem, linkage: &Vec<Linkage<'_>>) -> bool {
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

pub struct Linkage<'a> {
    pub smaller: Item<'a>,
    pub parent: Item<'a>,
}

#[derive(PartialEq, Eq)]
pub enum Item<'a> {
    NextStepItem(&'a NextStepItem),
    ReviewItem(&'a ReviewItem),
    ReasonItem(&'a ReasonItem)
}

impl<'a> Item<'a> {
    pub fn find_parents(&self, linkage: &'a Vec<Linkage<'a>>) -> Vec<&'a Item<'a>>
    {
        linkage.iter().filter_map(|x| {
            if &x.smaller == self {Some(&x.parent)}
            else {None}
        }).collect()
    }
}