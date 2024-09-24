pub(crate) mod base_data;
pub(crate) mod calculated_data;
pub(crate) mod data_storage;
pub(crate) mod display;
pub(crate) mod menu;
pub(crate) mod new_item;
pub(crate) mod new_time_spent;
mod node;
pub(crate) mod systems;

use std::{
    env,
    time::{Duration, SystemTime},
};

use tokio::sync::mpsc;

use crate::{
    data_storage::surrealdb_layer::data_layer_commands::data_storage_start_and_run,
    menu::inquire::do_now_list_menu::present_normal_do_now_list_menu,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let commands_in_flight_limit = 20;
    let (send_to_data_storage_layer_tx, have_data_storage_layer_use_to_receive_rx) =
        mpsc::channel(commands_in_flight_limit);

    let data_storage_join_handle = tokio::spawn(async move {
        data_storage_start_and_run(
            have_data_storage_layer_use_to_receive_rx,
            "file://c:/.on_purpose.db", //TODO: Get a default file location that works for both Linux and Windows
        )
        .await
    });

    //If the current executable is more than 3 months old print a message that there is probably a newer version available
    let exe_path = env::current_exe().unwrap();
    let exe_metadata = exe_path.metadata().unwrap();
    let exe_modified = exe_metadata.modified().unwrap();
    let now = SystemTime::now();
    let three_months = Duration::from_secs(60 * 60 * 24 * 30 * 3);
    if now.duration_since(exe_modified).unwrap() > three_months {
        println!("This version of On-Purpose is more than 3 months old. You may want to check for a newer version at https://github.com/rchriste/OnPurpose/releases");
    }

    loop {
        match present_normal_do_now_list_menu(&send_to_data_storage_layer_tx).await {
            Result::Ok(..) => (),
            Result::Err(..) => break,
        };

        if data_storage_join_handle.is_finished() {
            println!("Data Storage Layer closed early, unexpectedly");
        }
    }

    drop(send_to_data_storage_layer_tx);

    print!("Waiting for data storage layer to exit...");
    data_storage_join_handle.await.unwrap();
    println!("Done");

    Ok(())
}
