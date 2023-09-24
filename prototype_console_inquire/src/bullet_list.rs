pub mod bullet_list_single_item;

use std::fmt::Display;

use crate::{
    base_data::{Item, ItemType, SurrealItem, ToDo},
    create_next_step_parents,
    node::ToDoNode,
};

pub struct InquireBulletListItem<'a> {
    bullet_item: &'a ToDo<'a>, //TODO: This should be ToDoOrQuestion
    parents: Vec<&'a Item<'a>>,
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.bullet_item.summary)?;
        for item in &self.parents {
            match item.item_type {
                ItemType::ToDo => write!(f, "⬅ 🪜  ")?,
                ItemType::Hope => write!(f, "⬅ 🧠 ")?,
                ItemType::Reason => write!(f, "⬅ 🎁 ")?,
                ItemType::Question => todo!(),
            }
            write!(f, "{}", item.summary)?;
        }
        Ok(())
    }
}

impl<'a> From<InquireBulletListItem<'a>> for ToDo<'a> {
    fn from(value: InquireBulletListItem<'a>) -> Self {
        value.bullet_item.clone()
    }
}

impl<'a> From<InquireBulletListItem<'a>> for SurrealItem {
    fn from(value: InquireBulletListItem<'a>) -> Self {
        let surreal_item: &SurrealItem = value.bullet_item.into();
        surreal_item.clone()
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub fn create_list(next_step_nodes: &'a Vec<ToDoNode<'a>>) -> Vec<InquireBulletListItem<'a>> {
        let mut result = Vec::with_capacity(next_step_nodes.capacity());
        result.extend(next_step_nodes.iter().map(|x| InquireBulletListItem {
            bullet_item: x.to_do,
            parents: create_next_step_parents(x),
        }));
        result
    }
}
