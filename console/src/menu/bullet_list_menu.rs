pub(crate) mod bullet_list_single_item;

use std::{fmt::Display, iter::once};

use async_recursion::async_recursion;
use chrono::{DateTime, Local, Utc};
use inquire::{InquireError, Select};
use itertools::chain;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    display::display_item_node::DisplayItemNode,
    menu::top_menu::present_top_menu,
    node::item_node::ItemNode,
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
    systems::bullet_list::{BulletList, BulletListReason},
};

use self::bullet_list_single_item::{
    present_bullet_list_item_selected, present_is_person_or_group_around_menu,
    set_staging::present_set_staging_menu,
};

use super::top_menu::capture;

pub(crate) enum InquireBulletListItem<'e> {
    CaptureNewItem,
    SetStaging(&'e ItemNode<'e>),
    Item(&'e ItemNode<'e>, &'e DateTime<Utc>),
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureNewItem => write!(f, "🗬   Capture New Item          🗭")?,
            Self::Item(item_node, current_date_time) => {
                let display_item_node = DisplayItemNode::new(item_node, Some(current_date_time));
                write!(f, "{}", display_item_node)?;
            }
            Self::SetStaging(item_node) => {
                let display_item_node = DisplayItemNode::new(item_node, None);
                write!(f, "[SET STAGING] {}", display_item_node)?;
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub(crate) fn create_list(
        item_nodes: &'a [BulletListReason<'a>],
        current_date_time: &'a DateTime<Utc>,
    ) -> Vec<InquireBulletListItem<'a>> {
        chain!(
            once(InquireBulletListItem::CaptureNewItem),
            item_nodes.iter().map(|x| match x {
                BulletListReason::SetStaging(item_node) =>
                    InquireBulletListItem::SetStaging(item_node),
                BulletListReason::WorkOn(item_node) => {
                    InquireBulletListItem::Item(item_node, current_date_time)
                }
            })
        )
        .collect()
    }
}

#[async_recursion]
pub(crate) async fn present_normal_bullet_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    println!(
        "Time to get data from database: {}",
        Local::now() - before_db_query
    );
    let current_date_time = Utc::now();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let bullet_list = BulletList::new_bullet_list(base_data, &current_date_time);
    println!("Time to create bullet list: {}", Utc::now() - now);
    present_bullet_list_menu(bullet_list, &current_date_time, send_to_data_storage_layer).await
}

pub(crate) async fn present_bullet_list_menu(
    bullet_list: BulletList,
    current_date_time: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let item_nodes = bullet_list.get_bullet_list();

    let inquire_bullet_list = InquireBulletListItem::create_list(item_nodes, current_date_time);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("Select from the below list|", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::CaptureNewItem) => capture(send_to_data_storage_layer).await,
            Ok(InquireBulletListItem::Item(item_node, current_date_time)) => {
                if item_node.is_person_or_group() {
                    present_is_person_or_group_around_menu(item_node, send_to_data_storage_layer)
                        .await
                } else {
                    present_bullet_list_item_selected(
                        item_node,
                        current_date_time,
                        bullet_list.get_coverings(),
                        bullet_list.get_active_snoozed(),
                        bullet_list.get_active_items(),
                        send_to_data_storage_layer,
                    )
                    .await
                }
            }
            Ok(InquireBulletListItem::SetStaging(item_node)) => {
                present_set_staging_menu(item_node.get_item(), send_to_data_storage_layer).await
            }
            Err(InquireError::OperationCanceled) => {
                present_top_menu(send_to_data_storage_layer).await
            }
            Err(InquireError::OperationInterrupted) => Err(()),
            Err(err) => todo!("Unexpected InquireError of {}", err),
        }
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(send_to_data_storage_layer).await
    }
}

#[cfg(test)]
mod tests {}
