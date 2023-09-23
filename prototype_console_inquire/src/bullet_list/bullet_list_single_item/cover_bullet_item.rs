mod cover_with_another_item;

use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{base_data::NextStepItem, surrealdb_layer::DataLayerCommands};


enum CoverBulletItem {
    AnotherItem,
    Question,
    Event
}

impl Display for CoverBulletItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverBulletItem::AnotherItem => write!(f, "Another Item"),
            CoverBulletItem::Event => write!(f, "Event"),
            CoverBulletItem::Question => write!(f, "Question"),
        }
    }
}

fn create_list() -> Vec<CoverBulletItem> {
    vec![
        CoverBulletItem::AnotherItem,
        CoverBulletItem::Question,
        CoverBulletItem::Event
    ]
}

pub async fn cover_bullet_item(item_to_cover: NextStepItem, send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let choices = create_list();
    let selection = Select::new("Select one", choices).prompt().unwrap();
    match selection {
        CoverBulletItem::AnotherItem => cover_with_another_item::cover_with_another_item(item_to_cover, send_to_data_storage_layer).await,
        CoverBulletItem::Question => cover_with_question(item_to_cover, send_to_data_storage_layer).await,
        CoverBulletItem::Event => todo!(),
    }
}

async fn cover_with_question(item_to_cover: NextStepItem, send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let question = Text::new("Enter Question").prompt().unwrap();
    send_to_data_storage_layer.send(DataLayerCommands::CoverItemWithAQuestion(item_to_cover.into(), question)).await.unwrap()
}

