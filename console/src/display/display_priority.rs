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
    Yes {
        highest_lap_count: DisplayItemStatus<'e>,
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
                    DisplayStaging::new(self.display_item_status.get_staging()),
                    self.display_item_status.get_lap_count(),
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
    pub(crate) fn new(
        item_status: &'s ItemStatus<'s>,
        all_item_status: &'s [ItemStatus<'s>],
    ) -> Self {
        let has_children = if item_status.has_children(Filter::Active) {
            let highest_lap_count = calculate_lap_count(item_status, all_item_status);
            HasChildren::Yes {
                highest_lap_count: DisplayItemStatus::new(highest_lap_count),
            }
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

    pub(crate) fn get_item_status(&'s self) -> &'s ItemStatus<'s> {
        self.display_item_status.get_item_status()
    }

    pub(crate) fn has_children(&self) -> bool {
        match &self.has_children {
            HasChildren::Yes { .. } => true,
            HasChildren::No => false,
        }
    }
}

fn calculate_lap_count<'a>(
    item_status: &'a ItemStatus,
    all_item_status: &'a [ItemStatus],
) -> &'a ItemStatus<'a> {
    if item_status.has_children(Filter::Active) {
        let highest_lap_count = item_status
            .get_smaller(Filter::Active)
            .map(|x| {
                all_item_status
                    .iter()
                    .find(|y| y.get_item() == x.get_item())
                    .expect("Comes from this list so it will always be there")
            })
            .reduce(|a, b| {
                let a_highest = calculate_lap_count(a, all_item_status);
                let b_highest = calculate_lap_count(b, all_item_status);
                if a_highest.get_lap_count() > b_highest.get_lap_count() {
                    a_highest
                } else {
                    b_highest
                }
            })
            .expect("has_children is true so there is at least one item");

        // Reduce is not called if there is only one child so in that scenario this is needed to ensure that we select the deepest child
        if highest_lap_count.has_children(Filter::Active) {
            let children = highest_lap_count
                .get_smaller(Filter::Active)
                .map(|x| {
                    all_item_status
                        .iter()
                        .find(|y| y.get_item() == x.get_item())
                        .expect("Comes from this list so it will always be there")
                })
                .collect::<Vec<_>>();
            assert!(children.len() == 1, "This should only happen if reduce is never called, meaning there is only one child");
            let child = children
                .first()
                .expect("Because of assert there is only one child");
            calculate_lap_count(child, all_item_status)
        } else {
            highest_lap_count
        }
    } else {
        item_status
    }
}
