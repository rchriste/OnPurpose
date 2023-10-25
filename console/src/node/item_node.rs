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
        let visited = vec![];
        let parents = item.find_parents(coverings, possible_parents, &visited);
        let larger = create_growing_nodes(parents, coverings, possible_parents, visited);

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
    visited: Vec<&'a Item<'a>>,
) -> Vec<GrowingItemNode<'a>> {
    items
        .iter()
        .map(|x| {
            let mut visited = visited.clone();
            visited.push(x);
            create_growing_node(x, coverings, possible_parents, visited)
        })
        .collect()
}

pub(crate) fn create_growing_node<'a>(
    item: &'a Item<'a>,
    coverings: &'a [Covering<'a>],
    all_items: &'a [&'a Item<'a>],
    visited: Vec<&'a Item<'a>>,
) -> GrowingItemNode<'a> {
    let parents = item.find_parents(coverings, all_items, &visited);
    let larger = create_growing_nodes(parents, coverings, all_items, visited);
    GrowingItemNode { item, larger }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;
    use crate::{
        base_data::{item::ItemVecExtensions, ItemType},
        node::to_do_node::create_to_do_nodes,
        surrealdb_layer::{
            surreal_covering::SurrealCovering,
            surreal_item::{Responsibility, SurrealItem},
            SurrealTables,
        },
    };

    #[test]
    fn when_coverings_causes_a_circular_reference_create_growing_node_detects_this_and_terminates()
    {
        let surreal_items = vec![
            SurrealItem {
                id: Some(("surreal_item", "1").into()),
                summary: "Main Item that covers something else".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "2").into()),
                summary: "Item that is covered by main item and the item this covers".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
            SurrealItem {
                id: Some(("surreal_item", "3").into()),
                summary: "Item that is covers the item it is covered by, circular reference".into(),
                finished: None,
                item_type: ItemType::ToDo,
                smaller_items_in_priority_order: vec![],
                responsibility: Responsibility::default(),
                notes_location: Default::default(),
            },
        ];
        let surreal_coverings = vec![
            SurrealCovering {
                id: Some(("surreal_covering", "1").into()),
                smaller: surreal_items[0].id.as_ref().expect("set above").clone(),
                parent: surreal_items[1].id.as_ref().expect("set above").clone(),
            },
            SurrealCovering {
                id: Some(("surreal_covering", "2").into()),
                smaller: surreal_items[1].id.as_ref().expect("set above").clone(),
                parent: surreal_items[2].id.as_ref().expect("set above").clone(),
            },
            SurrealCovering {
                id: Some(("surreal_covering", "3").into()),
                smaller: surreal_items[2].id.as_ref().expect("set above").clone(),
                parent: surreal_items[1].id.as_ref().expect("set above").clone(),
            },
        ];
        let surreal_tables = SurrealTables {
            surreal_items,
            surreal_specific_to_hopes: vec![],
            surreal_coverings,
            surreal_required_circumstances: vec![],
            surreal_coverings_until_date_time: vec![],
            surreal_life_areas: vec![],
            surreal_routines: vec![],
        };
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_to_do_nodes(
            &to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            &wednesday_ignore,
            false,
        );

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(
            next_step_nodes
                .iter()
                .next()
                .unwrap()
                .create_next_step_parents()
                .len(),
            2
        );
    }
}
