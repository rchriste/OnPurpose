mod cover_bullet_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Local;
use inquire::{Editor, InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::ItemVecExtensions,
    bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item,
    node::create_to_do_nodes,
    surrealdb_layer::{surreal_item::SurrealItem, DataLayerCommands},
    top_menu::present_top_menu,
    update_item_summary,
};

use super::InquireBulletListItem;

#[async_recursion]
pub async fn present_bullet_list_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);
    let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

    let to_dos = &items.filter_just_to_dos();
    let current_date_time = Local::now();
    let next_step_nodes = create_to_do_nodes(
        to_dos,
        &coverings,
        &coverings_until_date_time,
        &current_date_time,
    );

    let inquire_bullet_list = InquireBulletListItem::create_list(&next_step_nodes);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(30)
            .prompt();

        match selected {
            Ok(selected) => {
                present_bullet_list_item_selected(selected, send_to_data_storage_layer).await
            }
            Err(InquireError::OperationCanceled) => {
                present_top_menu(send_to_data_storage_layer).await
            }
            Err(err) => todo!("Unexpected InquireError of {}", err),
        };
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(send_to_data_storage_layer).await
    }
}

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

pub async fn present_bullet_list_item_selected(
    menu_for: InquireBulletListItem<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.into(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            cover_bullet_item(menu_for.into(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(menu_for.into(), send_to_data_storage_layer).await
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
