use std::iter::once;

use chrono::Utc;
use inquire::{formatter::MultiOptionFormatter, MultiSelect};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    display::{
        display_action_with_item_status::DisplayActionWithItemStatus,
        display_item_status::DisplayItemStatus,
    },
    menu::inquire::bullet_list_menu::bullet_list_single_item::urgency_plan::prompt_for_triggers,
    node::{action_with_item_status::ActionWithItemStatus, item_status::ItemStatus, Filter},
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_in_the_moment_priority::{SurrealAction, SurrealPriorityKind},
        surreal_tables::SurrealTables,
    },
    systems::bullet_list::BulletList,
};

pub(crate) async fn prompt_priority_for_new_item(
    selected: &ItemStatus<'_>,
    old_bullet_list: &BulletList,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    if !selected.has_dependencies(Filter::Active) {
        let old_selected_root_parents = selected
            .get_root_parents(Filter::Active, old_bullet_list.get_all_items_status())
            .map(|x| {
                old_bullet_list
                    .get_all_items_status()
                    .iter()
                    .find(|y| y.get_surreal_record_id() == x.get_surreal_record_id())
                    .expect("This is a list of all items")
            });

        let old_roots_with_selected_as_most_important = old_selected_root_parents
            .filter(|root| {
                let most_important = root
                    .recursive_get_most_important_and_ready(old_bullet_list.get_all_items_status());
                most_important.map_or(false, |most_important: &ItemStatus| {
                    most_important.get_surreal_record_id() == selected.get_surreal_record_id()
                })
            })
            .collect::<Vec<_>>();

        if !old_roots_with_selected_as_most_important.is_empty() {
            let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                .await
                .unwrap();

            let now = Utc::now();
            let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
            let calculated_data = CalculatedData::new_from_base_data(base_data);
            let new_bullet_list = BulletList::new_bullet_list(calculated_data, &now);
            let updated_selected = new_bullet_list
                .get_all_items_status()
                .iter()
                .find(|x| *x == selected)
                .expect("Item exists in full list");

            let updated_selected_root_parents = updated_selected
                .get_root_parents(Filter::Active, new_bullet_list.get_all_items_status())
                .map(|x| {
                    new_bullet_list
                        .get_all_items_status()
                        .iter()
                        .find(|y| y.get_surreal_record_id() == x.get_surreal_record_id())
                        .expect("This is a list of all items")
                });

            let updated_roots_with_selected_as_most_important = updated_selected_root_parents
                .filter(|root| {
                    let most_important = root.recursive_get_most_important_and_ready(
                        new_bullet_list.get_all_items_status(),
                    );
                    most_important.map_or(false, |most_important: &ItemStatus| {
                        most_important.get_surreal_record_id()
                            == updated_selected.get_surreal_record_id()
                    })
                });

            let new_items = updated_roots_with_selected_as_most_important.filter(|root| {
                !old_roots_with_selected_as_most_important
                    .iter()
                    .any(|old_root| {
                        old_root.get_surreal_record_id() == root.get_surreal_record_id()
                    })
            });

            println!("Previous item: {}", DisplayItemStatus::new(selected));
            for new_item in new_items {
                let other_items = new_bullet_list
                    .get_ordered_bullet_list()
                    .iter()
                    .filter(|x| {
                        let same_urgency = new_item.get_urgency_now() == Some(&x.get_urgency_now());
                        let different_item =
                            new_item.get_surreal_record_id() != x.get_surreal_record_id();
                        same_urgency && different_item
                    })
                    .flat_map(|x| -> Box<dyn Iterator<Item = ActionWithItemStatus<'_>>> {
                        if let ActionWithItemStatus::PickWhatShouldBeDoneFirst(choices) = x {
                            Box::new(choices.clone().into_iter().filter(|y| {
                                y.get_surreal_record_id() != new_item.get_surreal_record_id()
                            }))
                        } else {
                            Box::new(once(x.clone()))
                        }
                    })
                    .collect::<Vec<ActionWithItemStatus>>();

                let display = other_items
                    .iter()
                    .map(DisplayActionWithItemStatus::new)
                    .collect::<Vec<_>>();
                let formatter: MultiOptionFormatter<'_, DisplayActionWithItemStatus> =
                    &|a| format!("{} items selected", a.len());
                if !other_items.is_empty() {
                    println!(
                        "Item is entering the list: {}",
                        DisplayItemStatus::new(new_item)
                    );
                    let selected = MultiSelect::new(
                        "Select the items that should be done first",
                        display.clone(),
                    )
                    .with_formatter(formatter)
                    .prompt()
                    .unwrap();

                    if !selected.is_empty() {
                        let in_effect_until =
                            prompt_for_triggers(&now, send_to_data_storage_layer).await;
                        send_to_data_storage_layer
                            .send(DataLayerCommands::DeclareInTheMomentPriority {
                                choice: SurrealAction::MakeProgress(
                                    new_item.get_surreal_record_id().clone(),
                                ),
                                kind: SurrealPriorityKind::LowestPriority,
                                not_chosen: selected
                                    .into_iter()
                                    .map(|x| x.clone_to_surreal_action())
                                    .collect(),
                                in_effect_until,
                            })
                            .await
                            .unwrap();
                    }

                    let selected = MultiSelect::new(
                        "Select the items that should wait until this item is done",
                        display,
                    )
                    .with_formatter(formatter)
                    .prompt()
                    .unwrap();
                    if !selected.is_empty() {
                        let in_effect_until =
                            prompt_for_triggers(&now, send_to_data_storage_layer).await;
                        send_to_data_storage_layer
                            .send(DataLayerCommands::DeclareInTheMomentPriority {
                                choice: SurrealAction::MakeProgress(
                                    new_item.get_surreal_record_id().clone(),
                                ),
                                kind: SurrealPriorityKind::HighestPriority,
                                not_chosen: selected
                                    .into_iter()
                                    .map(|x| x.clone_to_surreal_action())
                                    .collect(),
                                in_effect_until,
                            })
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}
