use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::{Local, Utc};
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    change_routine::change_routine,
    display::display_item_node::DisplayItemNode,
    menu::expectations::view_expectations,
    new_item::NewItem,
    node::item_node::ItemNode,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

use super::bullet_list_menu::present_normal_bullet_list_menu;

enum TopMenuSelection {
    Capture,
    ChangeRoutine,
    Reflection,
    ViewBulletList,
    ViewExpectations,
    ViewMotivations,
    DebugViewAllItems,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::Capture => write!(f, "🗬   Capture                  🗭"),
            TopMenuSelection::ChangeRoutine => write!(f, "↝ ↝ Change Routine            ↜"),
            TopMenuSelection::Reflection => write!(f, "    Reflection                 "),
            TopMenuSelection::ViewBulletList => write!(f, "👁 🗒️ View Bullet List (To Dos) 👁"),
            TopMenuSelection::ViewExpectations => {
                write!(f, "👁 🙏 View Expectations         👁")
            }
            TopMenuSelection::ViewMotivations => {
                write!(f, "👁 🎯 View Motivations          👁")
            }
            TopMenuSelection::DebugViewAllItems => {
                write!(f, "👁 🗒️ Debug View All Items      👁")
            }
        }
    }
}

impl TopMenuSelection {
    fn make_list() -> Vec<TopMenuSelection> {
        vec![
            Self::Capture,
            Self::ChangeRoutine,
            Self::Reflection,
            Self::ViewBulletList,
            Self::ViewExpectations,
            Self::ViewMotivations,
            Self::DebugViewAllItems,
        ]
    }
}

#[async_recursion]
pub(crate) async fn present_top_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let top_menu = TopMenuSelection::make_list();

    let selection = Select::new("Select from the below list|", top_menu).prompt();
    match selection {
        Ok(TopMenuSelection::Capture) => capture(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::ChangeRoutine) => change_routine(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::Reflection) => todo!("Implement Reflection"),
        Ok(TopMenuSelection::ViewExpectations) => {
            view_expectations(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewBulletList) => {
            present_normal_bullet_list_menu(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewMotivations) => view_motivations().await,
        Ok(TopMenuSelection::DebugViewAllItems) => {
            debug_view_all_items(send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => Err(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}

pub(crate) async fn capture(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let new_item_summary = Text::new("Enter New Item ⍠").prompt();

    match new_item_summary {
        Ok(new_item_summary) => {
            let new_item = NewItem::new(new_item_summary, Utc::now());
            send_to_data_storage_layer
                .send(DataLayerCommands::NewItem(new_item))
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => Ok(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}

async fn view_motivations() -> Result<(), ()> {
    todo!()
}

enum DebugViewItem<'e> {
    Item(DisplayItemNode<'e>),
}

impl Display for DebugViewItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugViewItem::Item(item) => write!(f, "{}", item),
        }
    }
}

impl<'e> DebugViewItem<'e> {
    fn make_list(items: &'e [&'e ItemNode<'e>]) -> Vec<DebugViewItem<'e>> {
        items.iter().copied().map(DebugViewItem::new).collect()
    }

    fn new(item: &'e ItemNode<'e>) -> Self {
        Self::Item(DisplayItemNode::new(item, None))
    }
}

async fn debug_view_all_items(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let active_items = base_data.get_active_items();
    let covering = base_data.get_coverings();
    let covering_until_date_time = base_data.get_coverings_until_date_time();
    let active_covering_until_date_time = base_data.get_active_snoozed();

    let item_nodes = active_items
        .iter()
        .map(|x| ItemNode::new(x, covering, active_covering_until_date_time, active_items))
        .collect::<Vec<_>>();

    let item_nodes = item_nodes.iter().collect::<Vec<_>>();
    let list = DebugViewItem::make_list(&item_nodes);

    let selection = Select::new("Select an item to show the debug view of...", list).prompt();
    match selection {
        Ok(DebugViewItem::Item(item)) => {
            println!("{}", item);
            println!("{:#?}", item.get_item_node());

            let item_node = item.get_item_node();
            let item = item.get_item_node().get_item();
            let covered_by = item.get_covered_by_another_item(covering);
            println!("Covered by: {:#?}", covered_by);

            let cover_others = item.get_covering_another_item(covering);
            println!("Covering others: {:#?}", cover_others);

            let local_now = Local::now();
            let covered_by_date_time = item
                .get_covered_by_date_time_filter_out_the_past(covering_until_date_time, &local_now);
            println!("Covered by date time: {:#?}", covered_by_date_time);

            println!("Lap Count: {}", item_node.get_lap_count(&now));
            Ok(())
        }
        Err(InquireError::OperationCanceled) => present_top_menu(send_to_data_storage_layer).await,
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}
