use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use ouroboros::self_referencing;

use crate::{
    base_data::{
        covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item, BaseData,
    },
    node::item_node::ItemNode,
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

                //Note that some of these bottom items might be from detecting a circular dependency
                let mut all_leaf_nodes = all_item_nodes
                    .into_iter()
                    .filter(|x| x.get_smaller().is_empty())
                    //Person or group items without a parent, meaning a reason for being on the list,
                    // should be filtered out.
                    .filter(|x| !x.is_person_or_group() || !x.get_larger().is_empty())
                    .collect::<Vec<_>>();

                //This first sort is just to give a stable order to the items. Another way of sorting would
                //work as well.
                all_leaf_nodes.sort_by(|a, b| a.get_thing().cmp(b.get_thing()));

                all_leaf_nodes.sort_by(|a, b| {
                    //Reactive items should be shown at the bottom so they are searchable
                    //TODO: I should have an item to state the purpose so the User knows they are not meant to do this
                    (if a.is_responsibility_reactive() {
                        if b.is_responsibility_reactive() {
                            Ordering::Equal
                        } else {
                            Ordering::Greater
                        }
                    } else if b.is_responsibility_reactive() {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    })
                    .then_with(|| {
                        //Snoozed items should be shown at the bottom so they are searchable
                        //TODO: I should have an item to state the purpose so the User knows they are not meant to do this, only if they need to search
                        if a.is_snoozed(*current_date_time) {
                            if b.is_snoozed(*current_date_time) {
                                Ordering::Equal
                            } else {
                                Ordering::Greater
                            }
                        } else if b.is_snoozed(*current_date_time) {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    })
                    .then_with(|| {
                        if a.is_type_undeclared() {
                            if b.is_type_undeclared() {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_type_undeclared() {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .then_with(|| {
                        if a.is_staging_not_set() {
                            if b.is_staging_not_set() {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_staging_not_set() {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .then_with(|| {
                        //TODO: Put mentally resident expired and on deck expired together into the same list that is then sorted so mentally resident is expired squared versus on deck that is just expired
                        if a.is_mentally_resident_expired(current_date_time)
                            || a.is_staging_on_deck_expired(current_date_time)
                        {
                            if b.is_mentally_resident_expired(current_date_time)
                                || b.is_staging_on_deck_expired(current_date_time)
                            {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_mentally_resident_expired(current_date_time)
                            || b.is_staging_on_deck_expired(current_date_time)
                        {
                            Ordering::Greater
                        } else {
                            a.get_staging().cmp(b.get_staging())
                        }
                    })
                    .then_with(|| {
                        //TODO: Have this be out of 1 rather than a percentage and then square the mentally resident number and keep the same the on deck expired number
                        let a_expired_percentage = a.expired_percentage(current_date_time);
                        let a_expired_amount = if a.is_staging_mentally_resident() {
                            f32::powf(a_expired_percentage / 100f32, 2f32)
                        } else {
                            a_expired_percentage / 100f32
                        };
                        let b_expired_percentage = b.expired_percentage(current_date_time);
                        let b_expired_amount = if b.is_staging_mentally_resident() {
                            f32::powf(b_expired_percentage / 100f32, 2f32)
                        } else {
                            b_expired_percentage / 100f32
                        };
                        if a_expired_amount > b_expired_amount {
                            Ordering::Less
                        } else if a_expired_percentage < b_expired_percentage {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                });

                all_leaf_nodes
                    .into_iter()
                    .map(BulletListReason::new)
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
        if item_node.is_staging_not_set() && !item_node.is_type_undeclared() {
            BulletListReason::SetStaging(item_node)
        } else {
            BulletListReason::WorkOn(item_node)
        }
    }
}

#[cfg(test)]
mod tests {}
