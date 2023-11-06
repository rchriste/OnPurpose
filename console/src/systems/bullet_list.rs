use std::cmp::Ordering;

use chrono::Local;
use itertools::chain;
use ouroboros::self_referencing;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData, covering::Covering,
    },
    node::item_node::{create_item_nodes, ItemNode},
};

#[self_referencing]
pub(crate) struct BulletList {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    item_nodes: Vec<ItemNode<'this>>,
}

impl BulletList {
    pub(crate) fn new_unfocused_bullet_list(base_data: BaseData) -> Self {
        let current_date_time = Local::now();
        BulletListBuilder {
            base_data,
            item_nodes_builder: |base_data| {
                let active_items = base_data.get_active_items();
                let persons_or_groups = active_items.filter_just_persons_or_groups();
                let person_or_groups_that_cover_an_item = persons_or_groups
                    .into_iter()
                    .filter(|x| x.is_covering_another_item(base_data.get_coverings()));
                let to_dos = active_items.filter_just_actions();

                let mentally_resident_hopes = active_items.filter_just_goals().filter(|x| {
                    (x.is_mentally_resident() || x.is_staging_not_set())
                        && (x.is_project() || x.is_permanence_not_set())
                });

                let bullet_list = chain!(
                    base_data.get_active_items().filter_just_undeclared_items(),
                    base_data.get_active_items().filter_just_simple_items(),
                    person_or_groups_that_cover_an_item,
                    to_dos,
                    mentally_resident_hopes
                );

                let mut item_nodes = create_item_nodes(
                    bullet_list,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    base_data.get_active_items(),
                    current_date_time,
                    false,
                )
                .filter(|x| !x.is_goal() || x.is_goal() && x.get_smaller().is_empty())
                .collect::<Vec<_>>();
            item_nodes.sort_by(|a, b| {
                let a_is_mentally_resident = a.is_mentally_resident();
                let b_is_mentally_resident = b.is_mentally_resident();
                if a_is_mentally_resident && !b_is_mentally_resident {
                    Ordering::Less
                } else if !a_is_mentally_resident && b_is_mentally_resident {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });
            item_nodes
            },
        }
        .build()
    }

    pub(crate) fn new_focused_bullet_list(base_data: BaseData) -> Self {
        let current_date_time = Local::now();
        BulletListBuilder {
            base_data,
            item_nodes_builder: |base_data| {
                let active_items = base_data.get_active_items();
                let to_dos = active_items.filter_just_actions();
                create_item_nodes(
                    to_dos,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    active_items,
                    current_date_time,
                    true,
                )
                .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_bullet_list(&self) -> &[ItemNode<'_>] {
        self.borrow_item_nodes()
    }

    pub(crate) fn get_active_items(&self) -> &[&Item<'_>] {
        self.borrow_base_data().get_active_items()
    }

    pub(crate) fn get_coverings(&self) -> &[Covering<'_>] {
        self.borrow_base_data().get_coverings()
    }
}

#[cfg(test)]
mod tests {}
