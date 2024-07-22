use chrono::Utc;
use tokio::sync::mpsc::Sender;

use crate::{
    new_time_spent::NewTimeSpent,
    node::item_status::ItemStatus,
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_in_the_moment_priority::SurrealAction,
        surreal_item::SurrealUrgency,
    },
};

use super::bullet_list_single_item::give_this_item_a_parent::give_this_item_a_parent;

pub(crate) async fn present_parent_back_to_a_motivation_menu(
    item_status: &ItemStatus<'_>,
    current_urgency: SurrealUrgency,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let start_present_parent_back_to_a_motivation_menu = Utc::now();
    give_this_item_a_parent(item_status.get_item(), send_to_data_storage_layer).await?;

    let new_time_spent = NewTimeSpent {
        working_on: vec![SurrealAction::ParentBackToAMotivation(
            item_status.get_surreal_record_id().clone(),
        )], //TODO: I should also add all the parent items that this is making progress towards the goal, I mean I guess there is no parent because that the goal of the exercise but still for maintainability sake I should add it
        when_started: start_present_parent_back_to_a_motivation_menu,
        when_stopped: Utc::now(),
        dedication: None,
        urgency: Some(current_urgency),
    };

    send_to_data_storage_layer
        .send(DataLayerCommands::RecordTimeSpent(new_time_spent))
        .await
        .unwrap();

    Ok(())
}
