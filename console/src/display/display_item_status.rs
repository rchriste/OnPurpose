use std::fmt::Display;

use crate::node::item_status::ItemStatus;

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
}
