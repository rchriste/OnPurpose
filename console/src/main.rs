pub(crate) mod base_data;
mod change_routine;
pub(crate) mod display;
mod mentally_resident;
pub(crate) mod menu;
pub(crate) mod new_item;
mod node;
mod surrealdb_layer;
pub(crate) mod systems;

use base_data::item::ItemVecExtensions;
use inquire::Text;
use surrealdb_layer::surreal_item::SurrealItem;
use tokio::sync::mpsc::{self, Sender};

use crate::{
    menu::bullet_list_menu::present_unfocused_bullet_list_menu,
    surrealdb_layer::{data_storage_start_and_run, DataLayerCommands},
};

#[must_use]
pub(crate) enum UnexpectedNextMenuAction {
    Back,
    Close,
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

    convert_covering_to_a_child(&send_to_data_storage_layer_tx).await;
    present_unfocused_bullet_list_menu(&send_to_data_storage_layer_tx).await;

    if data_storage_join_handle.is_finished() {
        println!("Data Storage Layer closed early, unexpectedly");
    }

    drop(send_to_data_storage_layer_tx);

    println!("Waiting for data storage layer to exit...");
    data_storage_join_handle.await.unwrap();

    Ok(())
}

async fn convert_covering_to_a_child(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);
    let active_items = items.filter_active_items();
    for item in active_items {
        if item.has_children() {
            continue;
        }
        let items_covered = item.get_covering_another_item(&coverings);
        if items_covered.len() == 1 {
            let item_covered = items_covered[0];
            assert!(item_covered != item); //Make sure the code is correct and I don't have the same item covering itself
            let these_coverings = coverings
                .iter()
                .filter(|x| x.parent == item_covered && x.smaller == item)
                .collect::<Vec<_>>();
            assert!(!these_coverings.is_empty()); //Make sure the code is correct and I have the order right to find the covering
            assert!(these_coverings.len() == 1); //Make sure the code is correct and I don't have the same item covering itself
            println!(
                "Converting {} to a child of {}",
                item.summary, item_covered.summary
            );
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: item.get_surreal_item().clone(),
                    parent: item_covered.get_surreal_item().clone(),
                })
                .await
                .unwrap();
            for covering in these_coverings {
                send_to_data_storage_layer
                    .send(DataLayerCommands::RemoveCoveringItem(
                        covering.get_surreal_covering().clone(),
                    ))
                    .await
                    .unwrap();
                println!("Removing covering {:?}", covering.get_surreal_covering().id);
            }
        }
    }
}
