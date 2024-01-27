use std::fmt::Display;

use crate::{
    node::{item_node::ItemNode, Filter},
    surrealdb_layer::surreal_item::Staging,
};

use super::display_item::DisplayItem;

pub struct DisplayItemNode<'s> {
    item_node: &'s ItemNode<'s>,
}

impl Display for DisplayItemNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_item = DisplayItem::new(self.item_node.get_item());

        let staging = self.get_staging();
        match staging {
            Staging::OnDeck { .. } => write!(f, "🔜 ")?,
            Staging::MentallyResident { .. } => write!(f, "🧠 ")?,
            Staging::Planned { .. } => write!(f, "📝 ")?,
            Staging::ThinkingAbout { .. } => write!(f, "🤔 ")?,
            Staging::Released { .. } => write!(f, "🪽 ")?,
            Staging::NotSet => write!(f, "❓ ")?,
        }

        if self.item_node.is_person_or_group() {
            write!(f, "Is {} around?", display_item)?;
        } else if self.item_node.is_goal() && !self.item_node.has_children(Filter::Active) {
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

    pub(crate) fn make_list(item_nodes: &'s [ItemNode<'s>]) -> Vec<DisplayItemNode<'s>> {
        item_nodes.iter().map(DisplayItemNode::new).collect()
    }

    pub(crate) fn get_staging(&self) -> &'s Staging {
        self.item_node.get_staging()
    }

    pub(crate) fn get_item_node(&self) -> &'s ItemNode<'s> {
        self.item_node
    }
}
