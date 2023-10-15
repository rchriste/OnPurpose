mod cover_bullet_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, to_do::ToDo},
    bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item,
    display_item::DisplayItem,
    surrealdb_layer::DataLayerCommands,
    update_item_summary, UnexpectedNextMenuAction,
};

enum BulletListSingleItemSelection<'e> {
    ProcessAndFinish,
    Cover,
    UpdateSummary,
    //Take a DisplayItem rather than a reference because it is felt that this type is only created
    //for this scenario rather than kept around.
    ParentItem(DisplayItem<'e>),
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::Cover => write!(f, "Cover ⼍"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::ParentItem(item) => write!(f, "Parent Item: {}", item),
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(parent_items: &[&'e Item<'e>]) -> Vec<Self> {
        let mut list = vec![Self::ProcessAndFinish, Self::Cover, Self::UpdateSummary];

        list.extend(
            parent_items
                .iter()
                .map(|x: &&'e Item<'e>| Self::ParentItem(DisplayItem::new(x))),
        );

        list.push(Self::DebugPrintItem);

        list
    }
}

#[async_recursion]
pub async fn present_bullet_list_item_selected(
    menu_for: &ToDo<'_>,
    parents: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = BulletListSingleItemSelection::create_list(parents);

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            let r = cover_bullet_item(menu_for, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    present_bullet_list_item_selected(menu_for, parents, send_to_data_storage_layer)
                        .await
                }
                Err(UnexpectedNextMenuAction::Close) => todo!(),
            }
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(
                menu_for.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::ParentItem(item)) => {
            present_bullet_list_item_parent_selected(item.into(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::DebugPrintItem) => {
            println!("{:?}", menu_for);
        }
        Err(InquireError::OperationCanceled) => (), //Nothing to do we just want to return to the bullet list
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn process_and_finish_bullet_item(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    //I should probably be processing and finishing all of the children next steps but this requires some thought
    //because sometimes or if there are multiple children next steps that that shouldn't happen rather the user
    //should be prompted to pick which children to also process and finish.
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = item.get_surreal_item();
    if !user_processed_text.is_empty() {
        send_to_data_storage_layer
            .send(DataLayerCommands::AddProcessedText(
                user_processed_text,
                surreal_item.clone(),
            ))
            .await
            .unwrap();
    }

    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem(surreal_item.clone()))
        .await
        .unwrap();
}

async fn present_bullet_list_item_parent_selected(
    selected_item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    match selected_item.item_type {
        crate::base_data::ItemType::ToDo => {
            let to_do = ToDo::new(selected_item);
            let parents = vec![]; //TODO: get parents, probably use send_to_data_storage_layer to get the data needed
            present_bullet_list_item_selected(&to_do, &parents, send_to_data_storage_layer).await
        }
        crate::base_data::ItemType::Hope => todo!(),
        crate::base_data::ItemType::Motivation => todo!(),
    }
}
