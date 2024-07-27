use core::fmt;
use std::{
    fmt::{Display, Formatter},
    iter::once,
};

use inquire::{InquireError, Select};
use itertools::chain;
use tokio::sync::mpsc::Sender;

use crate::{
    display::display_item::DisplayItem,
    menu::inquire::select_higher_importance_than_this::HigherImportanceThan,
    node::{item_node::ItemNode, Filter},
    surrealdb_layer::data_layer_commands::DataLayerCommands,
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