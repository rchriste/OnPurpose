use async_recursion::async_recursion;
use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    display::display_item::DisplayItem,
    menu::{
        bullet_list_menu::bullet_list_single_item::set_staging::present_set_staging_menu,
        select_higher_priority_than_this::select_higher_priority_than_this,
    },
    node::item_node::ItemNode,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

use super::ItemTypeSelection;

pub(crate) async fn state_a_smaller_next_step(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data
        .get_active_items()
        .iter()
        .filter(|x| **x != selected_item.get_item())
        .copied()
        .collect::<Vec<_>>();
    let mut list = Vec::default();
    if selected_item.is_type_motivation() {
        list.extend(active_items.filter_just_motivations().map(DisplayItem::new));
    }
    if selected_item.is_type_goal() || selected_item.is_type_motivation() {
        list.extend(active_items.filter_just_goals().map(DisplayItem::new));
    }
    if selected_item.is_type_action()
        || selected_item.is_type_goal()
        || selected_item.is_type_motivation()
    {
        list.extend(active_items.filter_just_actions().map(DisplayItem::new));
    }

    let selection = Select::new("Select from the below list|", list).prompt();

    match selection {
        Ok(child) => {
            let parent = selected_item;
            let child: &Item<'_> = child.into();

            let higher_priority_than_this = if parent.has_active_children() {
                todo!("User needs to pick what item this should be before. Although if all of the children are finished then it should be fine to just put it at the end. Also there is probably common menu code to call for this purpose")
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_record_id().clone(),
                    parent: parent.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();

            println!(
                "Please update Staging for {}",
                DisplayItem::new(parent.get_item())
            );
            present_set_staging_menu(parent, send_to_data_storage_layer).await;
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            state_a_smaller_next_step_new_item(selected_item, send_to_data_storage_layer).await;
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}

#[async_recursion]
pub(crate) async fn state_a_smaller_next_step_new_item(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            state_a_smaller_next_step_new_item(selected_item, send_to_data_storage_layer).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            state_a_smaller_next_step_new_item(selected_item, send_to_data_storage_layer).await
        }
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            let higher_priority_than_this = if selected_item.has_active_children() {
                let items = selected_item
                    .get_smaller()
                    .iter()
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            let parent = selected_item;
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewChildItem {
                    child: new_item,
                    parent: parent.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();

            println!(
                "Please update Staging for {}",
                DisplayItem::new(parent.get_item())
            );
            present_set_staging_menu(parent, send_to_data_storage_layer).await;
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}
