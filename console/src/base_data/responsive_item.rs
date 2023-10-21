use crate::surrealdb_layer::surreal_item::{Responsibility, SurrealItem};

use super::item::Item;

/// Could have a motivation_type with options for Commitment (do it because the outcome of doing it is wanted), Obligation (do it because the consequence of not doing it is bad), or Value
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ResponsiveItem<'s> {
    item: &'s Item<'s>,
}

impl<'s> ResponsiveItem<'s> {
    pub fn new(item: &'s Item<'s>) -> Self {
        assert_eq!(
            *item.responsibility,
            Responsibility::ReactiveBeAvailableToAct
        );
        Self { item }
    }

    pub fn get_item(&self) -> &'s Item {
        self.item //TODO: Switch to using a crate that does this automatically making this getter
    }

    pub fn get_surreal_item(&self) -> &'s SurrealItem {
        self.item.get_surreal_item()
    }
}
