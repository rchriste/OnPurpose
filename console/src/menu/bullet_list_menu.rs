pub(crate) mod bullet_list_single_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use inquire::{InquireError, Select};
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

pub(crate) enum InquireBulletListItem<'e> {
    ViewFocusItems,
    ViewOldBulletList,
    SetStaging(&'e ItemNode<'e>),
    Item(&'e ItemNode<'e>, &'e DateTime<Utc>),
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ViewFocusItems => write!(f, "⏲️  [View Focus Items] ⏲️")?,
            Self::ViewOldBulletList => write!(f, "⏲️  [View Old Bullet List] ⏲️")?,
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
    pub(crate) fn create_list_with_view_focus_items_option(
        item_nodes: &'a [BulletListReason<'a>],
        current_date_time: &'a DateTime<Utc>,
    ) -> Vec<InquireBulletListItem<'a>> {
        let mut list = Vec::with_capacity(item_nodes.len() + 2);
        list.push(Self::ViewFocusItems);
        let mut list = Self::add_items_to_list(list, item_nodes, current_date_time);
        list.push(Self::ViewOldBulletList);
        list
    }

    pub(crate) fn create_list_just_items(
        item_nodes: &'a [BulletListReason<'a>],
        current_date_time: &'a DateTime<Utc>,
    ) -> Vec<InquireBulletListItem<'a>> {
        let list = Vec::with_capacity(item_nodes.len());
        Self::add_items_to_list(list, item_nodes, current_date_time)
    }

    fn add_items_to_list(
        mut list: Vec<InquireBulletListItem<'a>>,
        item_nodes: &'a [BulletListReason<'a>],
        current_date_time: &'a DateTime<Utc>,
    ) -> Vec<InquireBulletListItem<'a>> {
        list.extend(item_nodes.iter().map(|x| match x {
            BulletListReason::SetStaging(item_node) => InquireBulletListItem::SetStaging(item_node),
            BulletListReason::WorkOn(item_node) => {
                InquireBulletListItem::Item(item_node, current_date_time)
            }
        }));
        list
    }
}

#[async_recursion]
pub(crate) async fn present_unfocused_bullet_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let current_date_time = Utc::now();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let bullet_list = BulletList::new_unfocused_bullet_list(base_data);
    present_bullet_list_menu(bullet_list, &current_date_time, send_to_data_storage_layer).await;
}

#[async_recursion]
pub(crate) async fn present_normal_bullet_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let current_date_time = Utc::now();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let bullet_list = BulletList::new_bullet_list(base_data, &current_date_time);
    present_bullet_list_menu(bullet_list, &current_date_time, send_to_data_storage_layer).await;
}

pub(crate) async fn present_bullet_list_menu(
    bullet_list: BulletList,
    current_date_time: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let item_nodes = bullet_list.get_bullet_list();

    let inquire_bullet_list = InquireBulletListItem::create_list_with_view_focus_items_option(
        item_nodes,
        current_date_time,
    );

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("Select from the below list", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                present_focused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::ViewOldBulletList) => {
                present_unfocused_bullet_list_menu(send_to_data_storage_layer).await
            }
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
                present_set_staging_menu(item_node, send_to_data_storage_layer).await
            }
            Err(InquireError::OperationCanceled) => {
                present_top_menu(send_to_data_storage_layer).await
            }
            Err(err) => todo!("Unexpected InquireError of {}", err),
        };
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(send_to_data_storage_layer).await
    }
}

async fn present_focused_bullet_list_menu(send_to_data_storage_layer: &Sender<DataLayerCommands>) {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let current_date_time = Utc::now();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let bullet_list = BulletList::new_focused_bullet_list(base_data);

    let item_nodes = bullet_list.get_bullet_list();

    let inquire_bullet_list =
        InquireBulletListItem::create_list_just_items(item_nodes, &current_date_time);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("Select from the below list", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::ViewOldBulletList) => {
                present_unfocused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::Item(item_node, &current_date_time)) => {
                present_bullet_list_item_selected(
                    item_node,
                    &current_date_time,
                    bullet_list.get_coverings(),
                    bullet_list.get_active_snoozed(),
                    bullet_list.get_active_items(),
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::SetStaging(item_node)) => {
                present_set_staging_menu(item_node, send_to_data_storage_layer).await
            }
            Err(InquireError::OperationCanceled) => {
                present_normal_bullet_list_menu(send_to_data_storage_layer).await
            }
            Err(err) => todo!("Unexpected InquireError of {}", err),
        };
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(send_to_data_storage_layer).await
    }
}

#[cfg(test)]
mod tests {}
