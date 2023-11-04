use chrono::{DateTime, Local};

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    surrealdb_layer::surreal_item::SurrealItem,
};

pub(crate) struct ItemNode<'s> {
    pub(crate) item: &'s Item<'s>,
    pub(crate) larger: Vec<GrowingItemNode<'s>>,
    pub(crate) smaller: Vec<ShrinkingItemNode<'s>>,
}

impl<'a> From<&'a ItemNode<'a>> for &'a Item<'a> {
    fn from(value: &ItemNode<'a>) -> Self {
        value.item
    }
}

impl<'a> From<&'a ItemNode<'a>> for &'a SurrealItem {
    fn from(value: &'a ItemNode<'a>) -> Self {
        value.item.into()
    }
}

impl<'s> ItemNode<'s> {
    pub(crate) fn new(
        item: &'s Item<'s>,
        coverings: &'s [Covering<'s>],
        possible_parents: &'s [&'s Item<'s>],
    ) -> Self {
        let visited = vec![];
        let parents = item.find_parents(coverings, possible_parents, &visited);
        let larger = create_growing_nodes(parents, coverings, possible_parents, visited.clone());
        let children = item.find_children(coverings, possible_parents, &visited);
        let smaller = create_shrinking_nodes(children, coverings, possible_parents, visited);

        ItemNode {
            item,
            larger,
            smaller,
        }
    }

    pub(crate) fn create_parent_chain(&'s self) -> Vec<&'s Item<'s>> {
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }

    pub(crate) fn get_smaller(&'s self) -> &'s [ShrinkingItemNode<'s>] {
        &self.smaller
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn get_surreal_item(&self) -> &'s SurrealItem {
        self.item.surreal_item
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item.is_person_or_group()
    }

    pub(crate) fn is_maintenance(&self) -> bool {
        self.item.is_maintenance()
    }

    pub(crate) fn is_goal(&self) -> bool {
        self.item.is_goal()
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
            if !visited.contains(x) {
                //TODO: Add a unit test for this circular reference in smaller and bigger
                let mut visited = visited.clone();
                visited.push(x);
                create_growing_node(x, coverings, possible_parents, visited)
            } else {
                GrowingItemNode {
                    item: x,
                    larger: vec![],
                }
            }
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

pub(crate) struct ShrinkingItemNode<'a> {
    _item: &'a Item<'a>,
    _smaller: Vec<ShrinkingItemNode<'a>>,
}

pub(crate) fn create_shrinking_nodes<'a>(
    items: Vec<&'a Item<'a>>,
    coverings: &'a [Covering<'a>],
    possible_children: &'a [&'a Item<'a>],
    visited: Vec<&'a Item<'a>>,
) -> Vec<ShrinkingItemNode<'a>> {
    items
        .iter()
        .map(|x| {
            if !visited.contains(x) {
                //TODO: Add a unit test for this circular reference in smaller and bigger
                let mut visited = visited.clone();
                visited.push(x);
                create_shrinking_node(x, coverings, possible_children, visited)
            } else {
                ShrinkingItemNode {
                    _item: x,
                    _smaller: vec![],
                }
            }
        })
        .collect()
}

pub(crate) fn create_shrinking_node<'a>(
    item: &'a Item<'a>,
    coverings: &'a [Covering<'a>],
    all_items: &'a [&'a Item<'a>],
    visited: Vec<&'a Item<'a>>,
) -> ShrinkingItemNode<'a> {
    let children = item.find_children(coverings, all_items, &visited);
    let smaller = create_shrinking_nodes(children, coverings, all_items, visited);
    ShrinkingItemNode {
        _item: item,
        _smaller: smaller,
    }
}

pub(crate) fn create_item_nodes<'s>(
    create_nodes_from: impl Iterator<Item = &'s Item<'s>> + 's,
    coverings: &'s [Covering<'s>],
    coverings_until_date_time: &'s [CoveringUntilDateTime<'s>],
    items: &'s [&'s Item<'s>],
    current_date: DateTime<Local>,
    currently_in_focus_time: bool,
) -> impl Iterator<Item = ItemNode<'s>> + 's {
    create_nodes_from.filter_map(move |x| {
        if !x.is_covered(coverings, coverings_until_date_time, items, &current_date)
            && !x.is_finished()
            && x.is_circumstances_met(&current_date, currently_in_focus_time)
        {
            Some(ItemNode::new(x, coverings, items))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{
        base_data::item::ItemVecExtensions,
        node::item_node::create_item_nodes,
        surrealdb_layer::{
            surreal_covering::SurrealCovering,
            surreal_item::{ItemType, SurrealItemBuilder},
            surreal_tables::SurrealTablesBuilder,
        },
    };

    #[test]
    fn when_coverings_causes_a_circular_reference_create_growing_node_detects_this_and_terminates()
    {
        let surreal_items = vec![
            SurrealItemBuilder::default()
                .id(Some(("surreal_item", "1").into()))
                .summary("Main Item that covers something else")
                .item_type(ItemType::ToDo)
                .build()
                .unwrap(),
            SurrealItemBuilder::default()
                .id(Some(("surreal_item", "2").into()))
                .summary("Item that is covered by main item and the item this covers")
                .item_type(ItemType::ToDo)
                .build()
                .unwrap(),
            SurrealItemBuilder::default()
                .id(Some(("surreal_item", "3").into()))
                .summary("Item that is covers the item it is covered by, circular reference")
                .item_type(ItemType::ToDo)
                .build()
                .unwrap(),
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
        let surreal_tables = SurrealTablesBuilder::default()
            .surreal_items(surreal_items)
            .surreal_coverings(surreal_coverings)
            .build()
            .expect("no required fields");
        let items = surreal_tables.make_items();
        let active_items = items.filter_active_items();
        let coverings = surreal_tables.make_coverings(&items);
        let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

        let to_dos = items.filter_just_to_dos();
        let wednesday_ignore =
            DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                .unwrap()
                .into();
        let next_step_nodes = create_item_nodes(
            to_dos,
            &coverings,
            &coverings_until_date_time,
            &active_items,
            wednesday_ignore,
            false,
        )
        .collect::<Vec<_>>();

        assert_eq!(next_step_nodes.len(), 1);
        assert_eq!(
            next_step_nodes
                .iter()
                .next()
                .unwrap()
                .create_parent_chain()
                .len(),
            2
        );
    }
}
