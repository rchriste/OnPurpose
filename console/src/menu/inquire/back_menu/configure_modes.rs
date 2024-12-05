use std::fmt::{Display, Formatter};

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    display::display_mode_node::DisplayModeNode,
    menu::inquire::back_menu::{present_back_menu, SurrealTables},
    new_mode::NewModeBuilder,
    node::mode_node::ModeNode,
};

use super::{DataLayerCommands, DisplayFormat};

enum ConfigureModesOptions<'e> {
    Add,
    Done,
    Mode(&'e ModeNode<'e>),
}

impl Display for ConfigureModesOptions<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigureModesOptions::Add => write!(f, "Add New Top Level Mode"),
            ConfigureModesOptions::Done => write!(f, "Done (Return to \"Do Now\" List)"),
            ConfigureModesOptions::Mode(mode) => write!(
                f,
                "{}",
                DisplayModeNode::new(mode, DisplayFormat::SingleLine)
            ),
        }
    }
}

enum ConfigureModesOptionsSelected<'e> {
    AddWithParent(&'e ModeNode<'e>),
    Done,
}

impl Display for ConfigureModesOptionsSelected<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigureModesOptionsSelected::AddWithParent(parent) => write!(
                f,
                "Add New Mode Under {}",
                DisplayModeNode::new(parent, DisplayFormat::SingleLine)
            ),
            ConfigureModesOptionsSelected::Done => write!(f, "Done (Return to \"Do Now\" List)"),
        }
    }
}

pub(crate) async fn configure_modes(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data);

    let mut options = vec![ConfigureModesOptions::Add];
    options.extend(
        calculated_data
            .get_mode_nodes()
            .iter()
            .map(ConfigureModesOptions::Mode),
    );
    options.push(ConfigureModesOptions::Done);

    let selection = Select::new("Select a mode", options).prompt();
    match selection {
        Ok(ConfigureModesOptions::Add) => {
            let name = inquire::Text::new("Enter the name of the new mode").prompt();
            match name {
                Ok(name) => {
                    let new_mode = NewModeBuilder::default().name(name).build().unwrap();
                    send_to_data_storage_layer
                        .send(DataLayerCommands::NewMode(new_mode))
                        .await
                        .unwrap();

                    Box::pin(configure_modes(send_to_data_storage_layer)).await
                }
                Err(InquireError::OperationCanceled) => {
                    Box::pin(configure_modes(send_to_data_storage_layer)).await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(_) => {
                    todo!()
                }
            }
        }
        Ok(ConfigureModesOptions::Mode(mode)) => {
            let options = vec![
                ConfigureModesOptionsSelected::AddWithParent(mode),
                ConfigureModesOptionsSelected::Done,
            ];

            let selection = Select::new("Select an option", options).prompt();
            match selection {
                Ok(ConfigureModesOptionsSelected::AddWithParent(parent)) => {
                    let name = inquire::Text::new("Enter the name of the new mode").prompt();
                    match name {
                        Ok(name) => {
                            let new_mode = NewModeBuilder::default()
                                .name(name)
                                .parent(Some(parent.get_surreal_id().clone()))
                                .build()
                                .unwrap();
                            send_to_data_storage_layer
                                .send(DataLayerCommands::NewMode(new_mode))
                                .await
                                .unwrap();

                            Box::pin(configure_modes(send_to_data_storage_layer)).await
                        }
                        Err(InquireError::OperationCanceled) => {
                            Box::pin(configure_modes(send_to_data_storage_layer)).await
                        }
                        Err(InquireError::OperationInterrupted) => Err(()),
                        Err(_) => {
                            todo!()
                        }
                    }
                }
                Ok(ConfigureModesOptionsSelected::Done) => Ok(()),
                Err(InquireError::OperationCanceled) => {
                    Box::pin(configure_modes(send_to_data_storage_layer)).await
                }
                Err(InquireError::OperationInterrupted) => Err(()),
                Err(_) => {
                    todo!()
                }
            }
        }
        Ok(ConfigureModesOptions::Done) => Ok(()),
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_back_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(_) => {
            todo!()
        }
    }
}
