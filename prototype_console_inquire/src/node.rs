use crate::base_data::{Item, NextStepItem, LinkageWithReferences};

pub struct GrowingNode<'a> {
    pub item: &'a Item<'a>,
    pub larger: Vec<GrowingNode<'a>>,
}

impl<'a> GrowingNode<'a> {
    pub fn create_growing_parents(&self) -> Vec<&'a Item<'a>>
    {
        let mut result: Vec<&'a Item<'a>> = Vec::default();
        for i in self.larger.iter() {
            result.push(&i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub struct NextStepNode<'a> {
    pub next_step_item: &'a NextStepItem,
    pub larger: Vec<GrowingNode<'a>>
}

pub fn create_next_step_nodes<'a>(next_steps: &'a Vec<NextStepItem>, linkage: &'a Vec<LinkageWithReferences<'a>>) -> Vec<NextStepNode<'a>>
{
    next_steps.iter().filter_map(|x| {
        if !x.is_covered(&linkage) && !x.is_finished() {
            Some(create_next_step_node(x, &linkage))
        } else { None }
    }).collect()
}

pub fn create_next_step_node<'a>(next_step: &'a NextStepItem, linkage: &'a Vec<LinkageWithReferences<'a>>) -> NextStepNode<'a>
{
    let item = Item::NextStepItem(&next_step);
    let parents = item.find_parents(&linkage);
    let larger = create_growing_nodes(parents, &linkage);

    NextStepNode {
        next_step_item: &next_step,
        larger
    }
}

pub fn create_growing_nodes<'a>(items: Vec<&'a Item<'a>>, linkage: &'a Vec<LinkageWithReferences<'a>>) -> Vec<GrowingNode<'a>>
{
    items.iter().map(|x| create_growing_node(x, &linkage)).collect()
}

pub fn create_growing_node<'a>(item: &'a Item<'a>, linkage: &'a Vec<LinkageWithReferences<'a>>) -> GrowingNode<'a>
{
    let parents = item.find_parents(&linkage);
    let larger = create_growing_nodes(parents, linkage);
    GrowingNode {
        item,
        larger
    }
}
