use tokio::sync::mpsc::Sender;

use crate::{
    data_storage::surrealdb_layer::data_layer_commands::DataLayerCommands,
    node::item_status::ItemStatus,
};

use super::do_now_list_single_item::give_this_item_a_parent::give_this_item_a_parent;

pub(crate) async fn present_parent_back_to_a_motivation_menu(
    item_status: &ItemStatus<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    give_this_item_a_parent(item_status.get_item(), true, send_to_data_storage_layer).await
}
