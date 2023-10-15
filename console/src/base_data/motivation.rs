use surrealdb::sql::{Datetime, Thing};

use super::item::Item;

/// Could have a motivation_type with options for Commitment (do it because the outcome of doing it is wanted), Obligation (do it because the consequence of not doing it is bad), or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Motivation<'a> {
    pub id: &'a Thing,
    pub summary: &'a String,
    pub finished: &'a Option<Datetime>,
    item: &'a Item<'a>,
}

impl<'a> From<Motivation<'a>> for Thing {
    fn from(value: Motivation<'a>) -> Self {
        value.id.clone()
    }
}

impl<'a> Motivation<'a> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

impl<'a> Motivation<'a> {
    pub(crate) fn new(item: &'a Item<'a>) -> Self {
        //TODO: Assert to ensure that it is a motivation
        Self {
            id: item.id,
            summary: item.summary,
            finished: item.finished,
            item,
        }
    }
}
