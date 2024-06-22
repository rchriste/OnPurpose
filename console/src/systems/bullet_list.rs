use ouroboros::self_referencing;
use surrealdb::opt::RecordId;

use crate::{
    base_data::item::Item,
    calculated_data::CalculatedData,
    node::{item_highest_lap_count::ItemHighestLapCount, item_lap_count::ItemLapCount, Filter},
};

#[self_referencing]
pub(crate) struct BulletList {
    calculated_data: CalculatedData,

    #[borrows(calculated_data)]
    #[covariant]
    ordered_bullet_list: Vec<BulletListReason<'this>>,
}

impl BulletList {
    pub(crate) fn new_bullet_list_version_2(calculated_data: CalculatedData) -> Self {
        BulletListBuilder {
            calculated_data,
            ordered_bullet_list_builder: |calculated_data| {
                //Get all top level items
                let _all_top_parents = calculated_data
                    .get_items_highest_lap_count()
                    .iter()
                    .filter(|x| !x.has_larger(Filter::Active) && !x.is_finished())
                    .cloned()
                    .collect::<Vec<_>>();
                todo!()
            },
        }
        .build()
    }

    pub(crate) fn new_bullet_list_version_1(calculated_data: CalculatedData) -> Self {
        BulletListBuilder {
            calculated_data,
            ordered_bullet_list_builder: |calculated_data| {
                //Note that some of these bottom items might be from detecting a circular dependency
                let mut all_leaf_status_nodes = calculated_data
                    .get_items_highest_lap_count()
                    .iter()
                    .filter(|x| !x.is_snoozed())
                    .filter(|x| !x.is_finished())
                    //Person or group items without a parent, meaning a reason for being on the list,
                    // should be filtered out.
                    .filter(|x| {
                        !x.is_person_or_group()
                            || (x.is_person_or_group() && x.has_larger(Filter::Active))
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                //This first sort is just to give a stable order to the items. Another way of sorting would
                //work as well.
                all_leaf_status_nodes.sort_by(|a, b| a.get_thing().cmp(b.get_thing()));

                all_leaf_status_nodes.sort_by(|a, b| {
                    (a.get_priority_level().cmp(&b.get_priority_level())).then_with(|| {
                        b.get_lap_count()
                            .partial_cmp(&a.get_lap_count())
                            .expect("Lap count is never a weird NaN")
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

    pub(crate) fn get_ordered_bullet_list(&self) -> &[BulletListReason<'_>] {
        self.borrow_ordered_bullet_list()
    }

    pub(crate) fn get_all_items_highest_lap_count(&self) -> &[ItemHighestLapCount<'_>] {
        self.borrow_calculated_data().get_items_highest_lap_count()
    }
}

pub(crate) enum BulletListReason<'e> {
    SetStaging(ItemHighestLapCount<'e>),
    WorkOn(ItemHighestLapCount<'e>),
}

impl<'e> BulletListReason<'e> {
    pub(crate) fn new(item_lap_count: ItemHighestLapCount<'e>) -> Self {
        if item_lap_count.is_staging_not_set() && !item_lap_count.is_type_undeclared() {
            BulletListReason::SetStaging(item_lap_count)
        } else {
            BulletListReason::WorkOn(item_lap_count)
        }
    }

    pub(crate) fn get_item_lap_count(&'e self) -> &ItemLapCount<'e> {
        match self {
            BulletListReason::SetStaging(item_lap_count) => item_lap_count.get_item_lap_count(),
            BulletListReason::WorkOn(item_lap_count) => item_lap_count.get_item_lap_count(),
        }
    }

    pub(crate) fn get_item(&'e self) -> &'e Item<'e> {
        self.get_item_lap_count().get_item()
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.get_item_lap_count().get_lap_count()
    }

    pub(crate) fn get_surreal_record_id(&'e self) -> &'e RecordId {
        self.get_item().get_surreal_record_id()
    }
}
