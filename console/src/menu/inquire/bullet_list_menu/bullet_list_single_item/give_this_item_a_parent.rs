use std::cmp::Ordering;

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    display::display_item_node::DisplayItemNode,
    menu::inquire::{
        bullet_list_menu::bullet_list_single_item::ItemTypeSelection,
        select_higher_importance_than_this::select_higher_importance_than_this,
    },
    node::{item_node::ItemNode, Filter},
    surrealdb_layer::{data_layer_commands::DataLayerCommands, surreal_tables::SurrealTables},
};

pub(crate) async fn give_this_item_a_parent(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let all_items = base_data.get_items();
    let active_items = base_data.get_active_items();
    let time_spent_log = base_data.get_time_spent_log();
    let mut list = active_items
        .iter()
        .filter(|x| x.get_surreal_record_id() != parent_this.get_surreal_record_id())
        .map(|item| ItemNode::new(item, all_items, time_spent_log))
        //Collect the ItemNodes because they need a place to be so they don't go out of scope as DisplayItemNode
        //only takes a reference.
        .collect::<Vec<_>>();

    list.sort_by(|a, b| {
        //Motivational items first, then goals, then everything else.
        if a.is_type_motivation() && !b.is_type_motivation() {
            Ordering::Less
        } else if !a.is_type_motivation() && b.is_type_motivation() {
            Ordering::Greater
        } else if a.is_type_goal() && !b.is_type_goal() {
            Ordering::Less
        } else if !a.is_type_goal() && b.is_type_goal() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    let list = list.iter().map(DisplayItemNode::new).collect::<Vec<_>>();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(parent) => {
            let parent: &ItemNode<'_> = parent.get_item_node();

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
                    child: parent_this.get_surreal_record_id().clone(),
                    parent: parent.get_surreal_record_id().clone(),
                    higher_importance_than_this,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) | Err(InquireError::InvalidConfiguration(_)) => {
            parent_to_a_goal_or_motivation_new_goal_or_motivation(
                parent_this,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}

async fn parent_to_a_goal_or_motivation_new_goal_or_motivation(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ItemTypeSelection::create_list_goals_and_motivations();
    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(parent_to_a_goal_or_motivation_new_goal_or_motivation(
                parent_this,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            Box::pin(parent_to_a_goal_or_motivation_new_goal_or_motivation(
                parent_this,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                    child: parent_this.get_surreal_record_id().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            todo!("I need to go back to what first called this");
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}
