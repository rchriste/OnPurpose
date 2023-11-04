use chrono::Local;
use itertools::chain;
use ouroboros::self_referencing;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    mentally_resident::create_hope_nodes,
    node::{
        hope_node::HopeNode,
        item_node::{create_item_nodes, ItemNode},
    },
};

#[self_referencing]
pub(crate) struct BulletList {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    item_nodes: Vec<ItemNode<'this>>,

    #[borrows(base_data)]
    #[covariant]
    hope_nodes_needing_a_next_step: Vec<HopeNode<'this>>,
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
                let to_dos = active_items.filter_just_to_dos();

                let bullet_list = chain!(
                    base_data.get_active_items().filter_just_undeclared_items(),
                    base_data.get_active_items().filter_just_simple_items(),
                    person_or_groups_that_cover_an_item,
                    to_dos,
                );

                create_item_nodes(
                    bullet_list,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    base_data.get_active_items(),
                    current_date_time,
                    false,
                )
                .collect::<Vec<_>>()
            },
            hope_nodes_needing_a_next_step_builder: |base_data| {
                let mentally_resident_hopes = base_data
                    .get_just_hopes()
                    .iter()
                    .filter(|x| x.is_mentally_resident() && x.is_project())
                    .collect::<Vec<_>>();
                let hope_nodes = create_hope_nodes(
                    &mentally_resident_hopes,
                    base_data.get_coverings(),
                    base_data.get_active_items(),
                );
                hope_nodes
                    .into_iter()
                    .filter(|x| x.next_steps.is_empty())
                    .collect()
            },
        }
        .build()
    }

    pub(crate) fn new_focused_bullet_list(base_data: BaseData) -> Self {
        let current_date_time = Local::now();
        BulletListBuilder {
            base_data,
            hope_nodes_needing_a_next_step_builder: |_| {
                vec![] //Hopes without a next step cannot be focus items
            },
            item_nodes_builder: |base_data| {
                let active_items = base_data.get_active_items();
                let to_dos = active_items.filter_just_to_dos();
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

    pub(crate) fn get_bullet_list(&self) -> (&[ItemNode<'_>], &[HopeNode<'_>]) {
        (
            self.borrow_item_nodes(),
            self.borrow_hope_nodes_needing_a_next_step(),
        )
    }

    pub(crate) fn get_active_items(&self) -> &[&Item<'_>] {
        self.borrow_base_data().get_active_items()
    }
}

#[cfg(test)]
mod tests {}
