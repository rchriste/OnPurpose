use std::{
    cmp::Ordering, fmt::{Display, Formatter}, iter::once
};

use inquire::{InquireError, Select};
use itertools::{chain, Itertools};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::mode::Mode, data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_current_mode::NewCurrentMode,
    }, display::display_mode_node::DisplayModeNode, menu::inquire::do_now_list_menu::present_normal_do_now_list_menu, node::mode_node::ModeNode, systems::do_now_list::current_mode::CurrentMode
};

use super::{CalculatedData, DisplayFormat};

pub(crate) enum InTheModeChoices<'e> {
    AddNewMode,
    SelectExistingMode(&'e ModeNode<'e>),
    ClearModeSelection,
}

impl Display for InTheModeChoices<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddNewMode => write!(f, "➕ Add New Mode"),
            Self::SelectExistingMode(mode_node) => write!(
                f,
                "{}",
                DisplayModeNode::new(mode_node, DisplayFormat::SingleLine)
            ),
        }
    }
}

enum DetailsMenu {
    ReturnToDoNowList,
    Rename,
    EditWhatIsInTheMode,
}

impl Display for DetailsMenu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Select(mode_node) => write!(
                f,
                "Set Current Mode to: {}",
                DisplayModeNode::new(mode_node, DisplayFormat::SingleLine)
            ),
            Self::Rename => write!(f, "Rename Mode"),
        }
    }
}

pub(crate) async fn present_change_mode_menu<'a>(
    current_mode: &'a CurrentMode<'a>,
    calculated_data: &'a CalculatedData,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let mut mode_nodes = calculated_data.get_mode_nodes().iter().collect::<Vec<_>>();
    mode_nodes.sort_by(|a, b| {
        fn compare_chains(
            mut a_parent_chain: Vec<&Mode<'_>>,
            mut b_parent_chain: Vec<&Mode<'_>>,
        ) -> Ordering {
            let a_parent_chain_last = a_parent_chain.last();
            let b_parent_chain_last = b_parent_chain.last();
            if a_parent_chain_last.is_none() && b_parent_chain_last.is_none() {
                Ordering::Equal
            } else if a_parent_chain_last.is_none() {
                Ordering::Less
            } else if b_parent_chain_last.is_none() {
                Ordering::Greater
            } else {
                let a_parent_chain_last =
                    a_parent_chain_last.expect("Earlier if statement guarantees this is_some()");
                let b_parent_chain_last =
                    b_parent_chain_last.expect("Earlier if statement guarantees this is_some()");
                let ordering = a_parent_chain_last
                    .get_name()
                    .cmp(b_parent_chain_last.get_name());
                if let Ordering::Equal = ordering {
                    a_parent_chain.pop();
                    b_parent_chain.pop();
                    compare_chains(a_parent_chain, b_parent_chain)
                } else {
                    ordering
                }
            }
        }

        let a_parent_chain = a.create_parent_chain();
        let b_parent_chain = b.create_parent_chain();
        compare_chains(a_parent_chain, b_parent_chain)
    });


    let choices = chain!(
        once(InTheModeChoices::AddNewMode),
        mode_nodes
            .into_iter()
            .map(|x| InTheModeChoices::SelectExistingMode(x)),
        once(InTheModeChoices::ClearModeSelection)
    )
    .collect::<Vec<_>>();
    let default_choice = choices
        .iter()
        .position(|x| match x {
            InTheModeChoices::SelectExistingMode(mode_node) => {
                mode_node.get_surreal_id() == current_mode.get_mode().get_surreal_id()
            }
            InTheModeChoices::AddNewMode |
            InTheModeChoices::ClearModeSelection => false,
        })
        .unwrap_or_default();
    let selection = Select::new("Select Mode to Change to", choices)
        .with_starting_cursor(default_choice)
        .prompt();
    match selection {
        Ok(InTheModeChoices::AddNewMode) => {
            todo!("Prompt for who the parent node should be and then prompt to name the mode, then set this new mode as the current mode and bring the user to the InTheModeChoices::SelectExistingMode menu so they can choose to edit what is in and out of the mode if they would like otherwise to continue")
        }
        Ok(InTheModeChoices::SelectExistingMode(mode_node)) => {
            let new_current_mode =
            NewCurrentMode::new(Some(mode_node.get_surreal_id().clone()));
        send_to_data_storage_layer
            .send(DataLayerCommands::SetCurrentMode(new_current_mode))
            .await
            .unwrap();

            let selection = Select::new(
                "Select one",
                vec![DetailsMenu::ReturnToDoNowList, DetailsMenu::EditWhatIsInTheMode, DetailsMenu::Rename],
            )
            .prompt();
            match selection {
                Ok(DetailsMenu::ReturnToDoNowList) => {
                    Ok(())
                }
                Ok(DetailsMenu::EditWhatIsInTheMode) => {
                    todo!("Present the menu to edit what is in and out of the mode, core work, non-core work, and explicitly out of scope work")
                }
                Ok(DetailsMenu::Rename) => {
                    let name = inquire::Text::new("Enter the new name of the mode")
                        .with_default(mode_node.get_name())
                        .prompt();
                    match name {
                        Ok(name) => {
                            send_to_data_storage_layer
                                .send(DataLayerCommands::UpdateModeSummary(
                                    mode_node.get_surreal_id().clone(),
                                    name,
                                ))
                                .await
                                .unwrap();

                            Ok(())

                        }
                        Err(InquireError::OperationCanceled) => {
                            Ok(()) //Just let it fall back to the normal menu
                        }
                        Err(InquireError::OperationInterrupted) => Err(()),
                        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
                    }
                }
                Err(InquireError::OperationCanceled) => {
                    Ok(()) //Just let it fall back to the normal menu
                }
                Err(InquireError::OperationInterrupted) => return Err(()),
                Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
            }
        }
        Err(InquireError::OperationCanceled) => {
            Ok(()) //Just let it fall back to the normal menu
        }
        Err(InquireError::OperationInterrupted) => return Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}
