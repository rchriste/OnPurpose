pub(crate) mod to_do_node;

use crate::base_data::{item::Item, Covering};

pub(crate) struct GrowingItemNode<'a> {
    pub(crate) item: &'a Item<'a>,
    pub(crate) larger: Vec<GrowingItemNode<'a>>,
}

impl<'a> GrowingItemNode<'a> {
    pub(crate) fn create_growing_parents(&self) -> Vec<&'a Item<'a>> {
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }
}

pub(crate) fn create_growing_nodes<'a>(
    items: Vec<&'a Item<'a>>,
    coverings: &'a [Covering<'a>],
    possible_parents: &'a [&'a Item<'a>],
) -> Vec<GrowingItemNode<'a>> {
    items
        .iter()
        .map(|x| create_growing_node(x, coverings, possible_parents))
        .collect()
}

pub(crate) fn create_growing_node<'a>(
    item: &'a Item<'a>,
    coverings: &'a [Covering<'a>],
    all_items: &'a [&'a Item<'a>],
) -> GrowingItemNode<'a> {
    let parents = item.find_parents(coverings, all_items);
    let larger = create_growing_nodes(parents, coverings, all_items);
    GrowingItemNode { item, larger }
}

//TODO: I think most of these test cases should be moved to another file
#[cfg(test)]
mod tests {}
