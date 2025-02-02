use tokio::sync::mpsc::Sender;

use crate::{
    data_storage::surrealdb_layer::data_layer_commands::DataLayerCommands,
    node::{item_status::ItemStatus, mode_node::ModeNode},
    systems::do_now_list::current_mode::CurrentMode,
};

pub(crate) async fn present_state_if_in_mode_menu(
    item_status: &ItemStatus<'_>,
    current_mode: &ModeNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    todo!()
}
