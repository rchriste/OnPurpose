use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::node::item_node::ItemNode;

use super::display_item::DisplayItem;

pub struct DisplayItemNode<'s> {
    item_node: &'s ItemNode<'s>,
    current_date_time: Option<&'s DateTime<Utc>>,
}

impl Display for DisplayItemNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_mentally_resident() {
            write!(f, "🧠 ")?;
        }
        let display_item = DisplayItem::new(self.item_node.get_item());
        if let Some(current_date_time) = self.current_date_time {
            if self.item_node.is_staging_on_deck_expired(current_date_time)
                || self
                    .item_node
                    .is_mentally_resident_expired(current_date_time)
            {
                write!(f, "❗ ")?;
            }
        }
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
    pub(crate) fn new(
        item_node: &'s ItemNode<'s>,
        current_date_time: Option<&'s DateTime<Utc>>,
    ) -> Self {
        DisplayItemNode {
            item_node,
            current_date_time,
        }
    }

    pub(crate) fn make_list(
        item_nodes: &'s [ItemNode<'s>],
        current_date_time: Option<&'s DateTime<Utc>>,
    ) -> Vec<DisplayItemNode<'s>> {
        item_nodes
            .iter()
            .map(|x| DisplayItemNode::new(x, current_date_time))
            .collect()
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        self.item_node.is_mentally_resident()
    }

    pub(crate) fn get_item_node(&self) -> &'s ItemNode<'s> {
        self.item_node
    }
}
