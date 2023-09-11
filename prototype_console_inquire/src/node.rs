use crate::{base_data::{Item, NextStepItem, Linkage}, is_covered, find_parents};

pub struct GrowingNode<'a> {
    pub item: &'a Item<'a>,
    pub larger: Vec<GrowingNode<'a>>,
}

pub struct NextStepNode<'a> {
    pub next_step_item: &'a NextStepItem,
    pub larger: Vec<GrowingNode<'a>>
}

pub fn create_next_step_nodes<'a>(next_steps: &'a Vec<NextStepItem>, linkage: &'a Vec<Linkage<'a>>) -> Vec<NextStepNode<'a>>
{
    next_steps.iter().filter_map(|x| {
        if !is_covered(&x, &linkage) {
            Some(create_next_step_node(x, &linkage))
        } else { None }
    }).collect()
}

pub fn create_next_step_node<'a>(next_step: &'a NextStepItem, linkage: &'a Vec<Linkage<'a>>) -> NextStepNode<'a>
{
    let item = Item::NextStepItem(&next_step);
    let parents = find_parents(&item, &linkage);
    let larger = create_growing_nodes(parents, &linkage);

    NextStepNode {
        next_step_item: &next_step,
        larger
    }
}

pub fn create_growing_nodes<'a>(items: Vec<&'a Item<'a>>, linkage: &'a Vec<Linkage<'a>>) -> Vec<GrowingNode<'a>>
{
    items.iter().map(|x| create_growing_node(x, &linkage)).collect()
}

pub fn create_growing_node<'a>(item: &'a Item<'a>, linkage: &'a Vec<Linkage<'a>>) -> GrowingNode<'a>
{
    let parents = find_parents(item, &linkage);
    let larger = create_growing_nodes(parents, linkage);
    GrowingNode {
        item,
        larger
    }
}
