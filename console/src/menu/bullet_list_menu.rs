pub(crate) mod bullet_list_single_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{hope::Hope, item::Item, to_do::ToDo, BaseData},
    display::display_item::DisplayItem,
    mentally_resident::present_mentally_resident_hope_selected_menu,
    menu::top_menu::present_top_menu,
    node::{hope_node::HopeNode, item_node::ItemNode, to_do_node::ToDoNode},
    surrealdb_layer::{surreal_tables::SurrealTables, DataLayerCommands},
    systems::bullet_list::BulletList,
};

use self::bullet_list_single_item::{
    present_bullet_list_item_selected, present_is_person_or_group_around_menu,
};

pub(crate) enum InquireBulletListItem<'e> {
    ViewFocusItems,
    Item {
        item_node: &'e ItemNode<'e>,
        parents: Vec<&'e Item<'e>>,
    },
    NextStepToDo {
        to_do: &'e ToDo<'e>,
        parents: Vec<&'e Item<'e>>,
    },
    NeedsNextStepHope {
        hope: &'e Hope<'e>,
        parents: Vec<&'e Item<'e>>,
    },
}

impl Display for InquireBulletListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ViewFocusItems => write!(f, "⏲️  [View Focus Items] ⏲️")?,
            Self::Item { item_node, parents } => {
                let display_item = DisplayItem::new(item_node.get_item());
                if item_node.is_person_or_group() {
                    write!(f, "Is {} around?", display_item)?;
                } else {
                    write!(f, "{} ", display_item)?;
                }
                for item in parents {
                    let display_item = DisplayItem::new(item);
                    write!(f, " ⬅ {}", display_item)?;
                }
            }
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
        item_nodes: &'a [ItemNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let mut list =
            Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len() + 1);
        list.push(Self::ViewFocusItems);
        Self::add_items_to_list(list, item_nodes, next_step_nodes, hopes_without_a_next_step)
    }

    pub(crate) fn create_list_just_items(
        item_nodes: &'a [ItemNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let list = Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len());
        Self::add_items_to_list(list, item_nodes, next_step_nodes, hopes_without_a_next_step)
    }

    fn add_items_to_list(
        mut list: Vec<InquireBulletListItem<'a>>,
        item_nodes: &'a [ItemNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        list.extend(item_nodes.iter().map(|x| InquireBulletListItem::Item {
            item_node: x,
            parents: x.create_next_step_parents(),
        }));
        list.extend(
            next_step_nodes
                .iter()
                .map(|x| InquireBulletListItem::NextStepToDo {
                    to_do: x.get_to_do(),
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
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let base_data = BaseData::new_from_surreal_tables(surreal_tables);
    let bullet_list = BulletList::new_unfocused_bullet_list(base_data);

    let (item_nodes, next_step_nodes, hope_nodes_needing_a_next_step) =
        bullet_list.get_bullet_list();

    let inquire_bullet_list = InquireBulletListItem::create_list_with_view_focus_items_option(
        item_nodes,
        next_step_nodes,
        hope_nodes_needing_a_next_step,
    );

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                present_focused_bullet_list_menu(send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::Item { item_node, parents }) => {
                if item_node.is_person_or_group() {
                    present_is_person_or_group_around_menu(item_node, send_to_data_storage_layer)
                        .await
                } else {
                    present_bullet_list_item_selected(
                        item_node.get_item(),
                        &parents,
                        bullet_list.get_active_items(),
                        send_to_data_storage_layer,
                    )
                    .await
                }
            }
            Ok(InquireBulletListItem::NextStepToDo { to_do, parents }) => {
                present_bullet_list_item_selected(
                    to_do.get_item(),
                    &parents,
                    bullet_list.get_active_items(),
                    send_to_data_storage_layer,
                )
                .await
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
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let base_data = BaseData::new_from_surreal_tables(surreal_tables);
    let bullet_list = BulletList::new_focused_bullet_list(base_data);

    let (item_nodes, next_step_nodes, hopes_without_a_next_step) = bullet_list.get_bullet_list();

    let inquire_bullet_list = InquireBulletListItem::create_list_just_items(
        item_nodes,
        next_step_nodes,
        hopes_without_a_next_step,
    );

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::Item { item_node, parents }) => {
                present_bullet_list_item_selected(
                    item_node.get_item(),
                    &parents,
                    bullet_list.get_active_items(),
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::NextStepToDo { to_do, parents }) => {
                present_bullet_list_item_selected(
                    to_do.get_item(),
                    &parents,
                    bullet_list.get_active_items(),
                    send_to_data_storage_layer,
                )
                .await
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

#[cfg(test)]
mod tests {}
