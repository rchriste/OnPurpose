use std::{cmp::Ordering, fmt::Display};

use chrono::{Local, Utc};
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    change_routine::change_routine,
    display::{
        display_item_node::DisplayItemNode, display_item_status::DisplayItemStatus,
        display_priority::DisplayPriority,
    },
    menu::inquire::expectations::view_expectations,
    new_item::NewItem,
    node::{item_node::ItemNode, Filter},
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
};

use super::bullet_list_menu::present_normal_bullet_list_menu;

enum TopMenuSelection {
    ChangeRoutine,
    Reflection,
    ViewBulletList,
    ViewExpectations,
    ViewMotivations,
    ViewPriorities,
    DebugViewAllItems,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::ChangeRoutine => write!(f, "↝ ↝  Change Routine            ↜"),
            TopMenuSelection::Reflection => write!(f, "      Reflection                 "),
            TopMenuSelection::ViewBulletList => write!(f, "👁 🗒️  View Bullet List (To Dos) 👁"),
            TopMenuSelection::ViewExpectations => {
                write!(f, "👁 🙏 View Expectations          👁")
            }
            TopMenuSelection::ViewMotivations => {
                write!(f, "👁 🎯 View Motivations           👁")
            }
            TopMenuSelection::ViewPriorities => write!(f, "👁 ⚖️  View Priorities           👁"),
            TopMenuSelection::DebugViewAllItems => {
                write!(f, "👁 🗒️  Debug View All Items      👁")
            }
        }
    }
}

impl TopMenuSelection {
    fn make_list() -> Vec<TopMenuSelection> {
        vec![
            Self::ViewPriorities,
            Self::ChangeRoutine,
            Self::Reflection,
            Self::ViewBulletList,
            Self::ViewExpectations,
            Self::ViewMotivations,
            Self::DebugViewAllItems,
        ]
    }
}

pub(crate) async fn present_top_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let top_menu = TopMenuSelection::make_list();

    let selection = Select::new("Select from the below list|", top_menu).prompt();
    match selection {
        Ok(TopMenuSelection::ChangeRoutine) => change_routine(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::Reflection) => todo!("Implement Reflection"),
        Ok(TopMenuSelection::ViewExpectations) => {
            view_expectations(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewBulletList) => {
            present_normal_bullet_list_menu(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewMotivations) => view_motivations().await,
        Ok(TopMenuSelection::ViewPriorities) => view_priorities(send_to_data_storage_layer).await,
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

async fn view_priorities(send_to_data_storage_layer: &Sender<DataLayerCommands>) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let elapsed = Local::now() - before_db_query;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to get data from database. Time taken: {}", elapsed);
    }
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data, &now);

    let mut all_top_nodes = calculated_data
        .get_item_status()
        .iter()
        .filter(|x| !x.is_finished())
        //Person or group items without a parent, meaning a reason for being on the list,
        // should be filtered out.
        .filter(|x| x.has_children(Filter::Active) && !x.has_larger(Filter::Active))
        .cloned()
        .collect::<Vec<_>>();

    all_top_nodes.sort_by(|a, b| {
        (if a.is_type_motivation() {
            if b.is_type_motivation() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if b.is_type_motivation() {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
        .then_with(|| a.get_summary().cmp(b.get_summary()))
    });

    let list = all_top_nodes
        .iter()
        .map(DisplayItemStatus::new)
        .collect::<Vec<_>>();

    let selection = Select::new("Select a priority to view...", list).prompt();
    match selection {
        Ok(display_item_status) => {
            view_priorities_of_item_status(
                display_item_status,
                Vec::new(),
                &calculated_data,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_top_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}

async fn view_priorities_of_item_status(
    display_item_status: DisplayItemStatus<'_>,
    mut parent: Vec<DisplayItemStatus<'_>>,
    calculated_data: &CalculatedData,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    println!("{}", display_item_status);
    let item_status = display_item_status.get_item_status();
    println!("Active children (Is this in priority order?):");
    let list = item_status
        .get_smaller(Filter::Active)
        .map(|x| {
            let item_status = calculated_data
                .get_item_status()
                .iter()
                .find(|y| y.get_item() == x.get_item())
                .expect("Comes from this list so will be found");
            DisplayPriority::new(item_status, calculated_data.get_item_status())
        })
        .collect();
    let selection = Select::new("Select a child to view...", list).prompt();
    match selection {
        Ok(display_priority) => {
            println!("{}", display_priority);
            if display_priority.has_children() {
                parent.push(display_item_status);
                let display_item_status =
                    DisplayItemStatus::new(display_priority.get_item_status());
                Box::pin(view_priorities_of_item_status(
                    display_item_status,
                    parent,
                    calculated_data,
                    send_to_data_storage_layer,
                ))
                .await
            } else {
                Ok(())
            }
        }
        Err(InquireError::OperationCanceled) => {
            if parent.is_empty() {
                Box::pin(view_priorities(send_to_data_storage_layer)).await
            } else {
                let top_item = parent.pop().expect("is not empty so will always succeed");
                Box::pin(view_priorities_of_item_status(
                    top_item,
                    parent,
                    calculated_data,
                    send_to_data_storage_layer,
                ))
                .await
            }
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
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
        Self::Item(DisplayItemNode::new(item))
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
    let all_items = base_data.get_items();
    let active_items = base_data.get_active_items();
    let covering = base_data.get_coverings();
    let covering_until_date_time = base_data.get_coverings_until_date_time();
    let active_covering_until_date_time = base_data.get_active_snoozed();

    let item_nodes = active_items
        .iter()
        .map(|x| ItemNode::new(x, covering, active_covering_until_date_time, all_items))
        .collect::<Vec<_>>();

    let item_nodes = item_nodes.iter().collect::<Vec<_>>();
    let list = DebugViewItem::make_list(&item_nodes);

    let selection = Select::new("Select an item to show the debug view of...", list).prompt();
    match selection {
        Ok(DebugViewItem::Item(item)) => {
            println!("{}", item);
            let item_node = item.get_item_node();
            println!("{:#?}", item_node);

            let item = item_node.get_item();
            let covered_by = item.get_covered_by_another_item(covering);
            println!("Covered by: {:#?}", covered_by);

            let cover_others = item.get_covering_another_item(covering);
            println!("Covering others: {:#?}", cover_others);

            let local_now = Local::now();
            let covered_by_date_time = item
                .get_covered_by_date_time_filter_out_the_past(covering_until_date_time, &local_now);
            println!("Covered by date time: {:#?}", covered_by_date_time);

            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_top_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}
