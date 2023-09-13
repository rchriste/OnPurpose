use std::fmt::Display;

use crate::{base_data::{NextStepItem, Item}, node::NextStepNode, create_next_step_parents};

pub enum InquireBulletListSelection<'a> {
    BulletItem(InquireBulletListItem<'a>),
    Capture
}

impl<'a> Display for InquireBulletListSelection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InquireBulletListSelection::BulletItem(bullet_item) => bullet_item.fmt(f),
            InquireBulletListSelection::Capture => write!(f, "🗬  Capture 🗭"),
        }
    }
}

impl<'a> From<InquireBulletListItem<'a>> for InquireBulletListSelection<'a> {
    fn from(value: InquireBulletListItem<'a>) -> Self {
        InquireBulletListSelection::BulletItem(value)
    }
}

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

impl<'a> InquireBulletListItem<'a> {
    pub fn create_list(next_step_nodes: &'a Vec<NextStepNode<'a>>) -> Vec<InquireBulletListSelection<'a>>
    {
        let mut result = Vec::with_capacity(next_step_nodes.capacity() + 1);
        result.extend(next_step_nodes.iter().map(|x| {
            InquireBulletListItem {
                bullet_item: x.next_step_item,
                parents: create_next_step_parents(&x),
            }.into()
        }));
        result.push(InquireBulletListSelection::Capture);
        result
    }
}
