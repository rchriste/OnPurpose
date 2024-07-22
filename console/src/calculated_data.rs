use crate::{
    base_data::{in_the_moment_priority::InTheMomentPriorityWithItemAction, BaseData},
    node::{item_node::ItemNode, item_status::ItemStatus},
};
use ouroboros::self_referencing;

#[self_referencing]
pub(crate) struct CalculatedData {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    items_nodes: Vec<ItemNode<'this>>,

    #[borrows(items_nodes)]
    #[covariant]
    items_status: Vec<ItemStatus<'this>>,

    #[borrows(items_status, base_data, items_nodes)]
    #[covariant]
    in_the_moment_priorities: Vec<InTheMomentPriorityWithItemAction<'this>>,
}

impl CalculatedData {
    pub(crate) fn new_from_base_data(base_data: BaseData) -> Self {
        CalculatedDataBuilder {
            base_data,
            items_nodes_builder: |base_data| {
                base_data
                    .get_items()
                    .iter()
                    .map(|x| {
                        ItemNode::new(x, base_data.get_items(), base_data.get_time_spent_log())
                    })
                    .collect::<Vec<_>>()
            },
            items_status_builder: |item_nodes| {
                item_nodes
                    .iter()
                    .map(|x| ItemStatus::new(x, item_nodes))
                    .collect::<Vec<_>>()
            },
            in_the_moment_priorities_builder: |items_status, base_data, all_nodes| {
                let now_sql = (*base_data.get_now()).into();
                let all_items = base_data.get_items();
                let time_spent_log = base_data.get_time_spent_log();
                base_data
                    .get_surreal_in_the_moment_priorities()
                    .iter()
                    .map(|x| {
                        InTheMomentPriorityWithItemAction::new(
                            x,
                            &now_sql,
                            all_items,
                            all_nodes,
                            items_status,
                            time_spent_log,
                        )
                    })
                    .collect::<Vec<_>>()
            },
        }
        .build()
    }

    pub(crate) fn get_items_status(&self) -> &[ItemStatus] {
        self.borrow_items_status()
    }

    pub(crate) fn get_in_the_moment_priorities(&self) -> &[InTheMomentPriorityWithItemAction] {
        self.borrow_in_the_moment_priorities()
    }
}
