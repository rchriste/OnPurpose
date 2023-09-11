pub mod base_data;
mod node;
mod test_data;
mod bullet_list;

use std::fmt::Display;

use base_data::{NextStepItem, Linkage, Item};
use inquire::Select;
use node::{NextStepNode, GrowingNode};

use crate::{node::create_next_step_nodes, test_data::create_items, test_data::create_linkage};

fn is_covered(next_step_item: &NextStepItem, linkage: &Vec<Linkage<'_>>) -> bool {
    let next_step_item = Item::NextStepItem(&next_step_item);
    linkage.iter().any(|x| x.parent == next_step_item)
}


fn find_parents<'a>(item: &Item<'a>, linkage: &'a Vec<Linkage<'a>>) -> Vec<&'a Item<'a>>
{
    linkage.iter().filter_map(|x| {
        if &x.smaller == item {Some(&x.parent)}
        else {None}
    }).collect()
}

enum InquireBulletListSelection<'a> {
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

struct InquireBulletListItem<'a> {
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
    fn create_list(next_step_nodes: &'a Vec<NextStepNode<'a>>) -> Vec<InquireBulletListSelection<'a>>
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

fn create_next_step_parents<'a>(item: &'a NextStepNode) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(&i.item);
        let parents = create_growing_parents(&i);
        result.extend(parents.iter());
    }
    result
}

fn create_growing_parents<'a>(item: &'a GrowingNode) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(&i.item);
        let parents = create_growing_parents(&i);
        result.extend(parents.iter());
    }
    result
}

fn main() {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let test_data = create_items();
    let linkage = create_linkage(&test_data);

    let next_step_nodes = create_next_step_nodes(&test_data.next_steps, &linkage);

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    let selected = Select::new("Select one", inquire_bullet_list).prompt();

    let selected = selected.unwrap();

    println!("{} selected", selected);
}