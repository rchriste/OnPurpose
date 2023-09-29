mod cover_with_another_item;

use std::fmt::Display;

use chrono::{DateTime, Local};
use duration_str::parse;
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
        .send(DataLayerCommands::CoverItemWithANewQuestion(
            item_to_cover.into(),
            question,
        ))
        .await
        .unwrap()
}

enum EventMenuItem {
    UntilAnExactDateTime,
    ForAnAmountOfTime,
}

impl Display for EventMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UntilAnExactDateTime => write!(f, "Until an exact date & time"),
            Self::ForAnAmountOfTime => write!(f, "For an amount of time"),
        }
    }
}

impl EventMenuItem {
    fn create_list() -> Vec<EventMenuItem> {
        vec![Self::UntilAnExactDateTime, Self::ForAnAmountOfTime]
    }
}

async fn cover_with_event<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = EventMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        EventMenuItem::UntilAnExactDateTime => cover_until_an_exact_date_time().await,
        EventMenuItem::ForAnAmountOfTime => {
            cover_for_an_amount_of_time(Local::now(), item_to_cover, send_to_data_storage_layer)
                .await
        }
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
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = RequirementMenuItem::create_list();

    let selection = Select::new("Select one", list).prompt().unwrap();

    match selection {
        RequirementMenuItem::NotSunday => {
            set_requirement_not_sunday(item_to_cover, send_to_data_storage_layer).await
        }
    }
}

async fn set_requirement_not_sunday(
    item_to_get_requirement: ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::AddRequirementNotSunday(
            item_to_get_requirement.into(),
        ))
        .await
        .unwrap()
}

async fn cover_until_an_exact_date_time() {
    todo!()
}

async fn cover_for_an_amount_of_time<'a>(
    now: DateTime<Local>,
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let wait_string = Text::new("Cover for how long?").prompt().unwrap();
    let wait_duration = parse(&wait_string).unwrap();
    let wait_until = now + wait_duration;
    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
            item_to_cover.get_surreal_item().clone(),
            wait_until.into(),
        ))
        .await
        .unwrap();
}
