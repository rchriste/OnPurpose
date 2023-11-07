use chrono::{DateTime, Local};
use surrealdb::sql::Thing;

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    surrealdb_layer::surreal_item::{ItemType, Staging, SurrealItem},
};

#[derive(Debug)]
pub(crate) struct ItemNode<'s> {
    item: &'s Item<'s>,
    larger: Vec<GrowingItemNode<'s>>,
    smaller: Vec<ShrinkingItemNode<'s>>,
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
        all_items: &'s [&'s Item<'s>],
    ) -> Self {
        let visited = vec![];
        let parents = item.find_parents(coverings, all_items, &visited);
        let larger = create_growing_nodes(parents, coverings, all_items, visited.clone());
        let children = item.find_children(coverings, all_items, &visited);
        let smaller = create_shrinking_nodes(children, coverings, all_items, visited);

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

    pub(crate) fn is_mentally_resident(&self) -> bool {
        if self.item.get_staging() == &Staging::NotSet {
            //Look to parents for a setting
            self.get_larger().iter().any(|x| x.is_mentally_resident())
        } else {
            //Value is set so use it
            self.item.is_mentally_resident()
        }
    }

    pub(crate) fn get_larger(&self) -> &[GrowingItemNode] {
        &self.larger
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item.get_type()
    }

    pub(crate) fn is_type_action(&self) -> bool {
        if self.item.get_type() == &ItemType::Undeclared {
            //Look to parents for a setting
            self.get_larger().iter().any(|x| x.is_type_action())
        } else {
            //Value is set so use it
            self.item.is_type_action()
        }
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        let is_type_undeclared = self.item.is_type_undeclared();
        if is_type_undeclared && self.is_type_action() {
            //This type can be inferred from the parent so check that first
            false
        } else {
            is_type_undeclared
        }
    }

    pub(crate) fn is_type_goal(&self) -> bool {
        self.item.is_type_goal()
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item.is_type_motivation()
    }

    pub(crate) fn is_type_simple(&self) -> bool {
        self.item.is_type_simple()
    }

    pub(crate) fn has_active_children(&self) -> bool {
        self.smaller.iter().any(|x| !x.get_item().is_finished())
    }

    pub(crate) fn is_circumstance_focus_time(&self) -> bool {
        self.item.is_circumstance_focus_time()
    }

    pub(crate) fn get_estimated_focus_periods(&self) -> Option<u32> {
        self.item.get_estimated_focus_periods()
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        //I should probably change this to search through the parents as well, but going with this for now to maintain backwards compatibility with the code already written before I switched over to this ItemNode type
        self.item.is_there_notes()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        let is_staging_not_set = self.item.is_staging_not_set();
        if is_staging_not_set {
            //This type can be inferred from the parent so check that first
            !self.get_larger().iter().any(|x| !x.is_staging_not_set())
        } else {
            is_staging_not_set
        }
    }

    pub(crate) fn get_staging(&'s self) -> &'s Staging {
        let staging = self.item.get_staging();
        if staging == &Staging::NotSet {
            //This type can be inferred from the parent so check that first
            for parent in self.get_larger().iter() {
                let staging = parent.get_staging();
                if staging != &Staging::NotSet {
                    return staging;
                }
            }
            &Staging::NotSet
        } else {
            staging
        }
    }

    pub(crate) fn get_thing(&self) -> &'s Thing {
        self.item.get_thing()
    }
}

#[derive(Debug)]
pub(crate) struct GrowingItemNode<'s> {
    pub(crate) item: &'s Item<'s>,
    pub(crate) larger: Vec<GrowingItemNode<'s>>,
}

impl<'s> GrowingItemNode<'s> {
    pub(crate) fn create_growing_parents(&self) -> Vec<&'s Item<'s>> {
        let mut result = Vec::default();
        for i in self.larger.iter() {
            result.push(i.item);
            let parents = i.create_growing_parents();
            result.extend(parents.iter());
        }
        result
    }

    pub(crate) fn is_mentally_resident(&self) -> bool {
        if self.item.get_staging() == &Staging::NotSet {
            //Look to parents for a setting
            self.larger.iter().any(|x| x.is_mentally_resident())
        } else {
            //Value is set so use it
            self.item.is_mentally_resident()
        }
    }

    pub(crate) fn is_type_action(&self) -> bool {
        if self.item.get_type() == &ItemType::Undeclared {
            //Look to parents for a setting
            self.larger.iter().any(|x| x.is_type_action())
        } else {
            //Value is set so use it
            self.item.is_type_action()
        }
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        let is_staging_not_set = self.item.is_staging_not_set();
        if is_staging_not_set {
            //This type can be inferred from the parent so check that first
            !self.larger.iter().any(|x| !x.is_staging_not_set())
        } else {
            is_staging_not_set
        }
    }

    pub(crate) fn get_staging(&'s self) -> &'s Staging {
        let staging = self.item.get_staging();
        if staging == &Staging::NotSet {
            //This type can be inferred from the parent so check that first
            for parent in self.get_larger().iter() {
                let staging = parent.get_staging();
                if staging != &Staging::NotSet {
                    return staging;
                }
            }
            &Staging::NotSet
        } else {
            staging
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn get_larger(&self) -> &[GrowingItemNode] {
        &self.larger
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

#[derive(Debug)]
pub(crate) struct ShrinkingItemNode<'s> {
    item: &'s Item<'s>,
    _smaller: Vec<ShrinkingItemNode<'s>>,
}

impl<'s> ShrinkingItemNode<'s> {
    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }
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
                    item: x,
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
        item,
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

        let to_dos = items.filter_just_actions();
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
