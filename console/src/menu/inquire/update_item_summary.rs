use crate::base_data::item::Item;
use inquire::{InquireError, Text};
use tokio::sync::mpsc::Sender;

use crate::data_storage::surrealdb_layer::data_layer_commands::DataLayerCommands;

pub(crate) async fn update_item_summary(
    item_to_update: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let new_summary = Text::new("Enter New Summary ⍠")
        .with_initial_value(item_to_update.get_summary())
        .prompt();
    match new_summary {
        Ok(new_summary) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateSummary(
                    item_to_update.get_surreal_record_id().clone(),
                    new_summary,
                ))
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => todo!("Handle return to caller"),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error: {:?}", err),
    }
}
