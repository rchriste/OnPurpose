use std::cmp::Ordering;

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_tables::SurrealTables,
    },
    display::display_item_node::DisplayItemNode,
    menu::inquire::select_higher_importance_than_this::select_higher_importance_than_this,
    node::{item_node::ItemNode, item_status::ItemStatus, Filter},
};

use super::{
    urgency_plan::{prompt_for_dependencies_and_urgency_plan, AddOrRemove},
    ItemTypeSelection,
};

pub(crate) enum SelectAnItemSortingOrder {
    MotivationsFirst,
    NewestFirst,
}

pub(crate) async fn select_an_item<'a>(
    dont_show_these_items: Vec<&Item<'_>>,
    sorting_order: SelectAnItemSortingOrder,
    calculated_data: &'a CalculatedData,
) -> Result<Option<&'a ItemStatus<'a>>, ()> {
    let active_items = calculated_data
        .get_items_status()
        .iter()
        .filter(|x| !dont_show_these_items.iter().any(|y| x.get_item() == *y) && !x.is_finished())
        .collect::<Vec<_>>();
    let mut list = active_items
        .iter()
        .map(|x| DisplayItemNode::new(x.get_item_node()))
        .collect::<Vec<_>>();
    match sorting_order {
        SelectAnItemSortingOrder::MotivationsFirst => list.sort_by(|a, b| {
            if a.is_type_motivation() {
                if b.is_type_motivation() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else if a.is_type_goal() {
                if b.is_type_motivation() {
                    Ordering::Greater
                } else if b.is_type_goal() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else if b.is_type_motivation() || b.is_type_goal() {
                Ordering::Greater
            } else if a.get_type() == b.get_type() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
            .then_with(|| a.get_created().cmp(b.get_created()).reverse())
        }),
        SelectAnItemSortingOrder::NewestFirst => {
            list.sort_by(|a, b| a.get_created().cmp(b.get_created()).reverse())
        }
    }

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(selected_item) => Ok(active_items
            .into_iter()
            .find(|x| x.get_item() == selected_item.get_item())),
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => Ok(None),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

pub(crate) async fn state_a_smaller_action(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data);
    let selection = select_an_item(
        vec![selected_item.get_item()],
        SelectAnItemSortingOrder::NewestFirst,
        &calculated_data,
    )
    .await;

    match selection {
        Ok(Some(child)) => {
            let parent = selected_item;
            let child: &Item<'_> = child.get_item();

            let higher_importance_than_this = if parent.has_children(Filter::Active) {
                let items = parent
                    .get_children(Filter::Active)
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_importance_than_this(&items, None)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_record_id().clone(),
                    parent: parent.get_surreal_record_id().clone(),
                    higher_importance_than_this,
                })
                .await
                .unwrap();

            Ok(())
        }
        Ok(None) => state_a_child_action_new_item(selected_item, send_to_data_storage_layer).await,
        Err(()) => Err(()),
    }
}

pub(crate) async fn state_a_child_action_new_item(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(state_a_child_action_new_item(
                selected_item,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(item_type_selection) => {
            let mut new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            let higher_importance_than_this = if selected_item.has_children(Filter::Active) {
                let items = selected_item
                    .get_children(Filter::Active)
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_importance_than_this(&items, None)
            } else {
                None
            };
            let parent = selected_item;

            let (dependencies, urgency_plan) =
                prompt_for_dependencies_and_urgency_plan(None, send_to_data_storage_layer).await;
            let dependencies = dependencies.into_iter().map(|(a, b)|
                match a {
                    AddOrRemove::Add => b,
                    AddOrRemove::Remove => panic!("You are adding a new item there is nothing to remove so this case will never be hit"),
                }).collect::<Vec<_>>();
            new_item.dependencies = dependencies;
            new_item.urgency_plan = Some(urgency_plan);

            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewChildItem {
                    child: new_item,
                    parent: parent.get_surreal_record_id().clone(),
                    higher_importance_than_this,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}
