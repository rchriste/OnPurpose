use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    display::display_item::DisplayItem,
    menu::select_higher_priority_than_this::select_higher_priority_than_this,
    node::item_node::ItemNode,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

use super::ItemTypeSelection;

pub(crate) async fn define_child_goals(
    wants_a_child: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    let list = active_items
        .filter_just_goals()
        .map(DisplayItem::new)
        .collect::<Vec<_>>();

    let selection = Select::new("Select from the below list", list).prompt();
    match selection {
        Ok(child) => {
            let child: &Item<'_> = child.into();

            let higher_priority_than_this = if wants_a_child.has_active_children() {
                let items = wants_a_child
                    .get_smaller()
                    .iter()
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_record_id().clone(),
                    parent: wants_a_child.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled) => {
            define_child_goals_new_goal(wants_a_child, send_to_data_storage_layer).await;
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
    //TODO: I need to update this to ask if you want to define another child goal after you define one of them or stop
}

pub(crate) async fn define_child_goals_new_goal(
    wants_a_child: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list_just_goals();

    let selection = Select::new("Select from the below list", list).prompt();
    match selection {
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            let higher_priority_than_this = if wants_a_child.has_active_children() {
                let items = wants_a_child
                    .get_smaller()
                    .iter()
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewChildItem {
                    child: new_item,
                    parent: wants_a_child.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}
