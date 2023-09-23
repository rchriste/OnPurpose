use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{base_data::ToDo, surrealdb_layer::DataLayerCommands};

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

pub(crate) async fn cover_with_another_item(
    item_to_cover: ToDo,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let choices = create_list();

    let selection = Select::new("Select one", choices).prompt().unwrap();
    match selection {
        AnotherItem::NewNextStep => {
            cover_with_new_next_step(item_to_cover, send_to_data_storage_layer).await
        }
        AnotherItem::ExistingNextStep => todo!(),
    }
}

async fn cover_with_new_next_step(
    item_to_cover: ToDo,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let new_next_step_text = Text::new("Enter New Covering Next Step ⍠")
        .prompt()
        .unwrap();

    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewToDo(
            item_to_cover.into(),
            new_next_step_text,
        ))
        .await
        .unwrap();
}
