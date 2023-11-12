use std::fmt::Display;

use chrono::{DateTime, Utc};
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    menu::{
        bullet_list_menu::bullet_list_single_item::present_bullet_list_item_selected,
        top_menu::capture,
    },
    node::item_node::ItemNode,
    surrealdb_layer::DataLayerCommands,
};

use super::finish_bullet_item;

enum WorkingOnNow {
    CaptureAnUnrelatedItem,
    DefineFutureItemOntoParent,
    DefineSmallerNextStepToWorkOnNow,
    IFinished,
}

impl Display for WorkingOnNow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkingOnNow::CaptureAnUnrelatedItem => write!(f, "Capture an unrelated item"),
            WorkingOnNow::DefineFutureItemOntoParent => write!(f, "Define future item onto parent"),
            WorkingOnNow::DefineSmallerNextStepToWorkOnNow => {
                write!(f, "Define smaller next step to work on now")
            }
            WorkingOnNow::IFinished => write!(f, "I finished"),
        }
    }
}

impl WorkingOnNow {
    fn make_list(currently_working_on: &ItemNode<'_>) -> Vec<Self> {
        let mut list = vec![
            WorkingOnNow::CaptureAnUnrelatedItem,
            WorkingOnNow::IFinished,
            WorkingOnNow::DefineSmallerNextStepToWorkOnNow,
        ];
        if currently_working_on.has_larger() {
            list.push(WorkingOnNow::DefineFutureItemOntoParent);
        }
        list
    }
}

pub(crate) async fn starting_to_work_on_this_now(
    currently_working_on: &ItemNode<'_>,
    current_date_time: &DateTime<Utc>,
    all_coverings: &[Covering<'_>],
    all_snoozed: &[&CoveringUntilDateTime<'_>],
    all_items: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = WorkingOnNow::make_list(currently_working_on);

    let selection = Select::new("Select from the below list", list).prompt();
    match selection {
        Ok(WorkingOnNow::CaptureAnUnrelatedItem) => {
            capture(send_to_data_storage_layer).await;
        }
        Ok(WorkingOnNow::DefineFutureItemOntoParent) => {
            todo!("Define future item onto parent")
        }
        Ok(WorkingOnNow::DefineSmallerNextStepToWorkOnNow) => {
            todo!("Define smaller next step")
        }
        Ok(WorkingOnNow::IFinished) => {
            finish_bullet_item(
                currently_working_on,
                all_coverings,
                all_snoozed,
                all_items,
                current_date_time,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => {
            present_bullet_list_item_selected(
                currently_working_on,
                current_date_time,
                all_coverings,
                all_snoozed,
                all_items,
                send_to_data_storage_layer,
            )
            .await;
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}
