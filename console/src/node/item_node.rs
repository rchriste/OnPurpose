use crate::base_data::{item::Item, Covering};

pub(crate) struct ItemNode<'s> {
    pub(crate) item: &'s Item<'s>,
    pub(crate) larger: Vec<GrowingItemNode<'s>>,
}

impl<'s> ItemNode<'s> {
    pub(crate) fn new(
        item: &'s Item<'s>,
        coverings: &'s [Covering<'s>],
        possible_parents: &'s [&'s Item<'s>],
    ) -> Self {
        let parents = item.find_parents(coverings, possible_parents);
        let larger = create_growing_nodes(parents, coverings, possible_parents);

        ItemNode { item, larger }
    }

    pub(crate) fn create_next_step_parents(&'s self) -> Vec<&'s Item<'s>> {
        //TODO: Rename to create_parent_chain
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item.summary
    }

    pub(crate) fn get_larger(&self) -> &[GrowingItemNode<'s>] {
        &self.larger
    }
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn when_coverings_causes_a_circular_reference_create_growing_node_detects_this_and_terminates()
    {
        todo!("I am testing create_growing_node specifically")
    }
}
