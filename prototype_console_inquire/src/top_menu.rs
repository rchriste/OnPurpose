use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::surrealdb_layer::DataLayerCommands;

enum TopMenuSelection {
    CaptureToDo,
    ViewToDos,
    CaptureHope,
    ViewHopes,
    CaptureReason,
    ViewReasons,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::CaptureToDo => write!(f, "🗬 🗒️  Capture To Do  🗭"),
            TopMenuSelection::ViewToDos => write!(f, "👁 🗒️  View To Dos    👁"),
            TopMenuSelection::CaptureHope => write!(f, "🗬 🙏 Capture Hope   🗭"),
            TopMenuSelection::ViewHopes => write!(f, "👁 🙏 View Hopes     👁"),
            TopMenuSelection::CaptureReason => write!(f, "🗬 🎯 Capture Reason 🗭"),
            TopMenuSelection::ViewReasons => write!(f, "👁 🎯 View Reasons   👁")
        }
    }
}

fn make_list() -> Vec<TopMenuSelection>
{
    vec![
        TopMenuSelection::CaptureToDo,
        TopMenuSelection::ViewToDos,
        TopMenuSelection::CaptureHope,
        TopMenuSelection::ViewHopes,
        TopMenuSelection::CaptureReason,
        TopMenuSelection::ViewReasons,
    ]
}

pub async fn present_top_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let top_menu = make_list();

    let selection = Select::new("Select one", top_menu).prompt().unwrap();
    match selection {
        TopMenuSelection::CaptureToDo => capture_to_do(send_to_data_storage_layer).await,
        TopMenuSelection::CaptureHope => capture_hope().await,
        TopMenuSelection::ViewHopes => view_hopes().await,
        TopMenuSelection::ViewToDos => view_to_dos().await,
        TopMenuSelection::CaptureReason => capture_reason().await,
        TopMenuSelection::ViewReasons => view_reasons().await,
    }
}

async fn capture_to_do(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let new_next_step_text = Text::new("Enter To Do ⍠").prompt().unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::NewToDo(new_next_step_text))
        .await
        .unwrap();
}

async fn capture_hope() {
    todo!()
}

async fn view_hopes() {
    todo!()
}

async fn view_to_dos() {
    todo!()
}

async fn capture_reason() {
    todo!()
}

async fn view_reasons() {
    todo!()
}