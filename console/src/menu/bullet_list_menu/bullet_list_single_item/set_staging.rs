use async_recursion::async_recursion;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::item::Item,
    menu::staging_query::{mentally_resident_query, on_deck_query},
    surrealdb_layer::{
        surreal_item::{Responsibility, Staging},
        DataLayerCommands,
    },
};
use inquire::{InquireError, Select};
use std::fmt::Display;

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum StagingMenuSelection {
    NotSet,
    MentallyResident,
    OnDeck,
    Planned,
    ThinkingAbout,
    Released,
    MakeItemReactive,
}

impl Display for StagingMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StagingMenuSelection::NotSet => write!(f, "Not Set"),
            StagingMenuSelection::MentallyResident => write!(f, "Mentally Resident"),
            StagingMenuSelection::OnDeck => write!(f, "On Deck"),
            StagingMenuSelection::Planned => write!(f, "Planned"),
            StagingMenuSelection::ThinkingAbout => write!(f, "Thinking About"),
            StagingMenuSelection::Released => write!(f, "Released"),
            StagingMenuSelection::MakeItemReactive => write!(f, "Make Item Reactive"),
        }
    }
}

impl StagingMenuSelection {
    /// Returns a tuple of the list and the default index or recommended default selection
    pub(crate) fn make_list(default_selection: Option<StagingMenuSelection>) -> (Vec<Self>, usize) {
        let choices = vec![
            StagingMenuSelection::MentallyResident,
            StagingMenuSelection::OnDeck,
            StagingMenuSelection::Planned,
            StagingMenuSelection::ThinkingAbout,
            StagingMenuSelection::Released,
            StagingMenuSelection::NotSet,
            StagingMenuSelection::MakeItemReactive,
        ];
        let default_index = match default_selection {
            Some(default_selection) => choices
                .iter()
                .position(|choice| choice == &default_selection)
                .unwrap(),
            None => 1,
        };

        (choices, default_index)
    }
}

#[async_recursion]
pub(crate) async fn present_set_staging_menu(
    selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    default_selection: Option<StagingMenuSelection>,
) -> Result<(), ()> {
    let staging = loop {
        let (list, starting_cursor) = StagingMenuSelection::make_list(default_selection);

        let selection = Select::new("Select from the below list|", list)
            .with_starting_cursor(starting_cursor)
            .prompt();
        let staging = match selection {
            Ok(StagingMenuSelection::NotSet) => Staging::NotSet,
            Ok(StagingMenuSelection::MentallyResident) => {
                let result = mentally_resident_query().await;
                match result {
                    Ok(mentally_resident) => mentally_resident,
                    Err(InquireError::OperationCanceled) => {
                        return present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        )
                        .await
                    }
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Ok(StagingMenuSelection::OnDeck) => {
                let result = on_deck_query().await;
                match result {
                    Ok(staging) => staging,
                    Err(InquireError::OperationCanceled) => {
                        return present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        )
                        .await
                    }
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Ok(StagingMenuSelection::Planned) => Staging::Planned,
            Ok(StagingMenuSelection::ThinkingAbout) => Staging::ThinkingAbout,
            Ok(StagingMenuSelection::Released) => Staging::Released,
            Ok(StagingMenuSelection::MakeItemReactive) => {
                send_to_data_storage_layer
                    .send(DataLayerCommands::UpdateItemResponsibility(
                        selected.get_surreal_record_id().clone(),
                        Responsibility::ReactiveBeAvailableToAct,
                    ))
                    .await
                    .unwrap();
                return Ok(());
            }
            Err(InquireError::OperationInterrupted) => return Err(()),
            Err(InquireError::OperationCanceled) => {
                // Just continue because we don't know exactly what to go back to
                return Ok(());
            }
            Err(err) => todo!("{:?}", err),
        };
        break staging;
    };

    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateItemStaging(
            selected.get_surreal_record_id().clone(),
            staging,
        ))
        .await
        .unwrap();
    Ok(())
}
