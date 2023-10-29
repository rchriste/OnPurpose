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
        person_or_group::PersonOrGroup,
        simple::{self, Simple},
        to_do::ToDo,
        undeclared::Undeclared,
    },
    display::display_item::DisplayItem,
    mentally_resident::{
        create_hope_nodes, present_mentally_resident_hope_selected_menu, HopeNode,
    },
    menu::top_menu::present_top_menu,
    node::{
        person_or_group_node::{create_person_or_group_nodes, PersonOrGroupNode},
        to_do_node::{create_to_do_nodes, ToDoNode},
    },
    surrealdb_layer::DataLayerCommands,
};

use self::bullet_list_single_item::{
    present_bullet_list_item_selected, present_is_person_or_group_around_menu,
};

pub(crate) enum InquireBulletListItem<'e> {
    ViewFocusItems,
    NextStepToDo {
        to_do: &'e ToDo<'e>,
        parents: Vec<&'e Item<'e>>,
    },
    NeedsNextStepHope {
        hope: &'e Hope<'e>,
        parents: Vec<&'e Item<'e>>,
    },
    IsAPersonOrGroupAround(&'e PersonOrGroupNode<'e>),
    NewlyCapturedItem(&'e Undeclared<'e>),
    SimpleItem(&'e simple::Simple<'e>),
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
            Self::IsAPersonOrGroupAround(person_or_group_node) => {
                let display_item =
                    DisplayItem::new(person_or_group_node.person_or_group().get_item());
                write!(f, "Is {} around?", display_item)?;
                for item in person_or_group_node.create_parent_chain() {
                    let display_item = DisplayItem::new(item);
                    write!(f, " ⬅ {}", display_item)?;
                }
            }
            Self::NewlyCapturedItem(undeclared) => {
                let display_item = DisplayItem::new(undeclared.get_item());
                write!(f, "{}", display_item)?;
            }
            Self::SimpleItem(simple) => {
                let display_item = DisplayItem::new(simple.get_item());
                write!(f, "{}", display_item)?;
            }
        }
        Ok(())
    }
}

impl<'a> InquireBulletListItem<'a> {
    pub(crate) fn create_list_with_view_focus_items_option(
        undeclared_items: &'a [Undeclared<'a>],
        simple_items: &'a [Simple<'a>],
        person_or_group_nodes: &'a [PersonOrGroupNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let mut list =
            Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len() + 1);
        list.push(Self::ViewFocusItems);
        Self::add_items_to_list(
            list,
            undeclared_items,
            simple_items,
            person_or_group_nodes,
            next_step_nodes,
            hopes_without_a_next_step,
        )
    }

    pub(crate) fn create_list_just_items(
        undeclared_items: &'a [Undeclared<'a>],
        simple_items: &'a [Simple<'a>],
        person_or_group_nodes: &'a [PersonOrGroupNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        let list = Vec::with_capacity(next_step_nodes.len() + hopes_without_a_next_step.len());
        Self::add_items_to_list(
            list,
            undeclared_items,
            simple_items,
            person_or_group_nodes,
            next_step_nodes,
            hopes_without_a_next_step,
        )
    }

    fn add_items_to_list(
        mut list: Vec<InquireBulletListItem<'a>>,
        undeclared_items: &'a [Undeclared<'a>],
        simple_items: &'a [Simple<'a>],
        person_or_group_nodes: &'a [PersonOrGroupNode<'a>],
        next_step_nodes: &'a [ToDoNode<'a>],
        hopes_without_a_next_step: &'a [HopeNode<'a>],
    ) -> Vec<InquireBulletListItem<'a>> {
        list.extend(
            undeclared_items
                .iter()
                .map(InquireBulletListItem::NewlyCapturedItem),
        );
        list.extend(simple_items.iter().map(InquireBulletListItem::SimpleItem));
        list.extend(
            person_or_group_nodes
                .iter()
                .map(InquireBulletListItem::IsAPersonOrGroupAround),
        );
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
    let persons_or_groups = items.filter_just_persons_or_groups();
    let person_or_groups_that_cover_an_item: Vec<PersonOrGroup> = persons_or_groups
        .into_iter()
        .filter(|x| x.is_covering_another_item(&coverings))
        .collect();
    let person_or_group_nodes_that_cover_an_item = create_person_or_group_nodes(
        &person_or_groups_that_cover_an_item,
        &coverings,
        &coverings_until_date_time,
        &active_items,
        &current_date_time,
        false,
    );

    let undeclared_items = items.filter_just_undeclared_items();

    let simple_items = items.filter_just_simple_items();

    let inquire_bullet_list = InquireBulletListItem::create_list_with_view_focus_items_option(
        &undeclared_items,
        &simple_items,
        &person_or_group_nodes_that_cover_an_item,
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
                present_bullet_list_item_selected(
                    to_do.get_item(),
                    &parents,
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::NeedsNextStepHope { hope, parents: _ }) => {
                present_mentally_resident_hope_selected_menu(hope, send_to_data_storage_layer).await
            }
            Ok(InquireBulletListItem::IsAPersonOrGroupAround(person_or_group_node)) => {
                present_is_person_or_group_around_menu(
                    person_or_group_node,
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::NewlyCapturedItem(undeclared)) => {
                present_bullet_list_item_selected(
                    undeclared.get_item(),
                    &[],
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::SimpleItem(_)) => {
                todo!("Implement this for a simple item")
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
    let persons_or_groups_that_cover_an_item = vec![]; //Sync'ing up with someone cannot be a focus item
    let undeclared_items = vec![]; //Newly captured items cannot be a focus item
    let simple_items = vec![]; //Simple items cannot be a focus item

    let inquire_bullet_list = InquireBulletListItem::create_list_just_items(
        &undeclared_items,
        &simple_items,
        &persons_or_groups_that_cover_an_item,
        &next_step_nodes,
        &hopes_without_a_next_step,
    );

    if !inquire_bullet_list.is_empty() {
        let selected = Select::new("", inquire_bullet_list)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(InquireBulletListItem::ViewFocusItems) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::NextStepToDo { to_do, parents }) => {
                present_bullet_list_item_selected(
                    to_do.get_item(),
                    &parents,
                    send_to_data_storage_layer,
                )
                .await
            }
            Ok(InquireBulletListItem::IsAPersonOrGroupAround(_)) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::NewlyCapturedItem(_)) => {
                panic!("The focus list should not present this option")
            }
            Ok(InquireBulletListItem::SimpleItem(_)) => {
                panic!("The focus list should not present this option")
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
