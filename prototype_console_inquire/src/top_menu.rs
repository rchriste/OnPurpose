use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::surrealdb_layer::DataLayerCommands;

enum TopMenuSelection {
    CaptureToDo,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::CaptureToDo => write!(f, "🗬  Capture To Do 🗭"),
        }
    }
}

pub async fn present_top_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let top_menu = vec![TopMenuSelection::CaptureToDo];

    let selection = Select::new("Select one", top_menu).prompt().unwrap();
    match selection {
        TopMenuSelection::CaptureToDo => capture_to_do(send_to_data_storage_layer).await,
    }
}

async fn capture_to_do(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_next_step_text = Text::new("Enter To Do").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewToDo(new_next_step_text))
        .await
        .unwrap();
}
