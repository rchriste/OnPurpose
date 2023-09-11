#[derive(PartialEq, Eq)]
pub struct NextStepItem {
    pub summary: String,
}

/// Could have a review_type with options for Milestone, StoppingPoint, and ReviewPoint
#[derive(PartialEq, Eq)]
pub struct ReviewItem {
    pub summary: String,
}

/// Could have a reason_type with options for Commitment, Maintenance, or Value
#[derive(PartialEq, Eq)]
pub struct ReasonItem {
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