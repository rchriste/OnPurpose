use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    bullet_list::present_unfocused_bullet_list_menu, mentally_resident::view_hopes,
    surrealdb_layer::DataLayerCommands,
};

enum TopMenuSelection {
    CaptureToDo,
    ViewToDos,
    CaptureHope,
    ViewHopes,
    CaptureMotivation,
    ViewMotivations,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::CaptureToDo => write!(f, "🗬 🗒️ Capture To Do      🗭"),
            TopMenuSelection::ViewToDos => write!(f, "👁 🗒️ View To Dos        👁"),
            TopMenuSelection::CaptureHope => write!(f, "🗬 🙏 Capture Hope       🗭"),
            TopMenuSelection::ViewHopes => {
                write!(f, "👁 🙏 View Hopes         👁")
            }
            TopMenuSelection::CaptureMotivation => {
                write!(f, "🗬 🎯 Capture Motivation 🗭")
            }
            TopMenuSelection::ViewMotivations => {
                write!(f, "👁 🎯 View Motivations   👁")
            }
        }
    }
}

impl TopMenuSelection {
    fn make_list() -> Vec<TopMenuSelection> {
        vec![
            TopMenuSelection::CaptureToDo,
            TopMenuSelection::ViewToDos,
            TopMenuSelection::CaptureHope,
            TopMenuSelection::ViewHopes,
            TopMenuSelection::CaptureMotivation,
            TopMenuSelection::ViewMotivations,
        ]
    }
}

pub async fn present_top_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let top_menu = TopMenuSelection::make_list();

    let selection = Select::new("", top_menu).prompt().unwrap();
    match selection {
        TopMenuSelection::CaptureToDo => capture_to_do(send_to_data_storage_layer).await,
        TopMenuSelection::CaptureHope => capture_hope(send_to_data_storage_layer).await,
        TopMenuSelection::ViewHopes => view_hopes(send_to_data_storage_layer).await,
        TopMenuSelection::ViewToDos => {
            present_unfocused_bullet_list_menu(send_to_data_storage_layer).await
        }
        TopMenuSelection::CaptureMotivation => capture_motivation(send_to_data_storage_layer).await,
        TopMenuSelection::ViewMotivations => view_motivations().await,
    }
}

async fn capture_to_do(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_next_step_text = Text::new("Enter To Do ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewToDo(new_next_step_text))
        .await
        .unwrap();
}

async fn capture_hope(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_hope_text = Text::new("Enter Hope ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewHope(new_hope_text))
        .await
        .unwrap();
}

async fn capture_motivation(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let summary_text = Text::new("Enter Motivation ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewMotivation(summary_text))
        .await
        .unwrap();
}

async fn view_motivations() {
    todo!()
}
