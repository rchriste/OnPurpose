pub mod bullet_list_single_item;

use std::fmt::Display;

use crate::{base_data::{NextStepItem, Item}, node::NextStepNode, create_next_step_parents};

pub struct InquireBulletListItem<'a> {
    bullet_item: &'a NextStepItem,
    parents: Vec<&'a Item<'a>>,
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.bullet_item.summary)?;
        for item in &self.parents {
            match item {
                Item::NextStepItem(next_step) => write!(f, "⬅ 🪜  {}", &next_step.summary)?,
                Item::ReviewItem(review) => write!(f, "⬅ 🧠 {}", &review.summary)?,
                Item::ReasonItem(reason) => write!(f, "⬅ 🎁 {}", &reason.summary)?,
            }
        }
        Ok(())
    }
}

impl<'a> From<InquireBulletListItem<'a>> for NextStepItem {
    fn from(value: InquireBulletListItem<'a>) -> Self {
        value.bullet_item.clone()
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub fn create_list(next_step_nodes: &'a Vec<NextStepNode<'a>>) -> Vec<InquireBulletListItem<'a>>
    {
        let mut result = Vec::with_capacity(next_step_nodes.capacity());
        result.extend(next_step_nodes.iter().map(|x| {
            InquireBulletListItem {
                bullet_item: x.next_step_item,
                parents: create_next_step_parents(&x),
            }
        }));
        result
    }
}
