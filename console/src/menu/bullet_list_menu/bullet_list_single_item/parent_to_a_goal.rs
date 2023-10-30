use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::item::{Item, ItemVecExtensions},
    display::display_item::DisplayItem,
    surrealdb_layer::DataLayerCommands,
};

pub(crate) async fn parent_to_a_goal(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items = surreal_tables.make_items();
    let goals = items.filter_just_hopes(&surreal_tables.surreal_specific_to_hopes);
    let list = goals
        .iter()
        .map(|x| DisplayItem::new(x.get_item()))
        .collect::<Vec<_>>();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(parent) => {
            let parent: &Item<'_> = parent.into();
            if parent.has_children() {
                todo!("I need to pick a priority for this item among the children of the parent");
            } else {
                send_to_data_storage_layer
                    .send(DataLayerCommands::ParentItemWithExistingItem {
                        child: parent_this.get_surreal_item().clone(),
                        parent: parent.get_surreal_item().clone(),
                    })
                    .await
                    .unwrap();
            }
        }
        Err(InquireError::OperationCanceled) => {
            todo!("Enter a new Goal / project")
        }
        Err(err) => {
            todo!("Error: {:?}", err);
        }
    }
}
