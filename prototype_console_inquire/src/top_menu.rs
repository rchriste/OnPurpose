use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::surrealdb_layer::DataLayerCommands;


enum TopMenuSelection {
    CaptureNextStep
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::CaptureNextStep => write!(f, "🗬  Capture 🗭"),
        }
    }
}

pub async fn present_top_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let top_menu = vec![TopMenuSelection::CaptureNextStep];

    let selection = Select::new("Select one", top_menu).prompt().unwrap();
    match selection {
        TopMenuSelection::CaptureNextStep => capture_next_step(send_to_data_storage_layer).await,
    }
}

async fn capture_next_step(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_next_step_text = Text::new("Enter Next Step").prompt().unwrap();

    send_to_data_storage_layer.send(DataLayerCommands::NewNextStep(new_next_step_text)).await.unwrap();
}