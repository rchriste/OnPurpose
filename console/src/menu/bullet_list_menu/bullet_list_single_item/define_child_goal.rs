use chrono::Utc;
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

pub(crate) async fn define_child_goals(
    wants_a_child: &Item<'_>,
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

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(child) => {
            let child: &Item<'_> = child.into();

            let higher_priority_than_this = if wants_a_child.has_active_children(active_items) {
                todo!("User needs to pick what item this should be before. Although if all of the children are finished then it should be fine to just put it at the end. Also there is probably common menu code to call for this purpose")
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_item().clone(),
                    parent: wants_a_child.get_surreal_item().clone(),
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
    _wants_a_child: &Item<'_>,
    _send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    todo!()
}
