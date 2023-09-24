mod cover_with_another_item;

use std::fmt::Display;

use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{base_data::ToDo, surrealdb_layer::DataLayerCommands};

enum CoverBulletItem {
    AnotherItem,
    Question,
    Event,
    Requirement,
}

impl Display for CoverBulletItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverBulletItem::AnotherItem => write!(f, "Another Item"),
            CoverBulletItem::Event => write!(f, "Event"),
            CoverBulletItem::Question => write!(f, "Question"),
            CoverBulletItem::Requirement => write!(f, "Requirement"),
        }
    }
}

impl CoverBulletItem {
    fn create_list() -> Vec<CoverBulletItem> {
        vec![
            CoverBulletItem::AnotherItem,
            CoverBulletItem::Question,
            CoverBulletItem::Event,
            CoverBulletItem::Requirement,
        ]
    }
}

pub async fn cover_bullet_item<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let choices = CoverBulletItem::create_list();
    let selection = Select::new("Select one", choices).prompt().unwrap();
    match selection {
        CoverBulletItem::AnotherItem => {
            cover_with_another_item::cover_with_another_item(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await
        }
        CoverBulletItem::Question => {
            cover_with_question(item_to_cover, send_to_data_storage_layer).await
        }
        CoverBulletItem::Event => cover_with_event(item_to_cover, send_to_data_storage_layer).await,
        CoverBulletItem::Requirement => {
            cover_with_requirement(item_to_cover, send_to_data_storage_layer).await
        }
    }
}

async fn cover_with_question<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let question = Text::new("Enter Question ⍠").prompt().unwrap();
    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithAQuestion(
            item_to_cover.into(),
            question,
        ))
        .await
        .unwrap()
}

enum EventMenuItem {
    UntilAnExactDateTime,
}

impl Display for EventMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventMenuItem::UntilAnExactDateTime => write!(f, "Until an exact date & time"),
        }
    }
}

impl EventMenuItem {
    fn create_list() -> Vec<EventMenuItem> {
        vec![Self::UntilAnExactDateTime]
    }
}

async fn cover_with_event<'a>(
    _item_to_cover: ToDo<'a>,
    _send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = EventMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        EventMenuItem::UntilAnExactDateTime => todo!(),
    }
}

enum RequirementMenuItem {
    NotSunday,
}

impl Display for RequirementMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequirementMenuItem::NotSunday => write!(f, "Not Sunday"),
        }
    }
}

impl RequirementMenuItem {
    fn create_list() -> Vec<RequirementMenuItem> {
        vec![RequirementMenuItem::NotSunday]
    }
}

async fn cover_with_requirement<'a>(
    _item_to_cover: ToDo<'a>,
    _send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = RequirementMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        RequirementMenuItem::NotSunday => todo!(),
    }
}
