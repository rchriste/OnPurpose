use chrono::{DateTime, Local, Utc};
use surrealdb::sql::Thing;

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    surrealdb_layer::surreal_item::{Facing, ItemType, Staging, SurrealItem},
};

use super::Filter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemNode<'s> {
    item: &'s Item<'s>,
    larger: Vec<GrowingItemNode<'s>>,
    smaller: Vec<ShrinkingItemNode<'s>>,
    snoozed_until: Vec<&'s DateTime<Local>>,
    facing: Vec<Facing>,
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
        snoozed: &'s [&'s CoveringUntilDateTime<'s>],
        all_items: &'s [Item<'s>],
    ) -> Self {
        let visited = vec![];
        let parents = item.find_parents(coverings, all_items, &visited);
        let larger = create_growing_nodes(parents, coverings, all_items, visited.clone());
        let children = item.find_children(coverings, all_items, &visited);
        let smaller = create_shrinking_nodes(children, coverings, all_items, visited);
        let snoozed_until = item.get_covered_by_date_time(snoozed);
        let item_facing = item.get_facing();
        let facing = if item_facing.is_empty() {
            //Look to parents for a setting
            larger
                .iter()
                .map(|x| x.get_facing())
                .filter(|x| !x.is_empty())
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
        } else {
            //Value is set so use it
            item_facing.to_vec()
        };
        ItemNode {
            item,
            larger,
            smaller,
            snoozed_until,
            facing,
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item.is_finished()
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

    pub(crate) fn get_smaller(
        &'s self,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = &'s ShrinkingItemNode<'s>> + 's + Send> {
        Box::new(self.smaller.iter().filter(move |x| match filter {
            Filter::All => true,
            Filter::Active => !x.item.is_finished(),
        }))
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn get_surreal_record_id(&self) -> &Thing {
        self.item.get_surreal_record_id()
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

    pub(crate) fn has_larger(&self, filter: Filter) -> bool {
        match filter {
            Filter::All => !self.larger.is_empty(),
            Filter::Active => self.larger.iter().any(|x| !x.item.is_finished()),
        }
    }

    pub(crate) fn get_larger(
        &'s self,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = &'s GrowingItemNode<'s>> + 's + Send> {
        match filter {
            Filter::All => Box::new(self.larger.iter()),
            Filter::Active => Box::new(self.larger.iter().filter(|x| !x.item.is_finished())),
        }
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item.get_type()
    }

    pub(crate) fn is_type_action(&self) -> bool {
        if self.item.get_type() == &ItemType::Undeclared {
            //Look to parents for a setting
            self.get_larger(Filter::Active).any(|x| x.is_type_action())
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

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        match filter {
            Filter::All => !self.smaller.is_empty(),
            Filter::Active => self.smaller.iter().any(|x| !x.get_item().is_finished()),
        }
    }

    pub(crate) fn is_there_notes(&self) -> bool {
        //I should probably change this to search through the parents as well, but going with this for now to maintain backwards compatibility with the code already written before I switched over to this ItemNode type
        self.item.is_there_notes()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item.is_staging_not_set()
    }

    pub(crate) fn get_staging(&'s self) -> &'s Staging {
        self.item.get_staging()
    }

    pub(crate) fn get_thing(&self) -> &'s Thing {
        self.item.get_thing()
    }

    pub(crate) fn is_responsibility_reactive(&self) -> bool {
        self.item.is_responsibility_reactive()
    }

    pub(crate) fn is_staging_mentally_resident(&self) -> bool {
        matches!(self.get_staging(), Staging::MentallyResident { .. })
    }

    pub(crate) fn get_snoozed_until(&'s self) -> &'s [&'s DateTime<Local>] {
        //TODO: snoozed_until should be DateTime<Utc> not local
        &self.snoozed_until
    }

    pub(crate) fn get_facing(&'s self) -> &'s Vec<Facing> {
        &self.facing
    }

    pub(crate) fn is_facing_undefined(&self) -> bool {
        self.get_facing().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GrowingItemNode<'s> {
    pub(crate) item: &'s Item<'s>,
    pub(crate) larger: Vec<GrowingItemNode<'s>>,
    facing: Vec<Facing>,
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

    pub(crate) fn is_type_action(&self) -> bool {
        if self.item.get_type() == &ItemType::Undeclared {
            //Look to parents for a setting
            self.larger.iter().any(|x| x.is_type_action())
        } else {
            //Value is set so use it
            self.item.is_type_action()
        }
    }

    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn get_facing(&'s self) -> &'s Vec<Facing> {
        &self.facing
    }

    pub(crate) fn get_node<'a>(&self, all_nodes: &'a [ItemNode<'a>]) -> &'a ItemNode<'a> {
        all_nodes
            .iter()
            .find(|x| x.get_item() == self.item)
            .expect("It should be in all nodes, programming error")
    }
}

pub(crate) fn create_growing_nodes<'a>(
    items: Vec<&'a Item<'a>>,
    coverings: &'a [Covering<'a>],
    possible_parents: &'a [Item<'a>],
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
                let item_facing = x.get_facing();
                let facing = item_facing.to_vec();
                GrowingItemNode {
                    item: x,
                    larger: vec![],
                    facing,
                }
            }
        })
        .collect()
}

pub(crate) fn create_growing_node<'a>(
    item: &'a Item<'a>,
    coverings: &'a [Covering<'a>],
    all_items: &'a [Item<'a>],
    visited: Vec<&'a Item<'a>>,
) -> GrowingItemNode<'a> {
    let parents = item.find_parents(coverings, all_items, &visited);
    let larger = create_growing_nodes(parents, coverings, all_items, visited);
    let item_facing = item.get_facing();
    let facing = if item_facing.is_empty() {
        //Look to parents for a setting
        larger
            .iter()
            .map(|x| x.get_facing())
            .filter(|x| !x.is_empty())
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
    } else {
        //Value is set so use it
        item_facing.to_vec()
    };
    GrowingItemNode {
        item,
        larger,
        facing,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShrinkingItemNode<'s> {
    item: &'s Item<'s>,
    smaller: Vec<ShrinkingItemNode<'s>>,
}

impl<'s> ShrinkingItemNode<'s> {
    pub(crate) fn get_item(&self) -> &'s Item<'s> {
        self.item
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item.is_finished()
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.smaller.iter().any(|x| !x.is_finished())
    }

    pub(crate) fn when_finished(&self) -> Option<DateTime<Utc>> {
        self.item.when_finished()
    }

    pub(crate) fn get_staging(&self) -> &Staging {
        self.item.get_staging()
    }
}

pub(crate) fn create_shrinking_nodes<'a>(
    items: Vec<&'a Item<'a>>,
    coverings: &'a [Covering<'a>],
    possible_children: &'a [Item<'a>],
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
                    smaller: vec![],
                }
            }
        })
        .collect()
}

pub(crate) fn create_shrinking_node<'a>(
    item: &'a Item<'a>,
    coverings: &'a [Covering<'a>],
    all_items: &'a [Item<'a>],
    visited: Vec<&'a Item<'a>>,
) -> ShrinkingItemNode<'a> {
    let children = item.find_children(coverings, all_items, &visited);
    let smaller = create_shrinking_nodes(children, coverings, all_items, visited);
    ShrinkingItemNode { item, smaller }
}

#[cfg(test)]
mod tests {
    use crate::{
        base_data::item::ItemVecExtensions,
        node::{item_node::ItemNode, Filter},
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
                .item_type(ItemType::Action)
                .build()
                .unwrap(),
            SurrealItemBuilder::default()
                .id(Some(("surreal_item", "2").into()))
                .summary("Item that is covered by main item and the item this covers")
                .item_type(ItemType::Action)
                .build()
                .unwrap(),
            SurrealItemBuilder::default()
                .id(Some(("surreal_item", "3").into()))
                .summary("Item that is covers the item it is covered by, circular reference")
                .item_type(ItemType::Action)
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
        let coverings = surreal_tables.make_coverings(&active_items);
        let coverings_until_date_time =
            surreal_tables.make_coverings_until_date_time(&active_items);
        let active_snoozed = coverings_until_date_time.iter().collect::<Vec<_>>();

        let to_dos = items.filter_just_actions();
        let next_step_nodes = to_dos
            .map(|x| ItemNode::new(x, &coverings, &active_snoozed, &items))
            .filter(|x| !x.has_children(Filter::Active))
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
