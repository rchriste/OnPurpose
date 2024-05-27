use crate::{
    base_data::BaseData,
    node::{
        item_highest_lap_count::ItemHighestLapCount, item_lap_count::ItemLapCount,
        item_node::ItemNode, item_status::ItemStatus,
    },
};
use chrono::{DateTime, Utc};
use ouroboros::self_referencing;

#[self_referencing]
pub(crate) struct CalculatedData {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    items_highest_lap_count: Vec<ItemHighestLapCount<'this>>,
}

impl CalculatedData {
    pub(crate) fn new_from_base_data(
        base_data: BaseData,
        current_date_time: &DateTime<Utc>,
    ) -> Self {
        CalculatedDataBuilder {
            base_data,
            items_highest_lap_count_builder: |base_data| {
                let all_items = base_data.get_items();
                let active_snoozed = base_data.get_active_snoozed();
                let all_item_nodes = all_items
                    .iter()
                    .map(|x| ItemNode::new(x, base_data.get_coverings(), active_snoozed, all_items))
                    .collect::<Vec<_>>();

                let time_spent_log = base_data.get_time_spent_log();
                let item_status = all_item_nodes
                    .iter()
                    .map(|x| {
                        ItemStatus::new(
                            x.clone(),
                            &all_item_nodes,
                            time_spent_log,
                            current_date_time,
                        )
                    })
                    .collect::<Vec<_>>();

                let items_lap_count = item_status
                    .iter()
                    .map(|x| ItemLapCount::new(x.clone(), &item_status))
                    .collect::<Vec<_>>();

                items_lap_count
                    .iter()
                    .map(|x| ItemHighestLapCount::new(x.clone(), &items_lap_count))
                    .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_items_highest_lap_count(&self) -> &[ItemHighestLapCount] {
        self.borrow_items_highest_lap_count()
    }
}
