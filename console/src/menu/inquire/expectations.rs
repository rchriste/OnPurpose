pub(crate) mod define_facing;

use std::fmt::Display;

use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    data_storage::surrealdb_layer::data_layer_commands::DataLayerCommands,
    menu::inquire::{expectations::define_facing::define_facing, top_menu::present_top_menu},
};

enum ExpectationsMenuItem {
    DefineFacing,
}

impl Display for ExpectationsMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefineFacing => write!(f, "👀 Define Facing              👀"),
        }
    }
}

impl ExpectationsMenuItem {
    fn make_list() -> Vec<Self> {
        vec![Self::DefineFacing]
    }
}

pub(crate) async fn view_expectations(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ExpectationsMenuItem::make_list();

    let selection = Select::new("Select from the below list|", list).prompt();

    match selection {
        Ok(ExpectationsMenuItem::DefineFacing) => define_facing(send_to_data_storage_layer).await,
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_top_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}
