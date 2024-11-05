pub(crate) mod classify_item;
pub(crate) mod do_now_list_single_item;
pub(crate) mod parent_back_to_a_motivation;
pub(crate) mod pick_item_review_frequency;
pub(crate) mod pick_what_should_be_done_first;
pub(crate) mod review_item;
pub(crate) mod search;

use std::{fmt::Display, iter::once};

use better_term::Style;
use chrono::{DateTime, Local, Utc};
use classify_item::present_item_needs_a_classification_menu;
use do_now_list_single_item::{urgency_plan::present_set_ready_and_urgency_plan_menu, LogTime};
use inquire::{InquireError, Select};
use itertools::chain;
use parent_back_to_a_motivation::present_parent_back_to_a_motivation_menu;
use pick_item_review_frequency::present_pick_item_review_frequency_menu;
use pick_what_should_be_done_first::present_pick_what_should_be_done_first_menu;
use review_item::present_review_item_menu;
use search::present_search_menu;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_tables::SurrealTables,
    },
    display::{
        display_action_with_item_status::DisplayActionWithItemStatus, display_item::DisplayItem,
        display_scheduled_item::DisplayScheduledItem,
    },
    menu::inquire::back_menu::present_back_menu,
    node::action_with_item_status::ActionWithItemStatus,
    systems::do_now_list::DoNowList,
};

use self::do_now_list_single_item::{
    present_do_now_list_item_selected, present_is_person_or_group_around_menu,
};

use super::back_menu::capture;

pub(crate) enum InquireDoNowListItem<'e> {
    CaptureNewItem,
    Search,
    DoNowListSingleItem(&'e ActionWithItemStatus<'e>),
    RefreshList(DateTime<Local>),
    BackMenu,
    Help,
}

impl Display for InquireDoNowListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureNewItem => write!(f, "🗬   Capture New Item"),
            Self::Search => write!(f, "🔍  Search"),
            Self::DoNowListSingleItem(item) => {
                let display = DisplayActionWithItemStatus::new(item);
                write!(f, "{}", display)
            }
            Self::RefreshList(bullet_list_created) => write!(
                f,
                "🔄  Reload List ({})",
                bullet_list_created.format("%I:%M%p")
            ),
            Self::BackMenu => write!(f, "🏠  Back Menu"),
            Self::Help => write!(f, "❓  Help"),
        }
    }
}

impl<'a> InquireDoNowListItem<'a> {
    pub(crate) fn create_list(
        item_action: &'a [ActionWithItemStatus<'a>],
        do_now_list_created: DateTime<Utc>,
    ) -> Vec<InquireDoNowListItem<'a>> {
        chain!(
            once(InquireDoNowListItem::RefreshList(
                do_now_list_created.into()
            )),
            once(InquireDoNowListItem::Search),
            once(InquireDoNowListItem::CaptureNewItem),
            item_action
                .iter()
                .map(InquireDoNowListItem::DoNowListSingleItem),
            once(InquireDoNowListItem::BackMenu),
            once(InquireDoNowListItem::Help)
        )
        .collect()
    }
}

pub(crate) async fn present_normal_do_now_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let elapsed = Local::now() - before_db_query;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to get data from database. Time taken: {}", elapsed);
    }
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let base_data_checkpoint = Utc::now();
    let calculated_data = CalculatedData::new_from_base_data(base_data);
    let calculated_data_checkpoint = Utc::now();
    let do_now_list = DoNowList::new_do_now_list(calculated_data, &now);
    let finish_checkpoint = Utc::now();
    let elapsed = finish_checkpoint - now;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to create do now list. Time taken: {}", elapsed);
        println!(
            "Base data took: {}, calculated data took: {}, do now list took: {}",
            base_data_checkpoint - now,
            calculated_data_checkpoint - base_data_checkpoint,
            finish_checkpoint - calculated_data_checkpoint
        );
    }
    present_upcoming(&do_now_list);
    present_do_now_list_menu(&do_now_list, now, send_to_data_storage_layer).await
}

pub(crate) fn present_upcoming(do_now_list: &DoNowList) {
    let upcoming = do_now_list.get_upcoming();
    if !upcoming.is_empty() {
        println!("Upcoming:");
        for scheduled_item in upcoming
            .get_ordered_scheduled_items()
            .as_ref()
            .expect("upcoming is not empty")
            .iter()
            .rev()
        {
            let display_scheduled_item = DisplayScheduledItem::new(scheduled_item);
            println!("{}", display_scheduled_item);
        }
    } else if upcoming.has_conflicts() {
        let bold_text = Style::new().bold();
        let not_bold_text = Style::new();
        println!("{}Scheduled items don't fit. At least one of the following items need to be adjusted:{}", bold_text, not_bold_text);
        for conflict in upcoming.get_conflicts() {
            println!("{}", DisplayItem::new(conflict));
        }
        println!();
    }
}

pub(crate) async fn present_do_now_list_menu(
    do_now_list: &DoNowList,
    do_now_list_created: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let ordered_do_now_list = do_now_list.get_ordered_do_now_list();

    let inquire_do_now_list =
        InquireDoNowListItem::create_list(ordered_do_now_list, do_now_list_created);

    let starting_cursor = if ordered_do_now_list.is_empty() { 4 } else { 3 };
    let selected = Select::new(
        "Select from this \"Do Now\" list (default choice is recommended)|",
        inquire_do_now_list,
    )
    .with_starting_cursor(starting_cursor)
    .with_page_size(10)
    .prompt();

    match selected {
        Ok(InquireDoNowListItem::Help) => present_do_now_help(),
        Ok(InquireDoNowListItem::CaptureNewItem) => capture(send_to_data_storage_layer).await,
        Ok(InquireDoNowListItem::Search) => {
            present_search_menu(do_now_list, send_to_data_storage_layer).await
        }
        Ok(InquireDoNowListItem::DoNowListSingleItem(selected)) => match selected {
            ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) => {
                present_pick_what_should_be_done_first_menu(
                    choices,
                    do_now_list,
                    send_to_data_storage_layer,
                )
                .await
            }
            ActionWithItemStatus::PickItemReviewFrequency(item_status) => {
                present_pick_item_review_frequency_menu(
                    item_status,
                    selected.get_urgency_now(),
                    send_to_data_storage_layer,
                )
                .await
            }
            ActionWithItemStatus::ItemNeedsAClassification(item_status) => {
                present_item_needs_a_classification_menu(
                    item_status,
                    selected.get_urgency_now(),
                    send_to_data_storage_layer,
                )
                .await
            }
            ActionWithItemStatus::ReviewItem(item_status) => {
                present_review_item_menu(
                    item_status,
                    selected.get_urgency_now(),
                    do_now_list.get_all_items_status(),
                    LogTime::SeparateTaskLogTheTime,
                    send_to_data_storage_layer,
                )
                .await
            }
            ActionWithItemStatus::MakeProgress(item_status) => {
                if item_status.is_person_or_group() {
                    present_is_person_or_group_around_menu(
                        item_status.get_item_node(),
                        send_to_data_storage_layer,
                    )
                    .await
                } else {
                    Box::pin(present_do_now_list_item_selected(
                        item_status,
                        Utc::now(),
                        do_now_list,
                        send_to_data_storage_layer,
                    ))
                    .await
                }
            }
            ActionWithItemStatus::SetReadyAndUrgency(item_status) => {
                present_set_ready_and_urgency_plan_menu(
                    item_status,
                    Some(selected.get_urgency_now()),
                    LogTime::SeparateTaskLogTheTime,
                    send_to_data_storage_layer,
                )
                .await
            }
            ActionWithItemStatus::ParentBackToAMotivation(item_status) => {
                present_parent_back_to_a_motivation_menu(item_status, send_to_data_storage_layer)
                    .await
            }
        },
        Ok(InquireDoNowListItem::RefreshList(..)) | Err(InquireError::OperationCanceled) => {
            println!("Press Ctrl+C to exit");
            Box::pin(present_normal_do_now_list_menu(send_to_data_storage_layer)).await
        }
        Ok(InquireDoNowListItem::BackMenu) => {
            Box::pin(present_back_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

enum DoNowHelpChoices {
    GettingStarted,
    HowWorkIsScheduled,
    Workarounds,
    ReturnToDoNowList,
}

impl Display for DoNowHelpChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoNowHelpChoices::GettingStarted => write!(f, "How to Get Started"),
            DoNowHelpChoices::HowWorkIsScheduled => write!(f, "How Work is Scheduled"),
            DoNowHelpChoices::Workarounds => {
                write!(f, "Workarounds for features not yet implemented")
            }
            DoNowHelpChoices::ReturnToDoNowList => write!(f, "🔙 Return to Do Now List"),
        }
    }
}

pub(crate) fn present_do_now_help() -> Result<(), ()> {
    let choices = vec![
        DoNowHelpChoices::GettingStarted,
        DoNowHelpChoices::HowWorkIsScheduled,
        DoNowHelpChoices::Workarounds,
        DoNowHelpChoices::ReturnToDoNowList,
    ];
    let selected = Select::new("Select from the below list|", choices).prompt();

    match selected {
        Ok(DoNowHelpChoices::GettingStarted) => {
            present_do_now_help_getting_started()?;
            present_do_now_help()
        }
        Ok(DoNowHelpChoices::HowWorkIsScheduled) => {
            present_do_now_how_work_is_scheduled()?;
            present_do_now_help()
        }
        Ok(DoNowHelpChoices::Workarounds) => {
            present_do_now_help_workarounds()?;
            present_do_now_help()
        }
        Ok(DoNowHelpChoices::ReturnToDoNowList) | Err(InquireError::OperationCanceled) => Ok(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

pub(crate) fn present_do_now_help_getting_started() -> Result<(), ()> {
    println!();
    println!("Getting Started Help Coming Soon!");
    println!();
    Ok(())
}

pub(crate) fn present_do_now_how_work_is_scheduled() -> Result<(), ()> {
    println!();
    println!("How Work is Scheduled Help Coming Soon!");
    println!();
    Ok(())
}

pub(crate) fn present_do_now_help_workarounds() -> Result<(), ()> {
    println!();
    println!("Workarounds Help Coming Soon!");
    println!();
    Ok(())
}
