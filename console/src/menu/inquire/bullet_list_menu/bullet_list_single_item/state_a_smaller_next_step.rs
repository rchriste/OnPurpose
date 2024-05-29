use std::cmp::Ordering;

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    display::{display_item::DisplayItem, display_item_node::DisplayItemNode},
    menu::inquire::{
        bullet_list_menu::bullet_list_single_item::set_staging::{
            present_set_staging_menu, StagingMenuSelection,
        },
        select_higher_priority_than_this::select_higher_priority_than_this,
        staging_query::{mentally_resident_query, on_deck_query},
    },
    node::{item_highest_lap_count::ItemHighestLapCount, item_node::ItemNode, Filter},
    surrealdb_layer::{
        surreal_item::SurrealStaging, surreal_tables::SurrealTables, DataLayerCommands,
    },
};

use super::ItemTypeSelection;

pub(crate) enum SelectAnItemSortingOrder {
    MotivationsFirst,
    NewestFirst,
}

pub(crate) async fn select_an_item<'a>(
    dont_show_these_items: Vec<&Item<'_>>,
    sorting_order: SelectAnItemSortingOrder,
    calculated_data: &'a CalculatedData,
) -> Result<Option<&'a ItemHighestLapCount<'a>>, ()> {
    let active_items = calculated_data
        .get_items_highest_lap_count()
        .iter()
        .filter(|x| !dont_show_these_items.iter().any(|y| x.get_item() == *y) && !x.is_finished())
        .collect::<Vec<_>>();
    let mut list = active_items
        .iter()
        .map(|x| DisplayItemNode::new(x.get_item_node()))
        .collect::<Vec<_>>();
    match sorting_order {
        SelectAnItemSortingOrder::MotivationsFirst => list.sort_by(|a, b| {
            if a.is_type_motivation() {
                if b.is_type_motivation() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else if a.is_type_goal() {
                if b.is_type_motivation() {
                    Ordering::Greater
                } else if b.is_type_goal() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else if b.is_type_motivation() || b.is_type_goal() {
                Ordering::Greater
            } else if a.get_type() == b.get_type() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
            .then_with(|| a.get_created().cmp(b.get_created()).reverse())
        }),
        SelectAnItemSortingOrder::NewestFirst => {
            list.sort_by(|a, b| a.get_created().cmp(b.get_created()).reverse())
        }
    }

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(selected_item) => Ok(active_items
            .into_iter()
            .find(|x| x.get_item() == selected_item.get_item())),
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => Ok(None),
        Err(err) => todo!("{:?}", err),
    }
}

pub(crate) async fn state_a_smaller_next_step(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
    let selection = select_an_item(
        vec![selected_item.get_item()],
        SelectAnItemSortingOrder::NewestFirst,
        &calculated_data,
    )
    .await;

    match selection {
        Ok(Some(child)) => {
            let parent = selected_item;
            let child: &Item<'_> = child.get_item();

            let higher_priority_than_this = if parent.has_children(Filter::Active) {
                let items = parent
                    .get_smaller(Filter::Active)
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: child.get_surreal_record_id().clone(),
                    parent: parent.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();

            println!(
                "Please update Staging for {}",
                DisplayItem::new(parent.get_item())
            );
            let default_selection = match parent.get_staging() {
                SurrealStaging::NotSet => Some(StagingMenuSelection::NotSet),
                SurrealStaging::MentallyResident { .. } => {
                    Some(StagingMenuSelection::MentallyResident)
                }
                SurrealStaging::OnDeck { .. } => Some(StagingMenuSelection::OnDeck),
                SurrealStaging::Planned => Some(StagingMenuSelection::Planned),
                SurrealStaging::ThinkingAbout => Some(StagingMenuSelection::ThinkingAbout),
                SurrealStaging::Released => Some(StagingMenuSelection::Released),
                SurrealStaging::InRelationTo { .. } => Some(StagingMenuSelection::InRelationTo),
            };
            present_set_staging_menu(
                parent.get_item(),
                send_to_data_storage_layer,
                default_selection,
            )
            .await
        }
        Ok(None) => {
            state_a_smaller_next_step_new_item(selected_item, send_to_data_storage_layer).await
        }
        Err(()) => Err(()),
    }
}

pub(crate) async fn state_a_smaller_next_step_new_item(
    selected_item: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(state_a_smaller_next_step_new_item(
                selected_item,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            Box::pin(state_a_smaller_next_step_new_item(
                selected_item,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(item_type_selection) => {
            let mut new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            let higher_priority_than_this = if selected_item.has_children(Filter::Active) {
                let items = selected_item
                    .get_smaller(Filter::Active)
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            let parent = selected_item;

            let (list, starting_cursor) =
                StagingMenuSelection::make_list(Some(StagingMenuSelection::NotSet), None);

            let selection = Select::new("Select from the below list|", list)
                .with_starting_cursor(starting_cursor)
                .prompt()
                .unwrap();
            new_item.staging = match selection {
                StagingMenuSelection::InRelationTo => {
                    todo!("In relation to")
                }
                StagingMenuSelection::NotSet => SurrealStaging::NotSet,
                StagingMenuSelection::MentallyResident => {
                    let result = mentally_resident_query().await;
                    match result {
                        Ok(mentally_resident) => mentally_resident,
                        Err(InquireError::OperationCanceled) => {
                            todo!("I probably need to refactor this into a function so I can use recursion here")
                        }
                        Err(InquireError::OperationInterrupted) => return Err(()),
                        Err(err) => todo!("{:?}", err),
                    }
                }
                StagingMenuSelection::OnDeck => {
                    let result = on_deck_query().await;
                    match result {
                        Ok(staging) => staging,
                        Err(InquireError::OperationCanceled) => {
                            todo!("I probably need to refactor this into a function so I can use recursion here")
                        }
                        Err(InquireError::OperationInterrupted) => return Err(()),
                        Err(err) => todo!("{:?}", err),
                    }
                }
                StagingMenuSelection::Planned => SurrealStaging::Planned,
                StagingMenuSelection::ThinkingAbout => SurrealStaging::ThinkingAbout,
                StagingMenuSelection::Released => SurrealStaging::Released,
                StagingMenuSelection::MakeItemReactive => {
                    todo!("I need to modify the return type to account for this different choice and pass up that information")
                }
                StagingMenuSelection::KeepAsIs(staging) => staging.clone(),
            };

            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewChildItem {
                    child: new_item,
                    parent: parent.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}
