use std::fmt::{Display, Formatter};

use crate::{
    base_data::item::Item,
    node::{item_status::ItemStatus, Filter},
    surrealdb_layer::surreal_item::Staging,
};

use super::{
    display_item::DisplayItem, display_item_status::DisplayItemStatus,
    display_staging::DisplayStaging,
};

pub(crate) struct DisplayPriority<'s> {
    display_item_status: DisplayItemStatus<'s>,
}

impl<'s> Display for DisplayPriority<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_item = DisplayItem::new(self.get_item());
        write!(f, "{} \n\t", display_item)?;
        if self.has_children(Filter::Active) {
            write!(f, "")
        } else {
            let display_staging = DisplayStaging::new(self.get_staging());
            write!(f, "|{:.1}| {}", self.get_lap_count(), display_staging)
        }
    }
}

impl<'s> DisplayPriority<'s> {
    pub(crate) fn new(item_status: &'s ItemStatus<'s>) -> Self {
        DisplayPriority {
            display_item_status: DisplayItemStatus::new(item_status),
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item {
        self.display_item_status.get_item()
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.display_item_status.has_children(filter)
    }

    pub(crate) fn get_staging(&self) -> &'s Staging {
        self.display_item_status.get_staging()
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.display_item_status.get_item_status().get_lap_count()
    }
}
