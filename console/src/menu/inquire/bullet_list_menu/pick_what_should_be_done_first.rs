use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use inquire::Select;
use rand::Rng;
use tokio::sync::mpsc::Sender;

use crate::{
    display::display_action_with_item_status::DisplayActionWithItemStatus,
    menu::inquire::bullet_list_menu::{
        bullet_list_single_item::{
            present_bullet_list_item_selected, present_is_person_or_group_around_menu,
            urgency_plan::present_set_ready_and_urgency_plan_menu,
        },
        parent_back_to_a_motivation::present_parent_back_to_a_motivation_menu,
        pick_item_review_frequency::present_pick_item_review_frequency_menu,
        review_item::present_review_item_menu,
    },
    node::action_with_item_status::ActionWithItemStatus,
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_in_the_moment_priority::SurrealPriorityKind,
    },
    systems::bullet_list::BulletList,
};

use super::bullet_list_single_item::urgency_plan::prompt_for_triggers;

enum HighestOrLowest {
    PickThisTime,
    RecordHighestPriorityUntil,
    RecordLowestPriorityUntil,
    FinishOrRetireItem,
}

impl Display for HighestOrLowest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HighestOrLowest::PickThisTime => write!(f, "Pick This Once"),
            HighestOrLowest::RecordHighestPriorityUntil => {
                write!(f, "Set as highest priority of these items until...")
            }
            HighestOrLowest::RecordLowestPriorityUntil => {
                write!(f, "Set as lowest priority of these items until...")
            }
            HighestOrLowest::FinishOrRetireItem => {
                write!(f, "Finish or retire item")
            }
        }
    }
}

pub(crate) async fn present_pick_what_should_be_done_first_menu<'a>(
    choices: &'a [ActionWithItemStatus<'a>],
    bullet_list: &BulletList,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let display_choices = choices
        .iter()
        .map(DisplayActionWithItemStatus::new)
        .collect::<Vec<_>>();

    let starting_choice = rand::thread_rng().gen_range(0..display_choices.len());
    let choice = Select::new("Pick a priority?", display_choices)
        .with_starting_cursor(starting_choice)
        .prompt()
        .unwrap();

    let highest_or_lowest = Select::new(
        "Highest or lowest priority?",
        vec![
            HighestOrLowest::PickThisTime,
            HighestOrLowest::RecordHighestPriorityUntil,
            HighestOrLowest::RecordLowestPriorityUntil,
            HighestOrLowest::FinishOrRetireItem,
        ],
    )
    .prompt()
    .unwrap();

    let highest_or_lowest = match highest_or_lowest {
        HighestOrLowest::RecordHighestPriorityUntil => SurrealPriorityKind::HighestPriority,
        HighestOrLowest::RecordLowestPriorityUntil => SurrealPriorityKind::LowestPriority,
        HighestOrLowest::FinishOrRetireItem => {
            let now = Utc::now();
            send_to_data_storage_layer
                .send(DataLayerCommands::FinishItem {
                    item: choice.get_surreal_record_id().clone(),
                    when_finished: now.into(),
                })
                .await
                .unwrap();

            return Ok(());
        }
        HighestOrLowest::PickThisTime => {
            let item_action = choice.get_action();
            match item_action {
                ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) => {
                    return Box::pin(present_pick_what_should_be_done_first_menu(
                        choices,
                        bullet_list,
                        send_to_data_storage_layer,
                    ))
                    .await;
                }
                ActionWithItemStatus::PickItemReviewFrequency(item_status) => {
                    return present_pick_item_review_frequency_menu(
                        item_status,
                        item_action.get_urgency_now(),
                        send_to_data_storage_layer,
                    )
                    .await;
                }
                ActionWithItemStatus::ReviewItem(item_status) => {
                    return present_review_item_menu(
                        item_status,
                        item_action.get_urgency_now(),
                        bullet_list.get_all_items_status(),
                        send_to_data_storage_layer,
                    )
                    .await;
                }
                ActionWithItemStatus::MakeProgress(item_status) => {
                    if item_status.is_person_or_group() {
                        return present_is_person_or_group_around_menu(
                            item_status.get_item_node(),
                            send_to_data_storage_layer,
                        )
                        .await;
                    } else {
                        return Box::pin(present_bullet_list_item_selected(
                            item_status,
                            chrono::Utc::now(),
                            bullet_list,
                            send_to_data_storage_layer,
                        ))
                        .await;
                    }
                }
                ActionWithItemStatus::SetReadyAndUrgency(item_status) => {
                    return present_set_ready_and_urgency_plan_menu(
                        item_status,
                        Some(item_action.get_urgency_now()),
                        send_to_data_storage_layer,
                    )
                    .await;
                }
                ActionWithItemStatus::ParentBackToAMotivation(item_status) => {
                    return present_parent_back_to_a_motivation_menu(
                        item_status,
                        item_action.get_urgency_now(),
                        send_to_data_storage_layer,
                    )
                    .await;
                }
            }
        }
    };

    println!("How long should this be in effect?");
    let now = Utc::now();
    let in_effect_until = prompt_for_triggers(&now, send_to_data_storage_layer).await;

    send_to_data_storage_layer
        .send(DataLayerCommands::DeclareInTheMomentPriority {
            choice: choice.clone_to_surreal_action(),
            kind: highest_or_lowest,
            not_chosen: choices
                .iter()
                .filter(|x| x != &choice.get_action())
                .map(|x| x.clone_to_surreal_action())
                .collect(),
            in_effect_until,
        })
        .await
        .unwrap();

    Ok(())
}
