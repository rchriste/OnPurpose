mod cover_with_another_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use duration_str::parse;
use inquire::{Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{ItemVecExtensions, ToDo},
    surrealdb_layer::DataLayerCommands,
};

enum CoverBulletItem {
    ProactiveActionToTake,
    ReactiveBeAvailableToAct,
    WaitingFor,
    TrackingToBeAwareOf,
    CircumstanceThatMustBeTrueToAct,
}

impl Display for CoverBulletItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverBulletItem::ProactiveActionToTake => write!(f, "Proactive action to take"),
            CoverBulletItem::WaitingFor => write!(f, "Waiting For"),
            CoverBulletItem::CircumstanceThatMustBeTrueToAct => {
                write!(f, "Circumstance that must be true to act")
            }
            CoverBulletItem::ReactiveBeAvailableToAct => write!(f, "Reactive be available to act"),
            CoverBulletItem::TrackingToBeAwareOf => write!(f, "Tracking to be aware of"),
        }
    }
}

impl CoverBulletItem {
    fn create_list() -> Vec<CoverBulletItem> {
        vec![
            Self::ProactiveActionToTake,
            Self::ReactiveBeAvailableToAct,
            Self::WaitingFor,
            Self::TrackingToBeAwareOf,
            Self::CircumstanceThatMustBeTrueToAct,
        ]
    }
}

#[async_recursion]
pub async fn cover_bullet_item(
    item_to_cover: ToDo<'async_recursion>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let choices = CoverBulletItem::create_list();
    let selection = Select::new("", choices).prompt().unwrap();
    match selection {
        CoverBulletItem::ProactiveActionToTake => {
            cover_with_another_item::cover_with_proactive_action_to_take(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await
        }
        CoverBulletItem::ReactiveBeAvailableToAct => todo!(),
        CoverBulletItem::WaitingFor => {
            cover_with_waiting_for(item_to_cover, send_to_data_storage_layer).await
        }
        CoverBulletItem::TrackingToBeAwareOf => todo!(),
        CoverBulletItem::CircumstanceThatMustBeTrueToAct => {
            cover_with_circumstance_that_must_be_true_to_act(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await
        }
    }
}

enum CoverWithWaitingFor {
    Question,
    Event,
}

impl Display for CoverWithWaitingFor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverWithWaitingFor::Question => write!(f, "Question"),
            CoverWithWaitingFor::Event => write!(f, "Event"),
        }
    }
}

impl CoverWithWaitingFor {
    fn create_list() -> Vec<CoverWithWaitingFor> {
        vec![Self::Question, Self::Event]
    }
}

pub async fn cover_with_waiting_for<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = CoverWithWaitingFor::create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(CoverWithWaitingFor::Event) => {
            cover_with_waiting_for_event(item_to_cover, send_to_data_storage_layer).await
        }
        Ok(CoverWithWaitingFor::Question) => {
            cover_with_waiting_for_question(item_to_cover, send_to_data_storage_layer).await
        }
        Err(inquire::InquireError::OperationCanceled) => {
            cover_bullet_item(item_to_cover, send_to_data_storage_layer).await
        }
        Err(err) => panic!("{}", err),
    }
}

enum CoverWithQuestionItem {
    NewQuestion,
    ExistingQuestion,
}

impl Display for CoverWithQuestionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewQuestion => write!(f, "New Question"),
            Self::ExistingQuestion => write!(f, "Existing Question"),
        }
    }
}

impl CoverWithQuestionItem {
    fn create_list() -> Vec<CoverWithQuestionItem> {
        vec![Self::NewQuestion, Self::ExistingQuestion]
    }
}

async fn cover_with_waiting_for_question<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = CoverWithQuestionItem::create_list();

    let selected = Select::new("", list).prompt();

    match selected {
        Ok(CoverWithQuestionItem::NewQuestion) => {
            cover_with_new_waiting_for_question(item_to_cover, send_to_data_storage_layer).await
        }
        Ok(CoverWithQuestionItem::ExistingQuestion) => {
            cover_with_existing_waiting_for_question(item_to_cover, send_to_data_storage_layer)
                .await
        }
        Err(inquire::InquireError::OperationCanceled) => {
            cover_bullet_item(item_to_cover, send_to_data_storage_layer).await
        }
        Err(err) => panic!("{}", err),
    }
}

async fn cover_with_new_waiting_for_question<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let question = Text::new("Enter Waiting For Question ⍠").prompt().unwrap();
    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewWaitingForQuestion(
            item_to_cover.into(),
            question,
        ))
        .await
        .unwrap()
}

async fn cover_with_existing_waiting_for_question<'a>(
    _item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let raw_current_items = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let current = raw_current_items.make_items();
    let _to_dos = current.filter_just_to_dos();

    todo!()
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

async fn cover_with_waiting_for_event<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = EventMenuItem::create_list();

    let selection = Select::new("", list).prompt().unwrap();

    match selection {
        EventMenuItem::UntilAnExactDateTime => cover_until_an_exact_date_time().await,
        EventMenuItem::ForAnAmountOfTime => {
            cover_for_an_amount_of_time(Local::now(), item_to_cover, send_to_data_storage_layer)
                .await
        }
    }
}

enum CircumstanceThatMustBeTrueToActMenuItem {
    NotSunday,
}

impl Display for CircumstanceThatMustBeTrueToActMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSunday => write!(f, "Not Sunday"),
        }
    }
}

impl CircumstanceThatMustBeTrueToActMenuItem {
    fn create_list() -> Vec<Self> {
        vec![Self::NotSunday]
    }
}

async fn cover_with_circumstance_that_must_be_true_to_act<'a>(
    item_to_cover: ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = CircumstanceThatMustBeTrueToActMenuItem::create_list();

    let selection = Select::new("", list).prompt().unwrap();

    match selection {
        CircumstanceThatMustBeTrueToActMenuItem::NotSunday => {
            set_circumstance_not_sunday(item_to_cover, send_to_data_storage_layer).await
        }
    }
}

async fn set_circumstance_not_sunday(
    item_to_get_requirement: ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::AddRequirementNotSunday(
            //TODO: This should be renamed to Circumstance
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
