use std::fmt::{Display, Formatter};

use crate::{
    base_data::item::Item,
    node::{item_highest_lap_count::ItemHighestLapCount, item_status::ItemStatus, Filter},
    surrealdb_layer::surreal_item::SurrealStaging,
};

use super::{
    display_item::DisplayItem, display_item_lap_count::DisplayItemLapCount,
    display_staging::DisplayStaging,
};

pub(crate) struct DisplayPriority<'s> {
    display_item_lap_count: DisplayItemLapCount<'s>,
    has_children: HasChildren<'s>,
}

enum HasChildren<'e> {
    Yes {
        highest_lap_count: DisplayItemLapCount<'e>,
    },
    No,
}

impl<'s> Display for DisplayPriority<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_item = DisplayItem::new(self.get_item());
        write!(f, "{} \n   ", display_item)?;
        let (display_staging, lap_count) = match &self.has_children {
            HasChildren::Yes { highest_lap_count } => {
                write!(f, "↥")?;
                (
                    DisplayStaging::new(highest_lap_count.get_staging()),
                    highest_lap_count.get_lap_count(),
                )
            }
            HasChildren::No => {
                write!(f, "•")?;
                (
                    DisplayStaging::new(self.get_staging()),
                    self.get_lap_count(),
                )
            }
        };
        write!(f, " |")?;
        write!(f, "{:.1}", lap_count)?;
        write!(f, "| {}", display_staging)
    }
}

impl<'s> DisplayPriority<'s> {
    pub(crate) fn new(item_highest_lap_count: &'s ItemHighestLapCount<'s>) -> Self {
        let has_children = if item_highest_lap_count.has_children(Filter::Active) {
            let highest_lap_count = item_highest_lap_count.get_highest_lap_count_item();
            HasChildren::Yes {
                highest_lap_count: DisplayItemLapCount::new(highest_lap_count),
            }
        } else {
            HasChildren::No
        };
        DisplayPriority {
            display_item_lap_count: DisplayItemLapCount::new(
                item_highest_lap_count.get_item_lap_count(),
            ),
            has_children,
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item {
        self.display_item_lap_count.get_item()
    }

    pub(crate) fn get_item_status(&'s self) -> &'s ItemStatus<'s> {
        self.display_item_lap_count.get_item_status()
    }

    pub(crate) fn has_children(&self) -> bool {
        match &self.has_children {
            HasChildren::Yes { .. } => true,
            HasChildren::No => false,
        }
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.display_item_lap_count.get_lap_count()
    }

    pub(crate) fn get_staging(&self) -> &'s SurrealStaging {
        self.display_item_lap_count.get_staging()
    }
}
