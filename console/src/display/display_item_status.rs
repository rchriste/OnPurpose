use std::fmt::Display;

use crate::{
    base_data::item::Item,
    node::item_status::ItemStatus,
    surrealdb_layer::surreal_item::Staging,
};

use super::display_item_node::DisplayItemNode;

pub struct DisplayItemStatus<'s> {
    item_status: &'s ItemStatus<'s>,
}

impl Display for DisplayItemStatus<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.item_status.is_first_lap_finished() {
            write!(f, "⏰ ")?;
        }

        let display_node = DisplayItemNode::new(self.item_status.get_item_node());
        write!(f, "{}", display_node)?;
        Ok(())
    }
}

impl<'s> DisplayItemStatus<'s> {
    pub(crate) fn new(item_status: &'s ItemStatus) -> Self {
        Self { item_status }
    }

    pub(crate) fn get_item_status(&self) -> &'s ItemStatus {
        self.item_status
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item_status.get_item()
    }

    pub(crate) fn get_staging(&self) -> &'s Staging {
        self.item_status.get_staging()
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.item_status.get_lap_count()
    }
}
