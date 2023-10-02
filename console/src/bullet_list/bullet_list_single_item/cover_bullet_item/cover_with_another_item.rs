use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::ToDo,
    surrealdb_layer::{
        surreal_specific_to_todo::{Order, Responsibility},
        DataLayerCommands,
    },
};

enum AnotherItem {
    //Eventually it would be nice if the new and existing could be combined into one UI control where you just type and
    //it shows items that already exist and you can pick one of them or use this to make a new one but because it is
    //easier to implement start with one for new and one for search.
    NewNextStep,
    ExistingNextStep,
}

impl Display for AnotherItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnotherItem::NewNextStep => write!(f, "New Next Step"),
            AnotherItem::ExistingNextStep => write!(f, "Search Existing Items"),
        }
    }
}

fn create_list() -> Vec<AnotherItem> {
    vec![AnotherItem::NewNextStep, AnotherItem::ExistingNextStep]
}

pub(crate) async fn cover_with_proactive_action_to_take<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let choices = create_list();

    let selection = Select::new("", choices).prompt().unwrap();
    match selection {
        AnotherItem::NewNextStep => {
            cover_with_new_proactive_action_to_take(item_to_cover, send_to_data_storage_layer).await
        }
        AnotherItem::ExistingNextStep => {
            cover_with_existing_next_step(item_to_cover, send_to_data_storage_layer).await
        }
    }
}

async fn cover_with_new_proactive_action_to_take<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let new_next_step_text = Text::new("Enter New Covering Action To Take ⍠")
        .prompt()
        .unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewToDo(
            item_to_cover.into(),
            new_next_step_text,
            Order::NextStep,
            Responsibility::ProactiveActionToTake,
        ))
        .await
        .unwrap();
}

async fn cover_with_existing_next_step(
    _item_to_cover: ToDo<'_>,
    _send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    todo!()
}
