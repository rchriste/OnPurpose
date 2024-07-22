use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use inquire::Select;
use rand::Rng;
use tokio::sync::mpsc::Sender;

use crate::{
    display::display_item_action::DisplayItemAction,
    node::item_action::ActionWithItemStatus,
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_in_the_moment_priority::SurrealPriorityKind,
    },
};

use super::bullet_list_single_item::urgency_plan::prompt_for_triggers;

enum HighestOrLowest {
    Highest,
    Lowest,
}

impl Display for HighestOrLowest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HighestOrLowest::Highest => write!(f, "Highest Priority"),
            HighestOrLowest::Lowest => write!(f, "Lowest Priority"),
        }
    }
}

pub(crate) async fn present_pick_what_should_be_done_first_menu<'a>(
    choices: &'a [ActionWithItemStatus<'a>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let display_choices = choices
        .iter()
        .map(DisplayItemAction::new)
        .collect::<Vec<_>>();

    let starting_choice = rand::thread_rng().gen_range(0..display_choices.len());
    println!("starting choice is {}", starting_choice);
    let choice = Select::new("Pick a priority?", display_choices)
        .with_starting_cursor(starting_choice)
        .prompt()
        .unwrap();

    let highest_or_lowest = Select::new(
        "Highest or lowest priority?",
        vec![HighestOrLowest::Highest, HighestOrLowest::Lowest],
    )
    .prompt()
    .unwrap();

    let highest_or_lowest = match highest_or_lowest {
        HighestOrLowest::Highest => SurrealPriorityKind::HighestPriority,
        HighestOrLowest::Lowest => SurrealPriorityKind::LowestPriority,
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
