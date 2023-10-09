mod cover_bullet_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::ToDo,
    bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item,
    surrealdb_layer::{surreal_item::SurrealItem, DataLayerCommands},
    update_item_summary,
};

enum BulletListSingleItemSelection {
    ProcessAndFinish,
    Cover,
    UpdateSummary,
}

impl Display for BulletListSingleItemSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::Cover => write!(f, "Cover ⼍"),
            Self::UpdateSummary => write!(f, "Update Summary"),
        }
    }
}

fn create_list() -> Vec<BulletListSingleItemSelection> {
    vec![
        BulletListSingleItemSelection::ProcessAndFinish,
        BulletListSingleItemSelection::Cover,
        BulletListSingleItemSelection::UpdateSummary,
    ]
}

#[async_recursion]
pub async fn present_bullet_list_item_selected(
    menu_for: &ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(
                menu_for.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            let r = cover_bullet_item(menu_for, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(InquireError::OperationCanceled) => {
                    present_bullet_list_item_selected(menu_for, send_to_data_storage_layer).await
                }
                Err(err) => todo!("{}", err),
            }
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(
                menu_for.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => (), //Nothing to do we just want to return to the bullet list
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn process_and_finish_bullet_item(
    item: SurrealItem, //TODO: Switch this over to Item<'_>
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    if !user_processed_text.is_empty() {
        send_to_data_storage_layer
            .send(DataLayerCommands::AddProcessedText(
                user_processed_text,
                item.clone(),
            ))
            .await
            .unwrap();
    }

    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem(item))
        .await
        .unwrap();
}
