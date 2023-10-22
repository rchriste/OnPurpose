pub(crate) mod bullet_list_single_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use chrono::Local;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        hope::Hope,
        item::{Item, ItemVecExtensions},
        to_do::ToDo,
    },
    display_item::DisplayItem,
    mentally_resident::{
        create_hope_nodes, present_mentally_resident_hope_selected_menu, HopeNode,
    },
    node::to_do_node::{create_to_do_nodes, ToDoNode},
    surrealdb_layer::DataLayerCommands,
    top_menu::present_top_menu,
};

use self::bullet_list_single_item::present_bullet_list_item_selected;

pub(crate) enum InquireBulletListItem<'a> {
    ViewFocusItems,
    NextStepToDo {
        to_do: &'a ToDo<'a>,
        parents: Vec<&'a Item<'a>>,
    },
    NeedsNextStepHope {
        hope: &'a Hope<'a>,
        parents: Vec<&'a Item<'a>>,
    },
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ViewFocusItems => write!(f, "⏲️  [View Focus Items] ⏲️")?,
            Self::NextStepToDo { to_do, parents } => {
                let display_item = DisplayItem::new(to_do.into());
                write!(f, "{} ", display_item)?;
                for item in parents {
                    let display_item = DisplayItem::new(item);
                    write!(f, " ⬅ {}", display_item)?;
                }
            }
            Self::NeedsNextStepHope { hope, parents } => {
                let display_item = DisplayItem::new(hope.into());
                write!(f, "[NEEDS NEXT STEP] ⬅ {}", display_item)?;
                for item in parents {
                    let display_item = DisplayItem::new(item);
                    write!(f, " ⬅  {}", display_item)?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub(crate) fn create_list_with_view_focus_items_option(
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let mut list =
            Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len() + 1);
        list.push(Self::ViewFocusItems);
        Self::add_items_to_list(list, next_step_nodes, hopes_without_a_next_step)
    }

    pub(crate) fn create_list_just_items(
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let list = Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len());
        Self::add_items_to_list(list, next_step_nodes, hopes_without_a_next_step)
    }

    fn add_items_to_list(
        mut list: Vec<InquireBulletListItem<'a>>,
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        list.extend(
            next_step_nodes
                .iter()
                .map(|x| InquireBulletListItem::NextStepToDo {
                    to_do: x.to_do,
                    parents: x.create_next_step_parents(),
                }),
        );
        list.extend(hopes_without_a_next_step.iter().map(|x| {
            InquireBulletListItem::NeedsNextStepHope {
                hope: x.hope,
                parents: x.towards_motivation_chain.clone(),
            }
        }));
        list
    }
}

#[async_recursion]
pub(crate) async fn present_unfocused_bullet_list_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let active_items = items.filter_active_items();
    let coverings = surreal_tables.make_coverings(&items);
    let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

    let to_dos = &items.filter_just_to_dos();
    let current_date_time = Local::now();
    let next_step_nodes = create_to_do_nodes(
        to_dos,
        &coverings,
        &coverings_until_date_time,
        &active_items,
        &current_date_time,
        false,
    );

    let mentally_resident_hopes: Vec<Hope<'_>> = items
        .filter_just_hopes(&surreal_tables.surreal_specific_to_hopes)
        .into_iter()
        .filter(|x| x.is_mentally_resident() && x.is_project())
        .collect();
    let hope_nodes = create_hope_nodes(&mentally_resident_hopes, &coverings);
    let hope_nodes_needing_a_next_step: Vec<HopeNode<'_>> = hope_nodes
        .into_iter()
        .filter(|x| x.next_steps.is_empty())
        .collect();

    let inquire_bullet_list = InquireBulletListItem::create_list_with_view_focus_items_option(
        &next_step_nodes,
        &hope_nodes_needing_a_next_step,
    );

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                present_focused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::NextStepToDo { to_do, parents }) => {
                present_bullet_list_item_selected(to_do, &parents, send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::NeedsNextStepHope { hope, parents: _ }) => {
                present_mentally_resident_hope_selected_menu(hope, send_to_data_storage_layer).await
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
    let surreal_tables = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();

    let items = surreal_tables.make_items();
    let active_items = items.filter_active_items();
    let coverings = surreal_tables.make_coverings(&items);
    let coverings_until_date_time = surreal_tables.make_coverings_until_date_time(&items);

    let to_dos = &items.filter_just_to_dos();
    let current_date_time = Local::now();
    let next_step_nodes = create_to_do_nodes(
        to_dos,
        &coverings,
        &coverings_until_date_time,
        &active_items,
        &current_date_time,
        true,
    );

    let hopes_without_a_next_step = vec![]; //Hopes without a next step cannot be focus items

    let inquire_bullet_list =
        InquireBulletListItem::create_list_just_items(&next_step_nodes, &hopes_without_a_next_step);

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::NextStepToDo { to_do, parents }) => {
                present_bullet_list_item_selected(to_do, &parents, send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::NeedsNextStepHope {
                hope: _,
                parents: _,
            }) => panic!("The focus list should not present this option"),
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
