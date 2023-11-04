use chrono::Local;
use ouroboros::self_referencing;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        simple::Simple,
        BaseData,
    },
    mentally_resident::create_hope_nodes,
    node::{
        hope_node::HopeNode,
        person_or_group_node::{create_person_or_group_nodes, PersonOrGroupNode},
        to_do_node::{create_to_do_nodes, ToDoNode}, item_node::ItemNode,
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

    #[borrows(base_data)]
    #[covariant]
    next_step_nodes: Vec<ToDoNode<'this>>,

    #[borrows(base_data)]
    #[covariant]
    person_or_group_nodes_that_cover_an_item: Vec<PersonOrGroupNode<'this>>,

    #[borrows(base_data)]
    #[covariant]
    simple_items: Vec<Simple<'this>>,
}

impl BulletList {
    pub(crate) fn new_unfocused_bullet_list(base_data: BaseData) -> Self {
        let current_date_time = Local::now();
        BulletListBuilder {
            base_data,
            item_nodes_builder: |base_data| {
                base_data.get_active_items().filter_just_undeclared_items().map(|x| {
                    let coverings = base_data.get_coverings();
                    let possible_parents = base_data.get_active_items();
                    ItemNode::new(x, coverings, possible_parents)
                }).collect::<Vec<_>>()
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
            next_step_nodes_builder: |base_data| {
                let current_date_time = current_date_time.clone();
                let active_items = base_data.get_active_items();
                let to_dos = active_items.filter_just_to_dos();
                create_to_do_nodes(
                    to_dos,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    active_items,
                    current_date_time,
                    false,
                )
                .collect::<Vec<_>>()
            },
            person_or_group_nodes_that_cover_an_item_builder: |base_data| {
                let active_items = base_data.get_active_items();
                let persons_or_groups = active_items.filter_just_persons_or_groups();
                let person_or_groups_that_cover_an_item = persons_or_groups
                    .into_iter()
                    .filter(|x| x.is_covering_another_item(base_data.get_coverings()));

                create_person_or_group_nodes(
                    person_or_groups_that_cover_an_item,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    active_items,
                    &current_date_time,
                    false,
                )
            },
            simple_items_builder: |base_data| {
                base_data.get_active_items().filter_just_simple_items()
            },
        }
        .build()
    }

    pub(crate) fn new_focused_bullet_list(base_data: BaseData) -> Self {
        let current_date_time = Local::now();
        BulletListBuilder {
            base_data,
            item_nodes_builder: |_| {
                vec![]
            },
            hope_nodes_needing_a_next_step_builder: |_| {
                vec![] //Hopes without a next step cannot be focus items
            },
            next_step_nodes_builder: |base_data| {
                let current_date_time = current_date_time.clone();
                let active_items = base_data.get_active_items();
                let to_dos = active_items.filter_just_to_dos();
                create_to_do_nodes(
                    to_dos,
                    base_data.get_coverings(),
                    base_data.get_coverings_until_date_time(),
                    active_items,
                    current_date_time,
                    true,
                )
                .collect::<Vec<_>>()
            },
            person_or_group_nodes_that_cover_an_item_builder: |_| {
                vec![] //Sync'ing up with someone cannot be a focus item
            },
            simple_items_builder: |_| {
                vec![] //Simple items cannot be a focus item
            },
        }.build()
    }

    pub(crate) fn get_bullet_list(
        &self,
    ) -> (
        &[ItemNode<'_>],
        &[Simple<'_>],
        &[PersonOrGroupNode<'_>],
        &[ToDoNode<'_>],
        &[HopeNode<'_>],
    ) {
        (
            self.borrow_item_nodes(),
            self.borrow_simple_items(),
            self.borrow_person_or_group_nodes_that_cover_an_item(),
            self.borrow_next_step_nodes(),
            self.borrow_hope_nodes_needing_a_next_step(),
        )
    }

    pub(crate) fn get_active_items(&self) -> &[&Item<'_>] {
        self.borrow_base_data().get_active_items()
    }
}

#[cfg(test)]
mod tests {}
