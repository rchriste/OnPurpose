use std::fmt::Display;

use crate::node::{
    item_status::{ItemStatus, LapCount, LapCountGreaterOrLess},
    Filter,
};

use super::display_item_node::DisplayItemNode;

pub struct DisplayItemStatus<'s> {
    item_status: &'s ItemStatus<'s>,
}

impl Display for DisplayItemStatus<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.has_children(Filter::Active) {
            let lap_count = self.get_lap_count();
            write!(f, "|")?;
            match lap_count {
                LapCount::F32(float) => {
                    if *float >= 0.0 {
                        write!(f, "{:.1}", float)?;
                    }
                }
                LapCount::Ratio {
                    other_item,
                    greater_or_less,
                } => {
                    write!(f, "{}", other_item)?;
                    match greater_or_less {
                        LapCountGreaterOrLess::GreaterThan => write!(f, "⭱ ")?,
                        LapCountGreaterOrLess::LessThan => write!(f, "⭳ ")?,
                    }
                }
            }
            write!(f, "| ")?;
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

    pub(crate) fn get_lap_count(&self) -> &LapCount {
        self.item_status.get_lap_count()
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_status.has_children(filter)
    }
}
