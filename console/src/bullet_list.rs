pub mod bullet_list_single_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Local;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{Item, ItemType, ItemVecExtensions, ToDo},
    create_next_step_parents,
    node::{create_to_do_nodes, ToDoNode},
    surrealdb_layer::DataLayerCommands,
    top_menu::present_top_menu,
};

use self::bullet_list_single_item::present_bullet_list_item_selected;

pub enum InquireBulletListItem<'a> {
    ViewFocusItems,
    Item {
        bullet_item: &'a ToDo<'a>,
        parents: Vec<&'a Item<'a>>,
    },
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ViewFocusItems => write!(f, "⏲️  [View Focus Items] ⏲️")?,
            Self::Item {
                bullet_item,
                parents,
            } => {
                write!(f, "{} ", bullet_item.summary)?;
                for item in parents {
                    match item.item_type {
                        ItemType::ToDo => write!(f, "⬅ 🪜  ")?,
                        ItemType::Hope => write!(f, "⬅ 🧠 ")?,
                        ItemType::Motivation => write!(f, "⬅ 🎯 ")?,
                    }
                    write!(f, "{}", item.summary)?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub fn create_list_with_view_focus_items_option(
        next_step_nodes: &'a Vec<ToDoNode<'a>>,
    ) -> Vec<InquireBulletListItem<'a>> {
        let mut list = Vec::with_capacity(next_step_nodes.len() + 1);
        list.push(Self::ViewFocusItems);
        Self::add_items_to_list(list, next_step_nodes)
    }

    pub fn create_list_just_items(
        next_step_nodes: &'a Vec<ToDoNode<'a>>,
    ) -> Vec<InquireBulletListItem<'a>> {
        let list = Vec::with_capacity(next_step_nodes.len() + 1);
        Self::add_items_to_list(list, next_step_nodes)
    }

    fn add_items_to_list(
        mut list: Vec<InquireBulletListItem<'a>>,
        next_step_nodes: &'a Vec<ToDoNode<'a>>,
    ) -> Vec<InquireBulletListItem<'a>> {
        list.extend(next_step_nodes.iter().map(|x| InquireBulletListItem::Item {
            bullet_item: x.to_do,
            parents: create_next_step_parents(x),
        }));
        list
    }
}

#[async_recursion]
pub async fn present_unfocused_bullet_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);
    let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

    let to_dos = &items.filter_just_to_dos();
    let current_date_time = Local::now();
    let next_step_nodes = create_to_do_nodes(
        to_dos,
        &coverings,
        &coverings_until_date_time,
        &current_date_time,
        false,
    );

    let inquire_bullet_list =
        InquireBulletListItem::create_list_with_view_focus_items_option(&next_step_nodes);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(30)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                present_focused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::Item {
                bullet_item,
                parents: _,
            }) => present_bullet_list_item_selected(bullet_item, send_to_data_storage_layer).await,
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
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let coverings = surreal_tables.make_coverings(&items);
    let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

    let to_dos = &items.filter_just_to_dos();
    let current_date_time = Local::now();
    let next_step_nodes = create_to_do_nodes(
        to_dos,
        &coverings,
        &coverings_until_date_time,
        &current_date_time,
        true,
    );

    let inquire_bullet_list =
        InquireBulletListItem::create_list_just_items(&next_step_nodes);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(30)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => panic!("The focus list should not present this option"),
            Ok(InquireBulletListItem::Item {
                bullet_item,
                parents: _,
            }) => present_bullet_list_item_selected(bullet_item, send_to_data_storage_layer).await,
            Err(InquireError::OperationCanceled) => {
                present_unfocused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Err(err) => todo!("Unexpected InquireError of {}", err),
        };
    } else {
        println!("To Do List is Empty, falling back to main menu");
        present_top_menu(send_to_data_storage_layer).await
    }
}
