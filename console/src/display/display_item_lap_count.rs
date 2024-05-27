use std::fmt::Display;

use crate::{
    base_data::item::Item,
    node::{item_lap_count::ItemLapCount, item_node::ItemNode, item_status::ItemStatus, Filter},
    surrealdb_layer::surreal_item::SurrealStaging,
};

use super::display_item_node::DisplayItemNode;

pub struct DisplayItemLapCount<'s> {
    item_lap_count: &'s ItemLapCount<'s>,
}

impl Display for DisplayItemLapCount<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.has_children(Filter::Active) {
            let lap_count = self.get_lap_count();
            write!(f, "|")?;
            if lap_count >= 0.0 {
                write!(f, "{:.1}", lap_count)?;
            }
            write!(f, "| ")?;
        }

        let display_node = DisplayItemNode::new(self.get_item_node());
        write!(f, "{}", display_node)?;
        Ok(())
    }
}

impl<'s> DisplayItemLapCount<'s> {
    pub(crate) fn new(item_lap_count: &'s ItemLapCount) -> Self {
        Self { item_lap_count }
    }

    pub(crate) fn get_item_node(&self) -> &'s ItemNode {
        self.item_lap_count.get_item_node()
    }

    pub(crate) fn get_item_status(&self) -> &'s ItemStatus {
        self.item_lap_count.get_item_status()
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item_lap_count.get_item()
    }

    pub(crate) fn get_staging(&self) -> &'s SurrealStaging {
        self.item_lap_count.get_staging()
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.item_lap_count.get_lap_count()
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_lap_count.has_children(filter)
    }
}
