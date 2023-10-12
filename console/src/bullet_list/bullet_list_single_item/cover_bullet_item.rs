mod cover_with_another_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use duration_str::parse;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{ItemVecExtensions, ToDo},
    surrealdb_layer::DataLayerCommands, UnexpectedNextMenuAction,
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
    item_to_cover: &'async_recursion ToDo<'async_recursion>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), UnexpectedNextMenuAction> {
    let choices = CoverBulletItem::create_list();
    let selection = Select::new("", choices).prompt();
    match selection {
        Ok(CoverBulletItem::ProactiveActionToTake) => {
            cover_with_another_item::cover_with_proactive_action_to_take(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await;
            Ok(())
        }
        Ok(CoverBulletItem::ReactiveBeAvailableToAct) => todo!(),
        Ok(CoverBulletItem::WaitingFor) => {
            let r = cover_with_waiting_for(item_to_cover, send_to_data_storage_layer).await;
            match r {
                Ok(()) => Ok(()),
                Err(UnexpectedNextMenuAction::Back) => cover_bullet_item(item_to_cover, send_to_data_storage_layer).await,
                Err(UnexpectedNextMenuAction::Close) => Err(UnexpectedNextMenuAction::Close),
            }
        }
        Ok(CoverBulletItem::TrackingToBeAwareOf) => todo!(),
        Ok(CoverBulletItem::CircumstanceThatMustBeTrueToAct) => {
            cover_with_circumstance_that_must_be_true_to_act(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await;
            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            Err(UnexpectedNextMenuAction::Back)
        }
        Err(InquireError::OperationInterrupted) => Err(UnexpectedNextMenuAction::Close),
        Err(err) => todo!("{}", err),
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

#[async_recursion]
pub async fn cover_with_waiting_for<'a>(
    item_to_cover: &'a ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), UnexpectedNextMenuAction> {
    let list = CoverWithWaitingFor::create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(CoverWithWaitingFor::Event) => {
            cover_with_waiting_for_event(item_to_cover, send_to_data_storage_layer).await;
            Ok(())
        }
        Ok(CoverWithWaitingFor::Question) => {
            cover_with_waiting_for_question(item_to_cover, send_to_data_storage_layer).await;
            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            Err(UnexpectedNextMenuAction::Back)
        }
        Err(err) => todo!("{}", err),
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

#[async_recursion]
async fn cover_with_waiting_for_question<'a>(
    item_to_cover: &'a ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = CoverWithQuestionItem::create_list();

    let selected = Select::new("", list).prompt();

    match selected {
        Ok(CoverWithQuestionItem::NewQuestion) => {
            cover_with_new_waiting_for_question(item_to_cover, send_to_data_storage_layer).await;
        }
        Ok(CoverWithQuestionItem::ExistingQuestion) => {
            cover_with_existing_waiting_for_question(item_to_cover, send_to_data_storage_layer)
                .await;
        }
        Err(InquireError::OperationCanceled) => {
            let r = cover_bullet_item(item_to_cover, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(_) => (), //ignore cancel because paying attention causes bugs due to not having a proper back chain
            }
        }
        Err(err) => todo!("{}", err),
    }
}

async fn cover_with_new_waiting_for_question<'a>(
    item_to_cover: &'a ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let question = Text::new("Enter Waiting For Question ⍠").prompt().unwrap();
    send_to_data_storage_layer
        .send(DataLayerCommands::CoverItemWithANewWaitingForQuestion(
            item_to_cover.get_surreal_item().clone(),
            question,
        ))
        .await
        .unwrap()
}

async fn cover_with_existing_waiting_for_question(
    _item_to_cover: &ToDo<'_>,
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

#[async_recursion]
async fn cover_with_waiting_for_event<'a>(
    item_to_cover: &'a ToDo<'a>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = EventMenuItem::create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(EventMenuItem::UntilAnExactDateTime) => {
            cover_until_an_exact_date_time().await;
        }
        Ok(EventMenuItem::ForAnAmountOfTime) => {
            cover_for_an_amount_of_time(Local::now(), item_to_cover, send_to_data_storage_layer)
                .await;
        }
        Err(InquireError::OperationCanceled) => {
            let r = cover_with_waiting_for(item_to_cover, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    cover_with_waiting_for_event(item_to_cover, send_to_data_storage_layer).await;
                },
                Err(UnexpectedNextMenuAction::Close) => todo!("Change return type of this function so this can be returned"),
            }
        }
        Err(err) => todo!("{}", err),
    }
}

enum CircumstanceThatMustBeTrueToActMenuItem {
    NotSunday,
    FocusTime,
}

impl Display for CircumstanceThatMustBeTrueToActMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSunday => write!(f, "Not On Sunday"),
            Self::FocusTime => write!(f, "During Focus Time"),
        }
    }
}

impl CircumstanceThatMustBeTrueToActMenuItem {
    fn create_list() -> Vec<Self> {
        vec![Self::NotSunday, Self::FocusTime]
    }
}

#[async_recursion]
async fn cover_with_circumstance_that_must_be_true_to_act(
    item_to_cover: &ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = CircumstanceThatMustBeTrueToActMenuItem::create_list();

    let selection = Select::new("", list).prompt();

    match selection {
        Ok(CircumstanceThatMustBeTrueToActMenuItem::NotSunday) => {
            set_circumstance_not_sunday(item_to_cover, send_to_data_storage_layer).await;
        }
        Ok(CircumstanceThatMustBeTrueToActMenuItem::FocusTime) => {
            set_circumstance_during_focus_time(item_to_cover, send_to_data_storage_layer).await;
        }
        Err(InquireError::OperationCanceled) => {
            let r = cover_bullet_item(item_to_cover, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    cover_with_circumstance_that_must_be_true_to_act(
                        item_to_cover,
                        send_to_data_storage_layer,
                    )
                    .await
                }
                Err(UnexpectedNextMenuAction::Close) => todo!("Change the return type of this function and return this"),
            }
        }
        Err(err) => todo!("{}", err),
    }
}

async fn set_circumstance_not_sunday(
    item_to_get_circumstance: &ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::AddCircumstanceNotSunday(
            item_to_get_circumstance.get_surreal_item().clone(),
        ))
        .await
        .unwrap()
}

async fn set_circumstance_during_focus_time(
    item_to_get_circumstance: &ToDo<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::AddCircumstanceDuringFocusTime(
            item_to_get_circumstance.get_surreal_item().clone(),
        ))
        .await
        .unwrap()
}

async fn cover_until_an_exact_date_time() {
    todo!()
}

#[async_recursion]
async fn cover_for_an_amount_of_time(
    now: DateTime<Local>,
    item_to_cover: &ToDo<'async_recursion>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let result = Text::new("Cover for how long?").prompt();
    match result {
        Ok(wait_string) => {
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
        Err(InquireError::OperationCanceled) => {
            cover_with_waiting_for_event(item_to_cover, send_to_data_storage_layer).await
        }
        Err(err) => todo!("{}", err),
    }
}
