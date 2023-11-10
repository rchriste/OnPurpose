use std::cmp::Ordering;

use chrono::{DateTime, Local, Utc};
use itertools::chain;
use ouroboros::self_referencing;

use crate::{
    base_data::{
        covering::Covering,
        covering_until_date_time::CoveringUntilDateTime,
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    node::item_node::{create_item_nodes, ItemNode},
};

#[self_referencing]
pub(crate) struct BulletList {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    item_nodes: Vec<BulletListReason<'this>>,
}

impl BulletList {
    pub(crate) fn new_bullet_list(base_data: BaseData, current_date_time: &DateTime<Utc>) -> Self {
        BulletListBuilder {
            base_data,
            item_nodes_builder: |base_data| {
                let active_items = base_data.get_active_items();
                let active_snoozed = base_data.get_active_snoozed();
                let all_item_nodes = active_items
                    .iter()
                    .map(|x| {
                        ItemNode::new(x, base_data.get_coverings(), active_snoozed, active_items)
                    })
                    .collect::<Vec<_>>();

                let local_current_date_time: DateTime<Local> = (*current_date_time).into();
                //Note that some of these bottom items might be from detecting a circular dependency
                let mut all_leaf_nodes = all_item_nodes
                    .into_iter()
                    .filter(|x| x.get_smaller().is_empty())
                    //Reactive items should not be leaf nodes
                    .filter(|x| !x.is_responsibility_reactive())
                    //Snoozed items should not be shown
                    .filter(|x| !x.is_snoozed(local_current_date_time))
                    .collect::<Vec<_>>();

                //This first sort is just to give a stable order to the items. Another way of sorting would
                //work as well.
                all_leaf_nodes.sort_by(|a, b| a.get_thing().cmp(b.get_thing()));

                all_leaf_nodes.sort_by(|a, b| {
                    (if a.is_type_undeclared() || a.is_type_simple() {
                        if b.is_type_undeclared() || b.is_type_simple() {
                            Ordering::Equal
                        } else {
                            Ordering::Less
                        }
                    } else if b.is_type_undeclared() || b.is_type_simple() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    })
                    .then_with(|| {
                        if a.is_mentally_resident_expired(current_date_time) {
                            if b.is_mentally_resident_expired(current_date_time) {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_mentally_resident_expired(current_date_time) {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .then_with(|| {
                        if a.is_staging_on_deck_expired(current_date_time) {
                            if b.is_staging_on_deck_expired(current_date_time) {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_staging_on_deck_expired(current_date_time) {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .then_with(|| a.get_staging().cmp(b.get_staging()))
                    //TODO: I need to sort on_deck items by how much percentage time is left before they expire
                    //TODO: I need to sort mentally resident items by how long until they expire
                });

                //TODO: Sort order the first of all top items
                all_leaf_nodes
                    .into_iter()
                    .map(BulletListReason::new)
                    .collect::<Vec<_>>()
            },
        }
        .build()
    }

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
                    base_data.get_active_snoozed(),
                    base_data.get_active_items(),
                    current_date_time,
                    false,
                )
                .filter(|x| !x.is_goal() || x.is_goal() && x.get_smaller().is_empty())
                .collect::<Vec<_>>();
                item_nodes.sort_by(|a, b| a.get_staging().cmp(b.get_staging()));
                let mut item_nodes = item_nodes
                    .into_iter()
                    .map(BulletListReason::new)
                    .collect::<Vec<_>>();
                item_nodes.sort_by(|a, b| {
                    let a_is_set_staging = matches!(a, BulletListReason::SetStaging(_));
                    let b_is_set_staging = matches!(b, BulletListReason::SetStaging(_));
                    if a_is_set_staging && !b_is_set_staging {
                        Ordering::Less
                    } else if !a_is_set_staging && b_is_set_staging {
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
                    base_data.get_active_snoozed(),
                    active_items,
                    current_date_time,
                    true,
                )
                .map(BulletListReason::WorkOn)
                .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_bullet_list(&self) -> &[BulletListReason<'_>] {
        self.borrow_item_nodes()
    }

    pub(crate) fn get_active_items(&self) -> &[&Item<'_>] {
        self.borrow_base_data().get_active_items()
    }

    pub(crate) fn get_coverings(&self) -> &[Covering<'_>] {
        self.borrow_base_data().get_coverings()
    }

    pub(crate) fn get_active_snoozed(&self) -> &[&CoveringUntilDateTime<'_>] {
        self.borrow_base_data().get_active_snoozed()
    }
}

pub(crate) enum BulletListReason<'e> {
    SetStaging(ItemNode<'e>),
    WorkOn(ItemNode<'e>),
}

impl<'e> BulletListReason<'e> {
    pub(crate) fn new(item_node: ItemNode<'e>) -> Self {
        if item_node.is_staging_not_set()
            && !item_node.is_type_simple()
            && !item_node.is_type_undeclared()
        {
            BulletListReason::SetStaging(item_node)
        } else {
            BulletListReason::WorkOn(item_node)
        }
    }
}

#[cfg(test)]
mod tests {}
