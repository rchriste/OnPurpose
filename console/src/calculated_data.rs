use crate::{
    base_data::{
        covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item, BaseData,
    },
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

                all_item_nodes
                    .iter()
                    .map(|x| ItemStatus::new(x.clone(), &all_item_nodes, current_date_time))
                    .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_item_status(&self) -> &[ItemStatus] {
        self.borrow_item_status()
    }

    pub(crate) fn get_active_items(&self) -> &[&Item] {
        self.borrow_base_data().get_active_items()
    }

    pub(crate) fn get_coverings(&self) -> &[Covering] {
        self.borrow_base_data().get_coverings()
    }

    pub(crate) fn get_active_snoozed(&self) -> &[&CoveringUntilDateTime] {
        self.borrow_base_data().get_active_snoozed()
    }
}
