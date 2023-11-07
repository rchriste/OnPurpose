use std::fmt::Display;

use crate::node::item_node::ItemNode;

use super::display_item::DisplayItem;

pub struct DisplayItemNode<'s> {
    item_node: &'s ItemNode<'s>,
}

impl Display for DisplayItemNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_mentally_resident() {
            write!(f, "🧠 ")?;
        }
        let display_item = DisplayItem::new(self.item_node.get_item());
        if self.item_node.is_person_or_group() {
            write!(f, "Is {} around?", display_item)?;
        } else if self.item_node.is_goal() && self.item_node.get_smaller().is_empty() {
            write!(f, "[NEEDS NEXT STEP] ⬅ {}", display_item)?;
        } else {
            write!(f, "{} ", display_item)?;
        }
        let parents = self.item_node.create_parent_chain();
        for item in parents.iter() {
            let display_item = DisplayItem::new(item);
            write!(f, " ⬅ {}", display_item)?;
        }
        Ok(())
    }
}

impl<'s> DisplayItemNode<'s> {
    pub(crate) fn new(item_node: &'s ItemNode<'s>) -> Self {
        DisplayItemNode { item_node }
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        self.item_node.is_mentally_resident()
    }

    pub(crate) fn get_item_node(&self) -> &'s ItemNode<'s> {
        self.item_node
    }
}
