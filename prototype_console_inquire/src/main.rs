pub mod base_data;
mod bullet_list;
mod mentally_resident;
mod node;
mod surrealdb_layer;
mod top_menu;

use base_data::SurrealItem;
use inquire::{InquireError, Select};
use node::ToDoNode;
use tokio::sync::mpsc;

use crate::{
    base_data::{convert_linkage_with_record_ids_to_references, SurrealItemVecExtensions},
    bullet_list::{
        bullet_list_single_item::present_bullet_list_item_selected, InquireBulletListItem,
    },
    node::create_to_do_nodes,
    surrealdb_layer::{data_storage_start_and_run, DataLayerCommands},
    top_menu::present_top_menu,
};

//I get an error about lifetimes that I can't figure out when I refactor this to be a member function of NextStepNode and I don't understand why
fn create_next_step_parents<'a>(item: &'a ToDoNode<'a>) -> Vec<&'a SurrealItem> {
    let mut result = Vec::default();
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
    let (send_to_data_storage_layer_tx, have_data_storage_layer_use_to_receive_rx) =
        mpsc::channel(commands_in_flight_limit);

    let data_storage_join_handle = tokio::spawn(async move {
        data_storage_start_and_run(
            have_data_storage_layer_use_to_receive_rx,
            "file:://~/.on_purpose.db",
        )
        .await
    });

    let (items, linkage) = DataLayerCommands::get_raw_data(&send_to_data_storage_layer_tx)
        .await
        .unwrap();

    let linkage = convert_linkage_with_record_ids_to_references(&linkage, &items);

    let to_dos = &items.filter_just_to_dos();
    let next_step_nodes = create_to_do_nodes(to_dos, &linkage);

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("Select one", inquire_bullet_list).prompt();

        match selected {
            Ok(selected) => {
                present_bullet_list_item_selected(selected, &send_to_data_storage_layer_tx).await
            }
            Err(err) => match err {
                InquireError::OperationCanceled => {
                    present_top_menu(&send_to_data_storage_layer_tx).await
                }
                _ => panic!("Unexpected InquireError of {}", err),
            },
        };
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(&send_to_data_storage_layer_tx).await
    }

    drop(send_to_data_storage_layer_tx);

    println!("Waiting for data storage layer to exit...");
    data_storage_join_handle.await.unwrap();

    Ok(())
}
