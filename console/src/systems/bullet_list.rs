use std::cmp::Ordering;

use ouroboros::self_referencing;

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    calculated_data::CalculatedData,
    node::{item_status::ItemStatus, Filter},
};

#[self_referencing]
pub(crate) struct BulletList {
    calculated_data: CalculatedData,

    #[borrows(calculated_data)]
    #[covariant]
    item_nodes: Vec<BulletListReason<'this>>, //TODO: Rename this to ordered_list or ordered_bullet_list or something to that effect
}

impl BulletList {
    pub(crate) fn new_bullet_list(calculated_data: CalculatedData) -> Self {
        BulletListBuilder {
            calculated_data,
            item_nodes_builder: |calculated_data| {
                //Note that some of these bottom items might be from detecting a circular dependency
                let mut all_leaf_status_nodes = calculated_data
                    .get_item_status()
                    .iter()
                    .filter(|x| !x.has_children(Filter::Active))
                    //Person or group items without a parent, meaning a reason for being on the list,
                    // should be filtered out.
                    .filter(|x| !x.is_person_or_group() || !x.has_larger(Filter::Active))
                    .cloned()
                    .collect::<Vec<_>>();

                //This first sort is just to give a stable order to the items. Another way of sorting would
                //work as well.
                all_leaf_status_nodes.sort_by(|a, b| a.get_thing().cmp(b.get_thing()));

                all_leaf_status_nodes.sort_by(|a, b| {
                    //Reactive items should be shown at the bottom so they are searchable TODO: I should show this in the UI that this is just for searching
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
                        if a.is_snoozed() {
                            if b.is_snoozed() {
                                Ordering::Equal
                            } else {
                                Ordering::Greater
                            }
                        } else if b.is_snoozed() {
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
                        if a.is_first_lap_finished() {
                            if b.is_first_lap_finished() {
                                Ordering::Equal
                            } else {
                                Ordering::Less
                            }
                        } else if b.is_first_lap_finished() {
                            Ordering::Greater
                        } else {
                            a.get_staging().cmp(b.get_staging())
                        }
                    })
                    .then_with(|| {
                        let a_lap_count = a.get_lap_count();
                        let a_expired_amount = if a.is_staging_mentally_resident() {
                            f32::powf(a_lap_count, 2f32)
                        } else {
                            a_lap_count
                        };
                        let b_lap_count = b.get_lap_count();
                        let b_expired_amount = if b.is_staging_mentally_resident() {
                            f32::powf(b_lap_count, 2f32)
                        } else {
                            b_lap_count
                        };
                        if a_expired_amount > b_expired_amount {
                            Ordering::Less
                        } else if a_expired_amount < b_expired_amount {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                });

                all_leaf_status_nodes
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
        self.borrow_calculated_data().get_active_items()
    }

    pub(crate) fn get_coverings(&self) -> &[Covering<'_>] {
        self.borrow_calculated_data().get_coverings()
    }

    pub(crate) fn get_active_snoozed(&self) -> &[&CoveringUntilDateTime<'_>] {
        self.borrow_calculated_data().get_active_snoozed()
    }

    pub(crate) fn get_all_item_status(&self) -> &[ItemStatus<'_>] {
        self.borrow_calculated_data().get_item_status()
    }
}

pub(crate) enum BulletListReason<'e> {
    SetStaging(ItemStatus<'e>),
    WorkOn(ItemStatus<'e>),
}

impl<'e> BulletListReason<'e> {
    pub(crate) fn new(item_status: ItemStatus<'e>) -> Self {
        if item_status.is_staging_not_set() && !item_status.is_type_undeclared() {
            BulletListReason::SetStaging(item_status)
        } else {
            BulletListReason::WorkOn(item_status)
        }
    }
}
