use chrono::Utc;
use inquire::{InquireError, Select};
use surrealdb::opt::RecordId;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    display::display_item::DisplayItem,
    new_item::NewItemBuilder,
    surrealdb_layer::{surreal_item::ItemType, DataLayerCommands},
};

pub(crate) async fn select_person_or_group(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Option<RecordId> {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    //I'm not sure if active items is correct or if the real problem is that I have persons and groups in
    //the item list rather than as a separate thing
    let person_or_group = active_items
        .iter()
        .copied()
        .filter(|x| x.is_person_or_group())
        .map(DisplayItem::new)
        .collect::<Vec<_>>();
    let selection = Select::new("Select a person or group |", person_or_group).prompt();
    match selection {
        Ok(selection) => Some(selection.get_surreal_record_id().clone()),
        Err(InquireError::OperationCanceled) => {
            select_person_or_group_new_person_or_group(send_to_data_storage_layer).await
        }
        Err(err) => todo!("{:?}", err),
    }
}

async fn select_person_or_group_new_person_or_group(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Option<RecordId> {
    let summary = inquire::Text::new("Enter a summary for the new person or group |")
        .prompt()
        .unwrap();

    let new_item = NewItemBuilder::default()
        .summary(summary.clone())
        .item_type(ItemType::PersonOrGroup)
        .build()
        .unwrap();
    send_to_data_storage_layer
        .send(DataLayerCommands::NewItem(new_item))
        .await
        .unwrap();

    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    let person_or_group = active_items
        .iter()
        .copied()
        .filter(|x| x.is_person_or_group())
        .filter(|x| x.get_summary() == summary)
        .collect::<Vec<_>>();
    assert_eq!(person_or_group.len(), 1);
    person_or_group
        .into_iter()
        .next()
        .map(|x| x.get_surreal_record_id().clone())
}
