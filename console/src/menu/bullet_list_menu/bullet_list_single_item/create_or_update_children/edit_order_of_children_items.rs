use core::fmt;
use std::{
    fmt::{Display, Formatter},
    iter::once,
};

use async_recursion::async_recursion;
use inquire::{InquireError, Select};
use itertools::chain;
use tokio::sync::mpsc::Sender;

use crate::{
    display::display_item::DisplayItem, menu::select_higher_priority_than_this::HigherPriorityThan,
    node::item_node::ItemNode, surrealdb_layer::DataLayerCommands,
};

enum EditOrderOfChildren<'e> {
    Done,
    Item(DisplayItem<'e>),
}

impl Display for EditOrderOfChildren<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EditOrderOfChildren::Done => write!(f, "Done"),
            EditOrderOfChildren::Item(display_item) => write!(f, "{}", display_item),
        }
    }
}

#[async_recursion]
pub(crate) async fn edit_order_of_children_items(
    item_node: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let smaller = item_node.get_smaller();
    let list = chain!(
        once(EditOrderOfChildren::Done),
        smaller
            .iter()
            .map(|item_node| EditOrderOfChildren::Item(DisplayItem::new(item_node.get_item())))
    )
    .collect::<Vec<_>>();
    let selection = Select::new("Select an item to move", list).prompt();
    match selection {
        Ok(EditOrderOfChildren::Item(selection)) => {
            let selected_item = selection.get_item();
            let items = smaller
                .iter()
                .map(|x| x.get_item())
                //Don't include the item that was selected
                .filter(|x| *x != selected_item)
                .collect::<Vec<_>>();
            let list = HigherPriorityThan::create_list(&items);
            let selected =
                Select::new("Select new position, higher priority than this|", list).prompt();
            todo!()
        }
        Ok(EditOrderOfChildren::Done) => Ok(()),
        Err(InquireError::OperationCanceled) => todo!("Return to what calls this"),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error: {:?}", err),
    }
}
