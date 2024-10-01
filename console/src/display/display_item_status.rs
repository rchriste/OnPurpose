use std::fmt::Display;

use crate::node::{item_status::ItemStatus, Filter};

use super::display_item_node::DisplayItemNode;

pub struct DisplayItemStatus<'s> {
    item_status: &'s ItemStatus<'s>,
}

impl Display for DisplayItemStatus<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "|")?;
        if self.has_dependencies(Filter::Active) {
            //write a red circle emoji
            write!(f, "⌛ ")?;
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

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_status.has_children(filter)
    }

    pub(crate) fn has_dependencies(&self, filter: Filter) -> bool {
        self.item_status.has_dependencies(filter)
    }
}
