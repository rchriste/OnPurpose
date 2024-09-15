pub(crate) mod bullet_list_single_item;
pub(crate) mod parent_back_to_a_motivation;
pub(crate) mod pick_item_review_frequency;
pub(crate) mod pick_what_should_be_done_first;
pub(crate) mod review_item;
pub(crate) mod search;

use std::{fmt::Display, iter::once};

use better_term::Style;
use bullet_list_single_item::{urgency_plan::present_set_ready_and_urgency_plan_menu, LogTime};
use chrono::{DateTime, Local, TimeDelta, Utc};
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
    menu::inquire::top_menu::present_top_menu,
    node::action_with_item_status::ActionWithItemStatus,
    systems::bullet_list::BulletList,
};

use self::bullet_list_single_item::{
    present_bullet_list_item_selected, present_is_person_or_group_around_menu,
};

use super::top_menu::capture;

pub(crate) enum InquireBulletListItem<'e> {
    CaptureNewItem,
    Search,
    BulletListSingleItem(&'e ActionWithItemStatus<'e>),
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureNewItem => write!(f, "🗬   Capture New Item  🗭"),
            Self::Search => write!(f, "🔍  Search            🔍"),
            Self::BulletListSingleItem(item) => {
                let display = DisplayActionWithItemStatus::new(item);
                write!(f, "{}", display)
            }
        }
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub(crate) fn create_list(
        item_action: &'a [ActionWithItemStatus<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        chain!(
            once(InquireBulletListItem::CaptureNewItem),
            once(InquireBulletListItem::Search),
            item_action
                .iter()
                .map(InquireBulletListItem::BulletListSingleItem)
        )
        .collect()
    }
}

pub(crate) async fn present_normal_bullet_list_menu(
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
    let bullet_list = BulletList::new_bullet_list(calculated_data, &now);
    let finish_checkpoint = Utc::now();
    let elapsed = finish_checkpoint - now;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to create bullet list. Time taken: {}", elapsed);
        println!(
            "Base data took: {}, calculated data took: {}, bullet list took: {}",
            base_data_checkpoint - now,
            calculated_data_checkpoint - base_data_checkpoint,
            finish_checkpoint - calculated_data_checkpoint
        );
    }
    present_upcoming(&bullet_list);
    present_bullet_list_menu(&bullet_list, now, send_to_data_storage_layer).await
}

pub(crate) fn present_upcoming(bullet_list: &BulletList) {
    let upcoming = bullet_list.get_upcoming();
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

pub(crate) async fn present_bullet_list_menu(
    bullet_list: &BulletList,
    bullet_list_created: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let ordered_bullet_list = bullet_list.get_ordered_bullet_list();

    let inquire_bullet_list = InquireBulletListItem::create_list(ordered_bullet_list);

    if !inquire_bullet_list.is_empty() {
        let starting_cursor = if ordered_bullet_list.is_empty() { 0 } else { 2 };
        let selected = Select::new("Select from the below list|", inquire_bullet_list)
            .with_starting_cursor(starting_cursor)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::CaptureNewItem) => capture(send_to_data_storage_layer).await,
            Ok(InquireBulletListItem::Search) => {
                present_search_menu(bullet_list, send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::BulletListSingleItem(selected)) => match selected {
                ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) => {
                    present_pick_what_should_be_done_first_menu(
                        choices,
                        bullet_list,
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
                ActionWithItemStatus::ReviewItem(item_status) => {
                    present_review_item_menu(
                        item_status,
                        selected.get_urgency_now(),
                        bullet_list.get_all_items_status(),
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
                        Box::pin(present_bullet_list_item_selected(
                            item_status,
                            Utc::now(),
                            bullet_list,
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
                    present_parent_back_to_a_motivation_menu(
                        item_status,
                        selected.get_urgency_now(),
                        send_to_data_storage_layer,
                    )
                    .await
                }
            },
            Err(InquireError::OperationCanceled) => {
                //Pressing Esc is meant to refresh the list unless you press it twice in a row then it will go to the top menu
                if Utc::now() - bullet_list_created > TimeDelta::seconds(5) {
                    println!("Refreshing the list");
                    Box::pin(present_normal_bullet_list_menu(send_to_data_storage_layer)).await
                } else {
                    Box::pin(present_top_menu(send_to_data_storage_layer)).await
                }
            }
            Err(InquireError::OperationInterrupted) => Err(()),
            Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
        }
    } else {
        println!("To Do List is Empty, falling back to main menu");
        Box::pin(present_top_menu(send_to_data_storage_layer)).await
    }
}
