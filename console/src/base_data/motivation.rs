use surrealdb::sql::{Datetime, Thing};

use super::item::Item;

/// Could have a motivation_type with options for Commitment (do it because the outcome of doing it is wanted), Obligation (do it because the consequence of not doing it is bad), or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Motivation<'s> {
    pub(crate) id: &'s Thing,
    pub(crate) summary: &'s str,
    pub(crate) finished: &'s Option<Datetime>,
    item: &'s Item<'s>,
}

impl<'s> From<Motivation<'s>> for Thing {
    fn from(value: Motivation<'s>) -> Self {
        value.id.clone()
    }
}

impl<'s> Motivation<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        //TODO: Assert to ensure that it is a motivation
        Self {
            id: item.get_id(),
            summary: item.get_summary(),
            finished: item.get_finished(),
            item,
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item {
        self.item //TODO: Change to using a derive crate that can generate these automatically
    }
}
