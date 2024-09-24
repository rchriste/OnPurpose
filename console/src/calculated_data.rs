use crate::{
    base_data::{
        in_the_moment_priority::InTheMomentPriorityWithItemAction, time_spent::TimeSpent, BaseData,
    },
    node::{item_node::ItemNode, item_status::ItemStatus},
    systems::do_now_list::current_mode::CurrentMode,
};
use ahash::HashMap;
use chrono::{DateTime, Utc};
use ouroboros::self_referencing;
use surrealdb::opt::RecordId;

#[self_referencing]
pub(crate) struct CalculatedData {
    base_data: BaseData,

    #[borrows(base_data)]
    #[covariant]
    items_nodes: HashMap<&'this RecordId, ItemNode<'this>>,

    #[borrows(items_nodes)]
    #[covariant]
    items_status: HashMap<&'this RecordId, ItemStatus<'this>>,

    #[borrows(items_status, base_data, items_nodes)]
    #[covariant]
    in_the_moment_priorities: Vec<InTheMomentPriorityWithItemAction<'this>>,

    #[borrows(base_data)]
    current_mode: CurrentMode,
}

impl CalculatedData {
    pub(crate) fn new_from_base_data(base_data: BaseData) -> Self {
        CalculatedDataBuilder {
            base_data,
            items_nodes_builder: |base_data| {
                base_data
                    .get_items()
                    .iter()
                    .map(|(k, x)| {
                        (
                            *k,
                            ItemNode::new(x, base_data.get_items(), base_data.get_time_spent_log()),
                        )
                    })
                    .collect::<HashMap<_, _>>()
            },
            items_status_builder: |item_nodes| {
                item_nodes
                    .iter()
                    .map(|(k, x)| (*k, ItemStatus::new(x, item_nodes)))
                    .collect::<HashMap<_, _>>()
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
            current_mode_builder: |base_data| {
                let surreal_current_modes = base_data.get_surreal_current_modes();
                assert!(surreal_current_modes.len() <= 1, "There should not be more than one current mode. In the future it would be great for this to be a silent error that just deletes the current mode and logs an error into an error log (that currently doesn't exist) clears it but for now this is an assert. Length is: {}", surreal_current_modes.len());
                let surreal_current_mode = surreal_current_modes.first();
                match surreal_current_mode {
                    None => CurrentMode::default(),
                    Some(surreal_current_mode) =>
                        CurrentMode::new(surreal_current_mode)
                }
            },
        }
        .build()
    }

    pub(crate) fn get_items_status(&self) -> &HashMap<&RecordId, ItemStatus> {
        self.borrow_items_status()
    }

    pub(crate) fn get_in_the_moment_priorities(&self) -> &[InTheMomentPriorityWithItemAction] {
        self.borrow_in_the_moment_priorities()
    }

    pub(crate) fn get_now(&self) -> &DateTime<Utc> {
        self.borrow_base_data().get_now()
    }

    pub(crate) fn get_time_spent_log(&self) -> &[TimeSpent] {
        self.borrow_base_data().get_time_spent_log()
    }

    pub(crate) fn get_current_mode(&self) -> &CurrentMode {
        self.borrow_current_mode()
    }
}
