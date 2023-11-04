use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use duration_str::parse;
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        BaseData,
    },
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
    UnexpectedNextMenuAction,
};

use super::cover_with_item;

enum CoverBulletItem {
    ItemOrNextStep,
    WaitForSomethingOrScheduled,
    GroupOrDoWith,
    Circumstance,
    Mood,
}

impl Display for CoverBulletItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverBulletItem::ItemOrNextStep => write!(f, "Item or Next Step"),
            CoverBulletItem::WaitForSomethingOrScheduled => write!(
                f,
                "Wait for something to happen or wait until a scheduled time"
            ),
            CoverBulletItem::Circumstance => write!(f, "Circumstance"),
            CoverBulletItem::Mood => write!(f, "Mood"),
            CoverBulletItem::GroupOrDoWith => write!(f, "Group or do with"),
        }
    }
}

impl CoverBulletItem {
    fn create_list() -> Vec<Self> {
        vec![
            Self::ItemOrNextStep,
            Self::WaitForSomethingOrScheduled,
            Self::GroupOrDoWith,
            Self::Circumstance,
            Self::Mood,
        ]
    }
}

#[async_recursion]
pub(crate) async fn cover_bullet_item(
    item_to_cover: &'async_recursion Item<'async_recursion>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), UnexpectedNextMenuAction> {
    let choices = CoverBulletItem::create_list();
    let selection = Select::new("", choices).prompt();
    match selection {
        Ok(CoverBulletItem::ItemOrNextStep) => {
            cover_with_item(item_to_cover, send_to_data_storage_layer).await;
            Ok(())
        }
        Ok(CoverBulletItem::WaitForSomethingOrScheduled) => {
            let r = cover_with_waiting_for(item_to_cover, send_to_data_storage_layer).await;
            match r {
                Ok(()) => Ok(()),
                Err(UnexpectedNextMenuAction::Back) => {
                    cover_bullet_item(item_to_cover, send_to_data_storage_layer).await
                }
                Err(UnexpectedNextMenuAction::Close) => Err(UnexpectedNextMenuAction::Close),
            }
        }
        Ok(CoverBulletItem::Circumstance) => {
            cover_with_circumstance_that_must_be_true_to_act(
                item_to_cover,
                send_to_data_storage_layer,
            )
            .await;
            Ok(())
        }
        Ok(CoverBulletItem::Mood) => {
            todo!("Setting this as to do during focus time should be part of it")
        }
        Ok(CoverBulletItem::GroupOrDoWith) => {
            todo!("Allow the user to select what group of items this should be a part of")
        }
        Err(InquireError::OperationCanceled) => Err(UnexpectedNextMenuAction::Back),
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
pub(crate) async fn cover_with_waiting_for<'a>(
    item_to_cover: &'a Item<'a>,
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
        Err(InquireError::OperationCanceled) => Err(UnexpectedNextMenuAction::Back),
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
    item_to_cover: &'a Item<'a>,
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
            let _ = cover_bullet_item(item_to_cover, send_to_data_storage_layer).await;
            //discard the result because paying attention causes bugs due to not having a proper back chain
        }
        Err(err) => todo!("{}", err),
    }
}

async fn cover_with_new_waiting_for_question<'a>(
    item_to_cover: &'a Item<'a>,
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

enum CoverExistingItem<'e> {
    ExistingToDo(&'e Item<'e>),
}

impl Display for CoverExistingItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverExistingItem::ExistingToDo(to_do) => write!(f, "{}", to_do.summary),
        }
    }
}

impl<'e> CoverExistingItem<'e> {
    fn create_list(to_dos: impl Iterator<Item = &'e Item<'e>>) -> Vec<Self> {
        to_dos
            .into_iter()
            .map(CoverExistingItem::ExistingToDo)
            .collect()
    }
}

#[async_recursion]
async fn cover_with_existing_waiting_for_question(
    item_to_cover: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let raw_current_items = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let base_data = BaseData::new_from_surreal_tables(raw_current_items);
    let current = base_data.get_items();

    let list = CoverExistingItem::create_list(current.filter_just_to_dos());

    let selection = Select::new("Start typing to search the list", list).prompt();
    match selection {
        Ok(CoverExistingItem::ExistingToDo(selected_to_do)) => send_to_data_storage_layer
            .send(DataLayerCommands::CoverItemWithAnExistingItem {
                item_to_be_covered: item_to_cover.get_surreal_item().clone(),
                item_that_should_do_the_covering: selected_to_do.get_surreal_item().clone(),
            })
            .await
            .unwrap(),
        Err(InquireError::OperationCanceled) => {
            cover_with_waiting_for_question(item_to_cover, send_to_data_storage_layer).await
        }
        Err(err) => todo!("{}", err),
    }
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
        vec![Self::ForAnAmountOfTime, Self::UntilAnExactDateTime]
    }
}

#[async_recursion]
async fn cover_with_waiting_for_event<'a>(
    item_to_cover: &'a Item<'a>,
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
                }
                Err(UnexpectedNextMenuAction::Close) => {
                    todo!("Change return type of this function so this can be returned")
                }
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
    item_to_cover: &Item<'_>,
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
                Err(UnexpectedNextMenuAction::Close) => {
                    todo!("Change the return type of this function and return this")
                }
            }
        }
        Err(err) => todo!("{}", err),
    }
}

async fn set_circumstance_not_sunday(
    item_to_get_circumstance: &Item<'_>,
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
    item_to_get_circumstance: &Item<'_>,
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
    item_to_cover: &Item<'async_recursion>,
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
