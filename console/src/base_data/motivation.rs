use surrealdb::sql::{Datetime, Thing};

use crate::surrealdb_layer::surreal_item::SurrealItem;

use super::item::Item;

/// Could have a motivation_type with options for Commitment (do it because the outcome of doing it is wanted), Obligation (do it because the consequence of not doing it is bad), or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Motivation<'s> {
    pub id: &'s Thing,
    pub summary: &'s String,
    pub finished: &'s Option<Datetime>,
    item: &'s Item<'s>,
}

impl<'s> From<Motivation<'s>> for Thing {
    fn from(value: Motivation<'s>) -> Self {
        value.id.clone()
    }
}

impl<'s> Motivation<'s> {
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

impl<'s> Motivation<'s> {
    pub(crate) fn new(item: &'s Item<'s>) -> Self {
        //TODO: Assert to ensure that it is a motivation
        Self {
            id: item.id,
            summary: item.summary,
            finished: item.finished,
            item,
        }
    }

    pub fn get_item(&self) -> &'s Item {
        self.item //TODO: Change to using a derive crate that can generate these automatically
    }

    pub fn get_surreal_item(&self) -> &'s SurrealItem {
        self.item.get_surreal_item()
    }
}
