use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    display::display_item::DisplayItem,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

pub(crate) async fn state_a_smaller_next_step(
    selected_item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables);
    let active_items = base_data
        .get_active_items()
        .iter()
        .filter(|x| x != &&selected_item)
        .copied()
        .collect::<Vec<_>>();
    let mut list = Vec::default();
    if selected_item.is_motivation() {
        list.extend(active_items.filter_just_motivations().map(DisplayItem::new));
    }
    if selected_item.is_goal() || selected_item.is_motivation() {
        list.extend(active_items.filter_just_goals().map(DisplayItem::new));
    }
    if selected_item.is_action() || selected_item.is_goal() || selected_item.is_motivation() {
        list.extend(active_items.filter_just_actions().map(DisplayItem::new));
    }

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(child) => {
            let parent = selected_item;
            let child: &Item<'_> = child.into();

            let higher_priority_than_this = if parent.has_children() {
                todo!("User needs to pick what item this should be before. Although if all of the children are finished then it should be fine to just put it at the end. Also there is probably common menu code to call for this purpose")
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_item().clone(),
                    parent: parent.get_surreal_item().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            state_a_smaller_next_step_new_item(selected_item, send_to_data_storage_layer).await;
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}

pub(crate) async fn state_a_smaller_next_step_new_item(
    _selected_item: &Item<'_>,
    _send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    todo!()
}
