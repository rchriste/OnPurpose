use chrono::Utc;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    display::display_staging::DisplayStaging,
    menu::inquire::{
        bullet_list_menu::bullet_list_single_item::state_a_smaller_next_step::{
            select_an_item, SelectAnItemSortingOrder,
        },
        staging_query::{mentally_resident_query, on_deck_query},
    },
    surrealdb_layer::{
        surreal_item::{InRelationToRatioType, Responsibility, SurrealStaging},
        surreal_tables::SurrealTables,
        DataLayerCommands,
    },
};
use inquire::{InquireError, Select, Text};
use std::fmt::Display;

#[derive(PartialEq, Eq, Copy, Clone)]
pub(crate) enum StagingMenuSelection<'e> {
    KeepAsIs(&'e SurrealStaging),
    InRelationTo,
    NotSet,
    MentallyResident,
    OnDeck,
    Planned,
    ThinkingAbout,
    Released,
    MakeItemReactive,
}

impl Display for StagingMenuSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StagingMenuSelection::NotSet => write!(f, "Not Set"),
            StagingMenuSelection::MentallyResident => write!(f, "🧠 Mentally Resident"),
            StagingMenuSelection::OnDeck => write!(f, "🔜 On Deck"),
            StagingMenuSelection::Planned => write!(f, "📌 Planned"),
            StagingMenuSelection::ThinkingAbout => write!(f, "🤔 Thinking About"),
            StagingMenuSelection::Released => write!(f, "Released"),
            StagingMenuSelection::MakeItemReactive => write!(f, "Make Item Reactive"),
            StagingMenuSelection::KeepAsIs(staging) => {
                let display_staging = DisplayStaging::new(staging);
                write!(f, "Keep current setting of {}", display_staging)
            }
            StagingMenuSelection::InRelationTo => write!(f, "In Relation To Another Item"),
        }
    }
}

impl<'e> StagingMenuSelection<'e> {
    /// Returns a tuple of the list and the default index or recommended default selection
    pub(crate) fn make_list(
        default_selection: Option<StagingMenuSelection>,
        current_setting: Option<&'e SurrealStaging>,
    ) -> (Vec<Self>, usize) {
        let mut choices = vec![
            StagingMenuSelection::MentallyResident,
            StagingMenuSelection::OnDeck,
            StagingMenuSelection::Planned,
            StagingMenuSelection::ThinkingAbout,
            StagingMenuSelection::Released,
            StagingMenuSelection::NotSet,
            StagingMenuSelection::MakeItemReactive,
            StagingMenuSelection::InRelationTo,
        ];
        if let Some(current_setting) = current_setting {
            choices.push(StagingMenuSelection::KeepAsIs(current_setting));
        }
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

enum InRelationToType {
    AmountOfTimeSpent,
    IterationCount,
}

impl Display for InRelationToType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InRelationToType::AmountOfTimeSpent => write!(f, "Amount of time spent"),
            InRelationToType::IterationCount => write!(f, "Iteration count"),
        }
    }
}

pub(crate) async fn present_set_staging_menu(
    selected: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
    default_selection: Option<StagingMenuSelection<'_>>,
) -> Result<(), ()> {
    let staging = loop {
        let (list, starting_cursor) =
            StagingMenuSelection::make_list(default_selection, Some(selected.get_staging()));

        let selection = Select::new("Select from the below list|", list)
            .with_starting_cursor(starting_cursor)
            .with_page_size(9)
            .prompt();
        let staging = match selection {
            Ok(StagingMenuSelection::InRelationTo) => {
                let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                    .await
                    .unwrap();
                let now = Utc::now();
                let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
                let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
                let selected = select_an_item(
                    vec![selected],
                    SelectAnItemSortingOrder::MotivationsFirst,
                    &calculated_data,
                )
                .await;
                match selected {
                    Ok(Some(relation_to_this)) => {
                        let list = vec![
                            InRelationToType::AmountOfTimeSpent,
                            InRelationToType::IterationCount,
                        ];
                        let relation_type =
                            Select::new("In relation to the selected item according to|", list)
                                .prompt();
                        match relation_type {
                            Ok(InRelationToType::AmountOfTimeSpent) => {
                                let ratio = Text::new("Ratio of time spent on this item to time spent on the selected item ⍠").prompt().unwrap();
                                let ratio = ratio.parse::<f32>().unwrap();
                                SurrealStaging::InRelationTo {
                                    start: now.into(),
                                    other_item: relation_to_this.get_surreal_record_id().clone(),
                                    ratio: InRelationToRatioType::AmountOfTimeSpent {
                                        multiplier: ratio.into(),
                                    },
                                }
                            }
                            Ok(InRelationToType::IterationCount) => todo!(),
                            Err(_) => todo!(),
                        }
                    }
                    Ok(None) => {
                        println!("Canceled");
                        todo!()
                    }
                    Err(()) => {
                        return Err(());
                    }
                }
            }
            Ok(StagingMenuSelection::NotSet) => SurrealStaging::NotSet,
            Ok(StagingMenuSelection::MentallyResident) => {
                let result = mentally_resident_query().await;
                match result {
                    Ok(mentally_resident) => mentally_resident,
                    Err(InquireError::OperationCanceled) => {
                        return Box::pin(present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        ))
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
                        return Box::pin(present_set_staging_menu(
                            selected,
                            send_to_data_storage_layer,
                            default_selection,
                        ))
                        .await
                    }
                    Err(InquireError::OperationInterrupted) => return Err(()),
                    Err(err) => todo!("{:?}", err),
                }
            }
            Ok(StagingMenuSelection::Planned) => SurrealStaging::Planned,
            Ok(StagingMenuSelection::ThinkingAbout) => SurrealStaging::ThinkingAbout,
            Ok(StagingMenuSelection::Released) => SurrealStaging::Released,
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
            Ok(StagingMenuSelection::KeepAsIs(_)) => {
                //Don't change anything
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
