use std::fmt::{Display, Formatter};

use crate::{
    base_data::item::Item,
    node::{item_status::ItemStatus, Filter},
};

use super::{
    display_item::DisplayItem, display_item_status::DisplayItemStatus,
    display_staging::DisplayStaging,
};

pub(crate) struct DisplayPriority<'s> {
    display_item_status: DisplayItemStatus<'s>,
    has_children: HasChildren<'s>,
}

enum HasChildren<'e> {
    Yes{highest_lap_count: DisplayItemStatus<'e>},
    No,
}

impl<'s> Display for DisplayPriority<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_item = DisplayItem::new(self.get_item());
        write!(f, "{} \n   ", display_item)?;
        let (display_staging, lap_count) = match &self.has_children {
            HasChildren::Yes{highest_lap_count} => {
                write!(f, "↥")?;
                (
                    DisplayStaging::new(highest_lap_count.get_staging()),
                    highest_lap_count.get_lap_count()
                )
            }
            HasChildren::No => {
                write!(f, "•")?;
                (
                    DisplayStaging::new(self.display_item_status.get_staging()),
                    self.display_item_status.get_lap_count()
                )
            }
        };
        write!(f, " |")?;
        if lap_count >= 0.0 {
            write!(f, "{:.1}", lap_count)?;
        }
        write!(f, "| {}", display_staging)
    }
}

impl<'s> DisplayPriority<'s> {
    pub(crate) fn new(item_status: &'s ItemStatus<'s>, all_item_status: &'s[ItemStatus<'s>]) -> Self {
        let has_children = if item_status.has_children(Filter::Active) {
            let highest_lap_count = all_item_status.iter().filter(|x| item_status.get_smaller(Filter::Active).find(|y| y.get_item() == x.get_item()).is_some()).max_by(|a, b| {
                a.get_lap_count().partial_cmp(&b.get_lap_count()).unwrap()
            }).expect("Has children so there is a max");
            HasChildren::Yes{highest_lap_count: DisplayItemStatus::new(highest_lap_count)}
        } else {
            HasChildren::No
        };
        DisplayPriority {
            display_item_status: DisplayItemStatus::new(item_status),
            has_children,
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item {
        self.display_item_status.get_item()
    }
}
