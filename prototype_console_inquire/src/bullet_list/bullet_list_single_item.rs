mod cover_bullet_item;

use std::fmt::Display;

use inquire::{Select, InquireError, Editor};
use tokio::sync::mpsc::Sender;

use crate::{surrealdb_layer::DataLayerCommands, base_data::ToDo, bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item};

use super::InquireBulletListItem;

enum BulletListSingleItemSelection {
    ProcessAndFinish,
    Cover
}

impl Display for BulletListSingleItemSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulletListSingleItemSelection::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            BulletListSingleItemSelection::Cover => write!(f, "Cover ⼍"),
        }
    }
}

fn create_list() -> Vec<BulletListSingleItemSelection>
{
    vec![
        BulletListSingleItemSelection::ProcessAndFinish,
        BulletListSingleItemSelection::Cover,
    ]
}

pub async fn present_bullet_list_item_selected(menu_for: InquireBulletListItem<'_>, send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let list = create_list();

    let selection = Select::new("Select one", list).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => process_and_finish_bullet_item(menu_for.into(), send_to_data_storage_layer).await,
        Ok(BulletListSingleItemSelection::Cover) => cover_bullet_item(menu_for.into(), send_to_data_storage_layer).await,
        Err(InquireError::OperationCanceled) => (), //Nothing to do we just want to return to the bullet list
        Err(err) => panic!("Unexpected {}", err),
    }
}

async fn process_and_finish_bullet_item(to_do: ToDo, send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let user_processed_text = Editor::new("Process text").prompt().unwrap();
    send_to_data_storage_layer.send(DataLayerCommands::AddUserProcessedText(user_processed_text, to_do.clone())).await.unwrap();
    send_to_data_storage_layer.send(DataLayerCommands::FinishToDo(to_do)).await.unwrap();
}

