use std::fmt::Display;

use crate::{
    base_data::item::Item,
    data_storage::surrealdb_layer::surreal_item::SurrealItemType,
    node::{item_node::ItemNode, Filter},
};

use super::display_item::DisplayItem;

pub struct DisplayItemNode<'s> {
    item_node: &'s ItemNode<'s>,
}

impl Display for DisplayItemNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_item = DisplayItem::new(self.item_node.get_item());

        if self.item_node.is_person_or_group() {
            write!(f, "Is {} around?", display_item)?;
        } else if self.item_node.is_goal() && !self.item_node.has_children(Filter::Active) {
            write!(f, "[NEEDS NEXT STEP] ⬅ {}", display_item)?;
        } else {
            write!(f, "{} ", display_item)?;
        }
        let parents = self.item_node.create_parent_chain(Filter::Active);
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

    pub(crate) fn make_list(item_nodes: &'s [ItemNode<'s>]) -> Vec<DisplayItemNode<'s>> {
        item_nodes.iter().map(DisplayItemNode::new).collect()
    }

    pub(crate) fn get_item_node(&self) -> &'s ItemNode<'s> {
        self.item_node
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item_node.is_type_motivation()
    }

    pub(crate) fn is_type_goal(&self) -> bool {
        self.item_node.is_type_goal()
    }

    pub(crate) fn get_type(&self) -> &SurrealItemType {
        self.item_node.get_type()
    }

    pub(crate) fn get_created(&self) -> &chrono::DateTime<chrono::Utc> {
        self.item_node.get_created()
    }

    pub(crate) fn get_item(&self) -> &Item<'s> {
        self.item_node.get_item()
    }
}
