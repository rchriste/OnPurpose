use tokio::sync::mpsc::Sender;

use crate::{
    menu::on_deck_query::on_deck_query,
    node::item_node::ItemNode,
    surrealdb_layer::{surreal_item::Staging, DataLayerCommands},
};
use inquire::{InquireError, Select};
use std::fmt::Display;

enum StagingMenuSelection {
    NotSet,
    MentallyResident,
    OnDeck,
    Intension,
    Released,
}

impl Display for StagingMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StagingMenuSelection::NotSet => write!(f, "Not Set"),
            StagingMenuSelection::MentallyResident => write!(f, "Mentally Resident"),
            StagingMenuSelection::OnDeck => write!(f, "On Deck"),
            StagingMenuSelection::Intension => write!(f, "Intension"),
            StagingMenuSelection::Released => write!(f, "Released"),
        }
    }
}

impl StagingMenuSelection {
    fn make_list() -> Vec<Self> {
        vec![
            StagingMenuSelection::OnDeck,
            StagingMenuSelection::MentallyResident,
            StagingMenuSelection::Intension,
            StagingMenuSelection::Released,
            StagingMenuSelection::NotSet,
        ]
    }
}

pub(crate) async fn present_set_staging_menu(
    selected: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = StagingMenuSelection::make_list();

    let selection = Select::new("", list).prompt().unwrap();
    let staging = match selection {
        StagingMenuSelection::NotSet => Staging::NotSet,
        StagingMenuSelection::MentallyResident => Staging::MentallyResident,
        StagingMenuSelection::OnDeck => {
            let result = on_deck_query().await;
            match result {
                Ok(staging) => staging,
                Err(InquireError::OperationCanceled) => {
                    todo!("There are multiple places to go back to")
                }
                Err(err) => todo!("{:?}", err),
            }
        }
        StagingMenuSelection::Intension => Staging::Intension,
        StagingMenuSelection::Released => Staging::Released,
    };

    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemStaging(
            selected.get_surreal_item().clone(),
            staging,
        ))
        .await
        .unwrap();
}
