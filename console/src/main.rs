pub mod base_data;
mod bullet_list;
mod mentally_resident;
mod node;
mod surrealdb_layer;
mod top_menu;

use base_data::Item;
use inquire::Text;
use node::ToDoNode;
use surrealdb_layer::surreal_item::SurrealItem;
use tokio::sync::mpsc::{self, Sender};

use crate::{
    bullet_list::present_unfocused_bullet_list_menu,
    surrealdb_layer::{data_storage_start_and_run, DataLayerCommands},
};

//I get an error about lifetimes that I can't figure out when I refactor this to be a member function of NextStepNode and I don't understand why
fn create_next_step_parents<'a>(item: &'a ToDoNode<'a>) -> Vec<&'a Item<'a>> {
    let mut result = Vec::default();
    for i in item.larger.iter() {
        result.push(i.item);
        let parents = i.create_growing_parents();
        result.extend(parents.iter());
    }
    result
}

async fn update_item_summary(
    item_to_cover: SurrealItem,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let new_summary = Text::new("Enter New Summary ⍠").prompt().unwrap();
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemSummary(
            item_to_cover,
            new_summary,
        ))
        .await
        .unwrap()
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
            "file://~/.on_purpose.db",
        )
        .await
    });

    present_unfocused_bullet_list_menu(&send_to_data_storage_layer_tx).await;

    if data_storage_join_handle.is_finished() {
        println!("Data Storage Layer closed early, unexpectedly");
    }

    drop(send_to_data_storage_layer_tx);

    println!("Waiting for data storage layer to exit...");
    data_storage_join_handle.await.unwrap();

    Ok(())
}
