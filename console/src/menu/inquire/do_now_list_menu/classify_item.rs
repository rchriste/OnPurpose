use ahash::HashSet;
use chrono::Utc;
use tokio::sync::mpsc::Sender;

use crate::{
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_in_the_moment_priority::SurrealAction,
        surreal_item::SurrealUrgency,
    },
    new_time_spent::NewTimeSpent,
    node::{
        item_status::ItemStatus,
        why_in_scope_and_action_with_item_status::{ToSurreal, WhyInScope},
    },
};

use super::do_now_list_single_item::declare_item_type;

pub(crate) async fn present_item_needs_a_classification_menu(
    item_status: &ItemStatus<'_>,
    current_urgency: Option<SurrealUrgency>,
    why_in_scope: &HashSet<WhyInScope>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let start_present_item_needs_a_classification_menu = Utc::now();

    declare_item_type(item_status.get_item(), send_to_data_storage_layer).await?;

    let new_time_spent = NewTimeSpent {
        why_in_scope: why_in_scope.to_surreal(),
        working_on: vec![SurrealAction::ItemNeedsAClassification(
            item_status.get_surreal_record_id().clone(),
        )], //TODO: I should also add all the parent items that this is making progress towards the goal, I mean I guess there is no parent because that the goal of the exercise but still for maintainability sake I should add it
        when_started: start_present_item_needs_a_classification_menu,
        when_stopped: Utc::now(),
        dedication: None,
        urgency: current_urgency,
    };

    send_to_data_storage_layer
        .send(DataLayerCommands::RecordTimeSpent(new_time_spent))
        .await
        .unwrap();

    Ok(())
}
