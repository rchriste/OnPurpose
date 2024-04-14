use crate::{
    base_data::BaseData,
    node::{item_node::ItemNode, item_status::ItemStatus},
};
use chrono::{DateTime, Utc};
use ouroboros::self_referencing;

#[self_referencing]
pub(crate) struct CalculatedData {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    item_status: Vec<ItemStatus<'this>>,
}

impl CalculatedData {
    pub(crate) fn new_from_base_data(
        base_data: BaseData,
        current_date_time: &DateTime<Utc>,
    ) -> Self {
        CalculatedDataBuilder {
            base_data,
            item_status_builder: |base_data| {
                let all_items = base_data.get_items();
                let active_snoozed = base_data.get_active_snoozed();
                let all_item_nodes = all_items
                    .iter()
                    .map(|x| ItemNode::new(x, base_data.get_coverings(), active_snoozed, all_items))
                    .collect::<Vec<_>>();

                let time_spent_log = base_data.get_time_spent_log();
                all_item_nodes
                    .iter()
                    .map(|x| {
                        ItemStatus::new(
                            x.clone(),
                            &all_item_nodes,
                            time_spent_log,
                            current_date_time,
                        )
                    })
                    .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_item_status(&self) -> &[ItemStatus] {
        self.borrow_item_status()
    }
}
