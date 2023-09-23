pub mod base_data;
mod node;
mod test_data;
mod bullet_list;
mod surrealdb_layer;
mod top_menu;

use base_data::Item;
use inquire::{Select, InquireError};
use node::NextStepNode;
use tokio::sync::mpsc;

use crate::{
    node::create_next_step_nodes, 
    bullet_list::{InquireBulletListItem, bullet_list_single_item::present_bullet_list_item_selected}, 
    base_data::convert_linkage_with_record_ids_to_references, surrealdb_layer::{DataLayerCommands, data_storage_start_and_run}, top_menu::present_top_menu
};

//I get an error about lifetimes that I can't figure out when I refactor this to be a member function of NextStepNode and I don't understand why
fn create_next_step_parents<'a>(item: &'a NextStepNode<'a>) -> Vec<&'a Item<'a>>
{
    let mut result: Vec<&'a Item<'a>> = Vec::default();
    for i in item.larger.iter() {
        result.push(i.item);
        let parents = i.create_growing_parents();
        result.extend(parents.iter());
    }
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let commands_in_flight_limit = 20;
    let (send_to_data_storage_layer_tx, have_data_storage_layer_use_to_receive_rx) = mpsc::channel(commands_in_flight_limit);

    let data_storage_join_handle = tokio::spawn(async move {
        data_storage_start_and_run(have_data_storage_layer_use_to_receive_rx, "file:://~/.on_purpose.db").await
    });

    let (test_data, linkage) = DataLayerCommands::get_raw_data(&send_to_data_storage_layer_tx).await.unwrap();

    let linkage = convert_linkage_with_record_ids_to_references(&linkage, &test_data);

    let next_step_nodes = create_next_step_nodes(&test_data.next_steps, &linkage);

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    let selected = Select::new("Select one", inquire_bullet_list).prompt();

    match selected {
        Ok(selected) => present_bullet_list_item_selected(selected, &send_to_data_storage_layer_tx).await,
        Err(err) => match err {
            InquireError::OperationCanceled => present_top_menu(&send_to_data_storage_layer_tx).await,
            _ => panic!("Unexpected InquireError of {}", err)
        }
    };

    drop(send_to_data_storage_layer_tx);

    println!("Waiting for data storage layer to exit...");
    data_storage_join_handle.await.unwrap();

    Ok(())
}