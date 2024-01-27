pub(crate) mod edit_order_of_children_items;

use core::fmt;
use std::fmt::{Display, Formatter};

use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item},
    display::display_item_status::DisplayItemStatus,
    menu::bullet_list_menu::bullet_list_single_item::create_or_update_children::edit_order_of_children_items::edit_order_of_children_items,
    node::{item_status::ItemStatus, Filter},
    surrealdb_layer::DataLayerCommands,
};

use super::present_bullet_list_item_selected;

enum CreateOrUpdateChildrenItem {
    ConfigureSchedulingPolicyForChildren,
    AddANewChildItem,
    EditOrderOfChildrenItems,
    ReturnToBulletList,
    ReturnToParentItem,
}

impl Display for CreateOrUpdateChildrenItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CreateOrUpdateChildrenItem::AddANewChildItem => write!(f, "Add a new child item"),
            CreateOrUpdateChildrenItem::ConfigureSchedulingPolicyForChildren => {
                write!(f, "Configure scheduling policy for children")
            }
            CreateOrUpdateChildrenItem::EditOrderOfChildrenItems => {
                write!(f, "Edit order of children items")
            }
            CreateOrUpdateChildrenItem::ReturnToBulletList => write!(f, "Return to bullet list"),
            CreateOrUpdateChildrenItem::ReturnToParentItem => write!(f, "Return to parent item"),
        }
    }
}

impl CreateOrUpdateChildrenItem {
    pub(crate) fn get_list(item_status: &ItemStatus<'_>) -> Vec<CreateOrUpdateChildrenItem> {
        let mut result = vec![
            CreateOrUpdateChildrenItem::AddANewChildItem,
            CreateOrUpdateChildrenItem::ConfigureSchedulingPolicyForChildren,
        ];
        if item_status.get_smaller(Filter::Active).next() != None {
            result.push(CreateOrUpdateChildrenItem::EditOrderOfChildrenItems);
        }
        result.push(CreateOrUpdateChildrenItem::ReturnToBulletList);
        result.push(CreateOrUpdateChildrenItem::ReturnToParentItem);

        result
    }
}

#[async_recursion]
pub(crate) async fn create_or_update_children(
    item_status: &ItemStatus<'_>,
    all_item_status: &[ItemStatus<'_>],
    current_date_time: &DateTime<Utc>,
    all_coverings: &[Covering<'_>],
    all_snoozed: &[&CoveringUntilDateTime<'_>],
    all_items: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    if !item_status.has_children(Filter::Active) {
        println!("No children found");
    } else {
        println!("Children:");
        for child in item_status.get_smaller(Filter::Active) {
            let item_status = all_item_status
                .iter()
                .find(|x| x.get_item() == child.get_item())
                .expect("All are here");
            println!("  {}", DisplayItemStatus::new(item_status));
        }
    }
    let list = CreateOrUpdateChildrenItem::get_list(item_status);
    let selection = Select::new("Select an option", list).prompt();
    match selection {
        Ok(CreateOrUpdateChildrenItem::AddANewChildItem) => {
            println!("Add a new child item");
            todo!()
        }
        Ok(CreateOrUpdateChildrenItem::ConfigureSchedulingPolicyForChildren) => {
            todo!("Configure scheduling policy for children")
        }
        Ok(CreateOrUpdateChildrenItem::EditOrderOfChildrenItems) => {
            edit_order_of_children_items(item_status.get_item_node(), send_to_data_storage_layer)
                .await
        }
        Ok(CreateOrUpdateChildrenItem::ReturnToBulletList) => {
            println!("Return to bullet list");
            todo!()
        }
        Ok(CreateOrUpdateChildrenItem::ReturnToParentItem) => {
            println!("Return to parent item");
            todo!()
        }
        Err(InquireError::OperationCanceled) => {
            present_bullet_list_item_selected(
                item_status,
                all_item_status,
                current_date_time,
                all_coverings,
                all_snoozed,
                all_items,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => {
            todo!("Error: {:?}", err)
        }
    }
}
