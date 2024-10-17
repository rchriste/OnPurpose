use std::fmt::{self, Display, Formatter};

use ahash::HashMap;
use chrono::Utc;
use inquire::Select;
use itertools::Itertools;
use surrealdb::{opt::RecordId, sql::Datetime};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_in_the_moment_priority::SurrealAction,
        surreal_item::SurrealUrgency, surreal_tables::SurrealTables,
    },
    display::{
        display_dependencies_with_item_node::DisplayDependenciesWithItemNode,
        display_item::DisplayItem, display_item_status::DisplayItemStatus,
        display_urgency_plan::DisplayUrgencyPlan,
    },
    menu::inquire::{
        do_now_list_menu::do_now_list_single_item::{
            give_this_item_a_parent::give_this_item_a_parent,
            state_a_smaller_action::state_a_smaller_action,
            urgency_plan::{prompt_for_dependencies, prompt_for_urgency_plan, AddOrRemove},
        },
        select_higher_importance_than_this::select_higher_importance_than_this,
    },
    new_time_spent::NewTimeSpent,
    node::{
        item_node::ItemNode,
        item_status::{DependencyWithItemNode, ItemStatus},
        Filter,
    },
};

use super::do_now_list_single_item::LogTime;

enum ReviewItemMenuChoices<'e> {
    DoneWithReview,
    UpdateRelativeImportanceDontShowSingleParent { parent: &'e Item<'e> },
    UpdateRelativeImportanceShowParent { parent: &'e Item<'e> },
    UpdateDependencies { current_item: &'e ItemStatus<'e> },
    UpdateUrgencyPlan { current_item: &'e ItemStatus<'e> },
    FinishThisItem,
    AddNewParent,
    AddNewChild,
    GoToParent(&'e Item<'e>),
    RemoveParent(&'e Item<'e>),
    GoToChild(&'e Item<'e>),
    RemoveChild(&'e Item<'e>),
}

impl Display for ReviewItemMenuChoices<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReviewItemMenuChoices::DoneWithReview => write!(f, "Done with this review"),
            ReviewItemMenuChoices::UpdateRelativeImportanceDontShowSingleParent { .. } => {
                write!(f, "Update relative importance of this item")
            }
            ReviewItemMenuChoices::UpdateRelativeImportanceShowParent { parent } => {
                let display_parent = DisplayItem::new(parent);
                write!(
                    f,
                    "Update relative importance of this item, for parent: {}",
                    display_parent
                )
            }
            ReviewItemMenuChoices::UpdateDependencies { current_item } => {
                let dependencies = current_item
                    .get_dependencies(Filter::Active)
                    .filter(|x| match x {
                        DependencyWithItemNode::AfterDateTime { .. }
                        | DependencyWithItemNode::UntilScheduled { .. }
                        | DependencyWithItemNode::AfterItem(_)
                        | DependencyWithItemNode::DuringItem(_) => true,
                        DependencyWithItemNode::AfterChildItem(_) => false,
                    })
                    .collect::<Vec<_>>();
                let display_ready = DisplayDependenciesWithItemNode::new(&dependencies);
                write!(
                    f,
                    "Update dependencies this item is waiting on, current setting is children plus: {}",
                    display_ready
                )
            }
            ReviewItemMenuChoices::UpdateUrgencyPlan { current_item } => {
                let display_urgency = DisplayUrgencyPlan::new(current_item.get_urgency_plan());
                write!(f, "Update urgency, current setting: {}", display_urgency)
            }
            ReviewItemMenuChoices::FinishThisItem => write!(f, "Finish this item"),
            ReviewItemMenuChoices::AddNewParent => write!(f, "Add new parent"),
            ReviewItemMenuChoices::AddNewChild => write!(f, "Add new child"),
            ReviewItemMenuChoices::GoToParent(item) => {
                let display_item = DisplayItem::new(item);
                write!(f, "Go to parent: {}", display_item)
            }
            ReviewItemMenuChoices::RemoveParent(item) => {
                let display_item = DisplayItem::new(item);
                write!(f, "Remove parent: {}", display_item)
            }
            ReviewItemMenuChoices::GoToChild(item) => {
                let display_item = DisplayItem::new(item);
                write!(f, "Go to child: {}", display_item)
            }
            ReviewItemMenuChoices::RemoveChild(item) => {
                let display_item = DisplayItem::new(item);
                write!(f, "Remove child: {}", display_item)
            }
        }
    }
}

impl ReviewItemMenuChoices<'_> {
    pub(crate) fn make_list<'e>(
        current_item: &'e ItemStatus<'e>,
    ) -> Vec<ReviewItemMenuChoices<'e>> {
        let mut list = vec![ReviewItemMenuChoices::DoneWithReview];

        if current_item
            .get_item_node()
            .get_parents(Filter::Active)
            .count()
            == 1
        {
            let parent = current_item
                .get_item_node()
                .get_parents(Filter::Active)
                .next()
                .expect("Item is for sure there because count is 1")
                .get_item();
            list.push(
                ReviewItemMenuChoices::UpdateRelativeImportanceDontShowSingleParent { parent },
            );
        } else {
            //Note that if there is no parent then we don't show this option and that is by design
            for parent in current_item.get_item_node().get_parents(Filter::Active) {
                list.push(ReviewItemMenuChoices::UpdateRelativeImportanceShowParent {
                    parent: parent.get_item(),
                });
            }
        }

        list.push(ReviewItemMenuChoices::UpdateUrgencyPlan { current_item });
        list.push(ReviewItemMenuChoices::UpdateDependencies { current_item });
        list.push(ReviewItemMenuChoices::FinishThisItem);
        list.push(ReviewItemMenuChoices::AddNewParent);

        for parent in current_item.get_item_node().get_parents(Filter::Active) {
            list.push(ReviewItemMenuChoices::GoToParent(parent.get_item()));
        }

        for parent in current_item.get_item_node().get_parents(Filter::Active) {
            list.push(ReviewItemMenuChoices::RemoveParent(parent.get_item()));
        }

        list.push(ReviewItemMenuChoices::AddNewChild);

        for child in current_item.get_item_node().get_children(Filter::Active) {
            list.push(ReviewItemMenuChoices::GoToChild(child.get_item()));
        }

        for child in current_item.get_item_node().get_children(Filter::Active) {
            list.push(ReviewItemMenuChoices::RemoveChild(child.get_item()));
        }

        list
    }
}

pub(crate) async fn present_review_item_menu(
    item_status: &ItemStatus<'_>,
    current_urgency: SurrealUrgency,
    all_items: &HashMap<RecordId, ItemStatus<'_>>,
    log_time: LogTime,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let start_review_item_menu = Utc::now();

    present_review_item_menu_internal(
        item_status,
        item_status,
        all_items,
        send_to_data_storage_layer,
    )
    .await?;

    match log_time {
        LogTime::SeparateTaskLogTheTime => {
            let new_time_spent = NewTimeSpent {
                working_on: vec![SurrealAction::ReviewItem(
                    item_status.get_surreal_record_id().clone(),
                )], //TODO: I should also add all the parent items that this is making progress towards the goal
                when_started: start_review_item_menu,
                when_stopped: Utc::now(),
                dedication: None,
                urgency: Some(current_urgency),
            };

            send_to_data_storage_layer
                .send(DataLayerCommands::RecordTimeSpent(new_time_spent))
                .await
                .unwrap();
        }
        LogTime::PartOfAnotherTaskDoNotLogTheTime => {
            //Do nothing
        }
    }

    Ok(())
}

async fn refresh_items_present_review_item_menu_internal(
    item_under_review: &ItemStatus<'_>,
    selected_item: &ItemStatus<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let updated_calculated_data = CalculatedData::new_from_base_data(base_data);

    let updated_all_items = updated_calculated_data.get_items_status();
    let updated_item_under_review = updated_all_items
        .get(item_under_review.get_item().get_surreal_record_id())
        .expect("Item under review must be in the list of all items");
    let updated_selected_item = updated_all_items
        .get(selected_item.get_item().get_surreal_record_id())
        .expect("Selected item must be in the list of all items");

    Box::pin(present_review_item_menu_internal(
        updated_item_under_review,
        updated_selected_item,
        updated_all_items,
        send_to_data_storage_layer,
    ))
    .await
}

async fn present_review_item_menu_internal<'a>(
    item_under_review: &ItemStatus<'a>,
    selected_item: &ItemStatus<'a>,
    all_items: &'a HashMap<RecordId, ItemStatus<'a>>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let choices = ReviewItemMenuChoices::make_list(selected_item);
    let selected = Select::new("What would you like to do with this item?", choices)
        .prompt()
        .unwrap();

    match selected {
        ReviewItemMenuChoices::DoneWithReview => {
            let now = Utc::now();
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateItemLastReviewedDate(
                    item_under_review.get_surreal_record_id().clone(),
                    now.into(),
                ))
                .await
                .unwrap();

            Ok(())
        }
        ReviewItemMenuChoices::UpdateRelativeImportanceDontShowSingleParent { parent }
        | ReviewItemMenuChoices::UpdateRelativeImportanceShowParent { parent } => {
            let parent = all_items
                .get(parent.get_surreal_record_id())
                .expect("Parent must be in the list of all items")
                .get_item_node();
            update_relative_importance(
                parent,
                selected_item.get_item(),
                send_to_data_storage_layer,
            )
            .await
            .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::UpdateDependencies { current_item } => {
            assert_eq!(
                current_item.get_item(),
                selected_item.get_item(),
                "current_item exists twice so it can be used by display trait"
            );
            let dependencies =
                prompt_for_dependencies(Some(selected_item), send_to_data_storage_layer)
                    .await
                    .unwrap();
            for (command, dependency) in dependencies.into_iter() {
                match command {
                    AddOrRemove::Add => {
                        send_to_data_storage_layer
                            .send(DataLayerCommands::AddItemDependency(
                                selected_item.get_surreal_record_id().clone(),
                                dependency,
                            ))
                            .await
                            .unwrap();
                    }
                    AddOrRemove::Remove => {
                        send_to_data_storage_layer
                            .send(DataLayerCommands::RemoveItemDependency(
                                selected_item.get_surreal_record_id().clone(),
                                dependency,
                            ))
                            .await
                            .unwrap();
                    }
                }
            }

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::UpdateUrgencyPlan { current_item } => {
            let now = Utc::now();
            let urgency_plan = prompt_for_urgency_plan(&now, send_to_data_storage_layer).await;
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateUrgencyPlan(
                    current_item.get_surreal_record_id().clone(),
                    Some(urgency_plan),
                ))
                .await
                .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::FinishThisItem => {
            let when_finished: Datetime = (Utc::now()).into();
            send_to_data_storage_layer
                .send(DataLayerCommands::FinishItem {
                    item: selected_item.get_surreal_record_id().clone(),
                    when_finished,
                })
                .await
                .unwrap();

            if selected_item.get_item() == item_under_review.get_item() {
                Ok(())
            } else {
                println!("Item finished, going back to the item under review");
                let display_item = DisplayItemStatus::new(item_under_review);
                println!("{}", display_item);
                refresh_items_present_review_item_menu_internal(
                    item_under_review,
                    item_under_review,
                    send_to_data_storage_layer,
                )
                .await
            }
        }
        ReviewItemMenuChoices::AddNewParent => {
            give_this_item_a_parent(selected_item.get_item(), false, send_to_data_storage_layer)
                .await
                .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::AddNewChild => {
            state_a_smaller_action(selected_item.get_item_node(), send_to_data_storage_layer)
                .await
                .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::GoToParent(item) => {
            let parent = all_items
                .get(item.get_surreal_record_id())
                .expect("Parent must be in the list of all items");
            Box::pin(present_review_item_menu_internal(
                item_under_review,
                parent,
                all_items,
                send_to_data_storage_layer,
            ))
            .await
        }
        ReviewItemMenuChoices::RemoveParent(item) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemRemoveParent {
                    child: selected_item.get_surreal_record_id().clone(),
                    parent_to_remove: item.get_surreal_record_id().clone(),
                })
                .await
                .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
        ReviewItemMenuChoices::GoToChild(item) => {
            let child = all_items
                .get(item.get_surreal_record_id())
                .expect("Child must be in the list of all items");
            Box::pin(present_review_item_menu_internal(
                item_under_review,
                child,
                all_items,
                send_to_data_storage_layer,
            ))
            .await
        }
        ReviewItemMenuChoices::RemoveChild(item) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemRemoveParent {
                    child: item.get_surreal_record_id().clone(),
                    parent_to_remove: selected_item.get_surreal_record_id().clone(),
                })
                .await
                .unwrap();

            refresh_items_present_review_item_menu_internal(
                item_under_review,
                selected_item,
                send_to_data_storage_layer,
            )
            .await
        }
    }
}

pub(crate) async fn update_relative_importance(
    parent: &ItemNode<'_>,
    item_to_move: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let (current_position, ..) = parent
        .get_children(Filter::Active)
        .find_position(|x| x.get_item() == item_to_move)
        .expect(
            "item to move must already be in the list so it must already be a child of the parent",
        );
    let priority_list = parent
        .get_children(Filter::Active)
        .map(|x| x.get_item())
        .filter(|x| *x != item_to_move)
        .collect::<Vec<_>>();
    let higher_than = select_higher_importance_than_this(&priority_list, Some(current_position));
    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateRelativeImportance {
            parent: parent.get_surreal_record_id().clone(),
            update_this_child: item_to_move.get_surreal_record_id().clone(),
            higher_importance_than_this_child: higher_than,
        })
        .await
        .unwrap();

    Ok(())
}
