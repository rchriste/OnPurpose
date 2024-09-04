pub(crate) mod give_this_item_a_parent;
pub(crate) mod log_worked_on_this;
pub(crate) mod prompt_priority_for_new_item;
mod something_else_should_be_done_first;
mod starting_to_work_on_this_now;
pub(crate) mod state_a_smaller_next_step;
pub(crate) mod urgency_plan;

use std::fmt::Display;

use better_term::Style;
use chrono::{DateTime, Utc};
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;
use urgency_plan::present_set_ready_and_urgency_plan_menu;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    display::{
        display_item::DisplayItem, display_item_node::DisplayItemNode,
        display_item_type::DisplayItemType, display_urgency_plan::DisplayUrgency, DisplayStyle,
    },
    menu::inquire::{
        bullet_list_menu::{
            bullet_list_single_item::{
                give_this_item_a_parent::give_this_item_a_parent,
                something_else_should_be_done_first::something_else_should_be_done_first,
                starting_to_work_on_this_now::starting_to_work_on_this_now,
                state_a_smaller_next_step::state_a_smaller_next_step,
            },
            review_item,
        },
        select_higher_importance_than_this::select_higher_importance_than_this,
        top_menu::capture,
        unable_to_work_on_item_right_now::unable_to_work_on_item_right_now,
        update_item_summary::update_item_summary,
    },
    new_item,
    node::{
        item_node::{DependencyWithItem, ItemNode},
        item_status::ItemStatus,
        Filter,
    },
    surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_item::{
            Responsibility, SurrealHowMuchIsInMyControl, SurrealItemType, SurrealMotivationKind,
            SurrealUrgency,
        },
        surreal_tables::SurrealTables,
    },
    systems::bullet_list::BulletList,
};

pub(crate) enum LogTime {
    SeparateTaskLogTheTime,
    PartOfAnotherTaskDoNotLogTheTime,
}

enum BulletListSingleItemSelection<'e> {
    ChangeItemType {
        current: &'e SurrealItemType,
    },
    CaptureNewItem,
    StartingToWorkOnThisNow,
    GiveThisItemAParent,
    ChangeReadyAndUrgencyPlan,
    EstimateHowManyFocusPeriodsThisWillTake,
    UnableToDoThisRightNow,
    NotInTheMoodToDoThisRightNow,
    SomethingElseShouldBeDoneFirst,
    ReviewItem,
    StateASmallerNextStep,
    DefineMilestones, //For a hope
    UpdateMilestones, //For a hope
    WorkedOnThis,
    Finished,
    ThisIsARepeatingItem,
    CreateNotesForThisItem,
    LinkNotesForThisItem,
    OpenNotesForThisItem,
    OpenNotesForParentItem {
        item_in_chain_with_notes: DisplayItem<'e>,
    },
    DoWithSomethingElse,
    SearchForSimilarWork,
    ReturnToBulletList,
    ProcessAndFinish,
    UpdateSummary,
    SwitchToParentItem(DisplayItem<'e>, &'e ItemStatus<'e>),
    ParentToItem,
    RemoveParent(DisplayItem<'e>, &'e ItemStatus<'e>),
    SwitchToChildItem(DisplayItem<'e>, &'e ItemStatus<'e>),
    RemoveChild(DisplayItem<'e>, &'e ItemStatus<'e>),
    CaptureAFork,
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::CaptureNewItem => write!(f, "Capture New Item"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item, _) => {
                write!(f, "⇄ Switch to parent: {}", parent_item)
            }
            Self::StartingToWorkOnThisNow => write!(f, "I'm starting to work on this now"),
            Self::StateASmallerNextStep => {
                write!(f, "State a smaller next step")
            }
            Self::ReviewItem => write!(f, "Review Item: Update or Reorder Children or Parents"),
            Self::ParentToItem => {
                write!(f, "⭱ Parent to a new or existing Item")
            }
            Self::SwitchToChildItem(child_item, _) => {
                write!(f, "⇄ Switch to child: {}", child_item)
            }
            Self::RemoveChild(child_item, _) => write!(f, "🚫 Remove Child: {}", child_item),
            Self::RemoveParent(parent_item, _) => write!(f, "🚫 Remove Parent: {}", parent_item),
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
            Self::SomethingElseShouldBeDoneFirst => {
                write!(f, "Something else should be done first")
            }
            Self::ChangeItemType { current } => {
                let current_item_type = DisplayItemType::new(DisplayStyle::Full, current);
                write!(f, "Change Item Type (Currently: {})", current_item_type)
            }
            Self::GiveThisItemAParent => write!(f, "Give this item a Parent"),
            Self::EstimateHowManyFocusPeriodsThisWillTake => {
                write!(f, "Estimate how many Focus Periods this will take")
            }
            Self::UnableToDoThisRightNow => write!(f, "I am unable to do this right now"),
            Self::NotInTheMoodToDoThisRightNow => {
                write!(f, "I am not in the mood to do this right now")
            }
            Self::DefineMilestones => write!(f, "Define milestones"),
            Self::UpdateMilestones => write!(f, "Update milestones"),
            Self::WorkedOnThis => write!(f, "I worked on this"),
            Self::Finished => write!(f, "I finished"),
            Self::ThisIsARepeatingItem => {
                write!(f, "This is a repeating item I need to do periodically")
            }
            Self::CreateNotesForThisItem => write!(f, "Create a OneNote page for this"),
            Self::LinkNotesForThisItem => write!(f, "Provide a link to the notes for this"),
            Self::OpenNotesForThisItem => write!(f, "Open notes for this"),
            Self::OpenNotesForParentItem {
                item_in_chain_with_notes: parent,
            } => write!(f, "Open notes for parent item: {}", parent),
            Self::DoWithSomethingElse => {
                write!(f, "Do with something else")
            }
            Self::SearchForSimilarWork => write!(f, "Look for similar work to also do"),
            Self::ReturnToBulletList => write!(f, "Return to the Bullet List Menu"),
            Self::CaptureAFork => write!(f, "Capture a fork"),
            Self::ChangeReadyAndUrgencyPlan => write!(f, "Change Ready & Urgency Plan"),
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(
        item_node: &'e ItemNode<'e>,
        all_items_status: &'e [ItemStatus<'e>],
    ) -> Vec<Self> {
        let mut list = Vec::default();

        let has_no_parent = !item_node.has_parents(Filter::Active);
        let is_type_goal = item_node.is_type_goal();
        let is_type_motivation = item_node.is_type_motivation();
        let has_active_children = item_node.has_children(Filter::Active);

        if has_no_parent {
            list.push(Self::GiveThisItemAParent);
        }

        list.push(Self::CaptureNewItem);
        list.push(Self::StartingToWorkOnThisNow);
        list.push(Self::WorkedOnThis);

        list.push(Self::Finished);

        list.push(Self::UnableToDoThisRightNow);
        list.push(Self::NotInTheMoodToDoThisRightNow);

        list.push(Self::StateASmallerNextStep);

        list.push(Self::SomethingElseShouldBeDoneFirst);

        if !is_type_motivation {
            list.push(Self::EstimateHowManyFocusPeriodsThisWillTake);
            list.push(Self::DoWithSomethingElse);
            list.push(Self::SearchForSimilarWork);
        }

        if item_node.is_there_notes() {
            list.push(Self::OpenNotesForThisItem);
        } else {
            list.push(Self::CreateNotesForThisItem);
            list.push(Self::LinkNotesForThisItem);
        }

        list.push(Self::ReviewItem);

        if is_type_goal {
            if has_active_children {
                list.push(Self::UpdateMilestones);
            } else {
                list.push(Self::DefineMilestones);
            }
        }

        let parent_items = item_node
            .get_parents(Filter::Active)
            .map(|x| x.get_item())
            .collect::<Vec<_>>();
        list.push(Self::ParentToItem);
        list.extend(parent_items.iter().map(|x: &&'e Item<'e>| {
            let item_status = all_items_status
                .iter()
                .find(|y| y.get_item() == *x)
                .expect("All items are here");
            Self::SwitchToParentItem(DisplayItem::new(x), item_status)
        }));
        list.extend(parent_items.iter().map(|x: &&'e Item<'e>| {
            let item_status = all_items_status
                .iter()
                .find(|y| y.get_item() == *x)
                .expect("All items are here");
            Self::RemoveParent(DisplayItem::new(x), item_status)
        }));

        let parent_chain = item_node.create_parent_chain();
        for parent in parent_chain {
            if parent.is_there_notes() {
                list.push(Self::OpenNotesForParentItem {
                    item_in_chain_with_notes: DisplayItem::new(parent),
                });
            }
        }

        let child_items = item_node
            .get_children(Filter::Active)
            .map(|x| x.get_item())
            .collect::<Vec<_>>();
        list.extend(child_items.iter().map(|child: &&'e Item<'e>| {
            let child_item_status = all_items_status
                .iter()
                .find(|y| y.get_item() == *child)
                .expect("All items are here");
            Self::SwitchToChildItem(DisplayItem::new(child), child_item_status)
        }));

        list.extend(child_items.iter().map(|child: &&'e Item<'e>| {
            let child_item_status = all_items_status
                .iter()
                .find(|y| y.get_item() == *child)
                .expect("All items are here");
            Self::RemoveChild(DisplayItem::new(child), child_item_status)
        }));

        list.push(Self::CaptureAFork);
        list.push(Self::ThisIsARepeatingItem);
        list.push(Self::ChangeItemType {
            current: item_node.get_type(),
        });
        list.push(Self::ChangeReadyAndUrgencyPlan);

        list.extend(vec![
            Self::ProcessAndFinish,
            Self::UpdateSummary,
            Self::DebugPrintItem,
            Self::ReturnToBulletList,
        ]);

        list
    }
}

pub(crate) async fn present_bullet_list_item_selected(
    menu_for: &ItemStatus<'_>,
    when_selected: DateTime<Utc>, //Owns the value because you are meant to give the current time
    bullet_list: &BulletList,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    print_completed_children(menu_for);
    print_in_progress_children(menu_for, bullet_list.get_all_items_status());
    println!();

    let all_items_lap_highest_count = bullet_list.get_all_items_status();
    let list = BulletListSingleItemSelection::create_list(
        menu_for.get_item_node(),
        all_items_lap_highest_count,
    );

    let selection = Select::new("Select from the below list|", list)
        .with_page_size(14)
        .prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ChangeItemType { .. }) => {
            declare_item_type(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::StartingToWorkOnThisNow) => {
            starting_to_work_on_this_now(
                menu_for,
                &when_selected,
                bullet_list,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::CaptureNewItem) => {
            capture(send_to_data_storage_layer).await?;
            Box::pin(present_bullet_list_item_selected(
                menu_for,
                when_selected,
                bullet_list,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(BulletListSingleItemSelection::StateASmallerNextStep) => {
            state_a_smaller_next_step(menu_for.get_item_node(), send_to_data_storage_layer).await?;
            //TODO: Refresh menu_for and bullet_list
            let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                .await
                .unwrap();
            let now = Utc::now();
            let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
            let calculated_data = CalculatedData::new_from_base_data(base_data);
            let bullet_list = BulletList::new_bullet_list(calculated_data, &now);

            let menu_for = bullet_list
                .get_all_items_status()
                .iter()
                .find(|x| x.get_item() == menu_for.get_item())
                .expect("We will find this existing item once");

            Box::pin(present_bullet_list_item_selected(
                menu_for,
                when_selected,
                &bullet_list,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(BulletListSingleItemSelection::GiveThisItemAParent) => {
            give_this_item_a_parent(menu_for.get_item(), false, send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::RemoveParent(_, selected)) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemRemoveParent {
                    child: menu_for.get_item().get_surreal_record_id().clone(),
                    parent_to_remove: selected.get_item().get_surreal_record_id().clone(),
                })
                .await
                .unwrap();
            Ok(())
        }
        Ok(BulletListSingleItemSelection::RemoveChild(_, selected)) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemRemoveParent {
                    child: selected.get_item().get_surreal_record_id().clone(),
                    parent_to_remove: menu_for.get_item().get_surreal_record_id().clone(),
                })
                .await
                .unwrap();
            Ok(())
        }
        Ok(BulletListSingleItemSelection::EstimateHowManyFocusPeriodsThisWillTake) => {
            todo!("TODO: Implement EstimateHowManyFocusPeriodsThisWillTake");
        }
        Ok(BulletListSingleItemSelection::UnableToDoThisRightNow) => {
            unable_to_work_on_item_right_now(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::NotInTheMoodToDoThisRightNow) => {
            todo!("TODO: Implement NotInTheMoodToDoThisRightNow");
        }
        Ok(BulletListSingleItemSelection::SomethingElseShouldBeDoneFirst) => {
            something_else_should_be_done_first(menu_for.get_item(), send_to_data_storage_layer)
                .await
        }
        Ok(BulletListSingleItemSelection::ReviewItem) => {
            review_item::present_review_item_menu(
                menu_for,
                menu_for
                    .get_urgency_now()
                    .unwrap_or(&SurrealUrgency::InTheModeByImportance)
                    .clone(),
                bullet_list.get_all_items_status(),
                LogTime::PartOfAnotherTaskDoNotLogTheTime,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::DefineMilestones) => {
            todo!("TODO: Implement DefineMilestones");
        }
        Ok(BulletListSingleItemSelection::UpdateMilestones) => {
            todo!("TODO: Implement UpdateMilestones");
        }
        Ok(BulletListSingleItemSelection::WorkedOnThis) => {
            present_set_ready_and_urgency_plan_menu(
                menu_for,
                menu_for.get_urgency_now().cloned(),
                LogTime::PartOfAnotherTaskDoNotLogTheTime,
                send_to_data_storage_layer,
            )
            .await?;
            log_worked_on_this::log_worked_on_this(
                menu_for,
                &when_selected,
                Utc::now(),
                send_to_data_storage_layer,
            )
            .await?;
            prompt_priority_for_new_item::prompt_priority_for_new_item(
                menu_for,
                bullet_list,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::Finished) => {
            finish_bullet_item(
                menu_for,
                bullet_list,
                Utc::now(),
                send_to_data_storage_layer,
            )
            .await?;
            log_worked_on_this::log_worked_on_this(
                menu_for,
                &when_selected,
                Utc::now(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::ThisIsARepeatingItem) => {
            todo!("TODO: Implement ThisIsARepeatingItem");
        }
        Ok(BulletListSingleItemSelection::CreateNotesForThisItem) => {
            todo!("TODO: Implement CreateNotes");
        }
        Ok(BulletListSingleItemSelection::LinkNotesForThisItem) => {
            todo!("TODO: Implement LinkNotes");
        }
        Ok(BulletListSingleItemSelection::OpenNotesForThisItem) => {
            todo!("TODO: Implement OpenNotesForThisItem");
        }
        Ok(BulletListSingleItemSelection::OpenNotesForParentItem {
            item_in_chain_with_notes: _,
        }) => {
            todo!("TODO: Implement OpenNotesForParentItem");
        }
        Ok(BulletListSingleItemSelection::DoWithSomethingElse) => {
            todo!("TODO: Implement DoWithSomethingElse");
        }
        Ok(BulletListSingleItemSelection::SearchForSimilarWork) => {
            todo!("TODO: Implement SearchForSimilarWork");
        }
        Ok(BulletListSingleItemSelection::ReturnToBulletList) => {
            todo!("TODO: Implement ReturnToBulletList");
        }
        Ok(BulletListSingleItemSelection::CaptureAFork) => {
            todo!("TODO: Implement CaptureAFork");
        }
        Ok(BulletListSingleItemSelection::ChangeReadyAndUrgencyPlan) => {
            present_set_ready_and_urgency_plan_menu(
                menu_for,
                menu_for.get_urgency_now().cloned(),
                LogTime::PartOfAnotherTaskDoNotLogTheTime,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(menu_for.get_item(), send_to_data_storage_layer).await?;
            //After updating the summary we want to stay on the same item with the same times
            Box::pin(present_bullet_list_item_selected(
                menu_for,
                when_selected,
                bullet_list,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(BulletListSingleItemSelection::SwitchToParentItem(_, selected))
        | Ok(BulletListSingleItemSelection::SwitchToChildItem(_, selected)) => {
            Box::pin(present_bullet_list_item_selected(
                selected,
                chrono::Utc::now(),
                bullet_list,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(BulletListSingleItemSelection::ParentToItem) => {
            parent_to_item(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::DebugPrintItem) => {
            println!("{:?}", menu_for);
            Ok(())
        }
        Err(InquireError::OperationCanceled) => Ok(()), //Nothing to do we just want to return to the bullet list
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}

fn print_completed_children(menu_for: &ItemStatus<'_>) {
    let completed_children = menu_for
        .get_children(Filter::Finished)
        .map(|x| x.get_item())
        .collect::<Vec<_>>();
    if !completed_children.is_empty() {
        if completed_children.len() < 8 {
            println!("Completed Children:",);
            for child in completed_children {
                println!("  ✅{}", DisplayItem::new(child));
            }
        } else {
            println!("{} completed children ✅", completed_children.len());
        }
    }
}

fn print_in_progress_children(menu_for: &ItemStatus<'_>, all_item_status: &[ItemStatus<'_>]) {
    let in_progress_children = menu_for.get_children(Filter::Active).collect::<Vec<_>>();
    if !in_progress_children.is_empty() {
        let most_important = menu_for.recursive_get_most_important_and_ready(all_item_status);
        let most_important = if let Some(most_important) = most_important {
            most_important.get_self_and_parents_flattened(Filter::Active)
        } else {
            Default::default()
        };
        println!("Children:");
        for child in in_progress_children {
            print!("  ");
            if most_important.iter().any(|most_important| {
                most_important.get_surreal_record_id() == child.get_item().get_surreal_record_id()
            }) {
                print!("🔝");
            }
            let has_dependencies = child.get_dependencies(Filter::Active).any(|x| match x {
                //A child item being a dependency doesn't make sense to the user in this context
                DependencyWithItem::AfterChildItem(_) => false,
                _ => true,
            });
            if has_dependencies {
                print!("⌛");
            }
            let urgency_now = child
                .get_urgency_now()
                .map(|x| DisplayUrgency::new(x, DisplayStyle::Abbreviated));
            if let Some(urgency_now) = urgency_now {
                print!("{}", urgency_now);
            }
            println!("{}", DisplayItem::new(child.get_item()));
        }
    }
}

enum FinishSelection<'e> {
    CreateNextStepWithParent(&'e Item<'e>),
    GoToParent(&'e Item<'e>),
    CaptureNewItem,
    ReturnToBulletList,
}

impl Display for FinishSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinishSelection::CreateNextStepWithParent(parent) => write!(
                f,
                "Create Next Step with Parent: {}",
                DisplayItem::new(parent)
            ),
            FinishSelection::GoToParent(parent) => {
                write!(f, "Go to Parent: {}", DisplayItem::new(parent))
            }
            FinishSelection::CaptureNewItem => write!(f, "Capture New Item"),
            FinishSelection::ReturnToBulletList => write!(f, "Return to Bullet List"),
        }
    }
}

impl<'e> FinishSelection<'e> {
    fn make_list(parents: &[&'e Item<'e>]) -> Vec<Self> {
        let mut list = Vec::default();
        list.push(Self::ReturnToBulletList);
        list.push(Self::CaptureNewItem);
        list.extend(
            parents
                .iter()
                .flat_map(|x| vec![Self::CreateNextStepWithParent(x), Self::GoToParent(x)]),
        );
        list
    }
}

async fn finish_bullet_item(
    finish_this: &ItemStatus<'_>,
    bullet_list: &BulletList,
    now: DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let when_this_function_was_called = now;
    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem {
            item: finish_this.get_surreal_record_id().clone(),
            when_finished: now.into(),
        })
        .await
        .unwrap();

    let list = FinishSelection::make_list(
        &finish_this
            .get_parents(Filter::Active)
            .map(|x| x.get_item())
            .collect::<Vec<_>>(),
    );
    let selection = Select::new("Select from the below list|", list).prompt();

    match selection {
        Ok(FinishSelection::CaptureNewItem) => capture(send_to_data_storage_layer).await,
        Ok(FinishSelection::CreateNextStepWithParent(parent)) => {
            let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                .await
                .unwrap();
            let now = Utc::now();
            let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
            let items = base_data.get_items();
            let active_items = base_data.get_active_items();
            let parent_surreal_record_id = parent.get_surreal_record_id();
            let time_spent_log = base_data.get_time_spent_log();
            let updated_parent = active_items
                .iter()
                .filter(|x| x.get_surreal_record_id() == parent_surreal_record_id)
                .map(|x| ItemNode::new(x, items, time_spent_log))
                .next()
                .expect("We will find this existing item once");

            state_a_smaller_next_step(&updated_parent, send_to_data_storage_layer).await?;

            //Recursively call as a way of creating a loop, we don't want to return to the main bullet list
            Box::pin(finish_bullet_item(
                finish_this,
                bullet_list,
                Utc::now(),
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(FinishSelection::GoToParent(parent)) => {
            let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                .await
                .unwrap();
            let now = Utc::now();
            let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
            let calculated_data = CalculatedData::new_from_base_data(base_data);
            let parent_surreal_record_id = parent.get_surreal_record_id();
            let updated_parent = calculated_data
                .get_items_status()
                .iter()
                .find(|x| x.get_surreal_record_id() == parent_surreal_record_id)
                .expect("We will find this existing item once");

            Box::pin(present_bullet_list_item_selected(
                updated_parent,
                when_this_function_was_called,
                bullet_list,
                send_to_data_storage_layer,
            ))
            .await
        }
        Ok(FinishSelection::ReturnToBulletList) => Ok(()),
        Err(InquireError::OperationCanceled) => {
            todo!("This should undo the finish and put the item back to what it was before")
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn process_and_finish_bullet_item(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    //I should probably be processing and finishing all of the children next steps but this requires some thought
    //because sometimes or if there are multiple children next steps that that shouldn't happen rather the user
    //should be prompted to pick which children to also process and finish.
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = item.get_surreal_record_id();
    if !user_processed_text.is_empty() {
        send_to_data_storage_layer
            .send(DataLayerCommands::AddProcessedText(
                user_processed_text,
                surreal_item.clone(),
            ))
            .await
            .unwrap();
    }

    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem {
            item: surreal_item.clone(),
            when_finished: Utc::now().into(),
        })
        .await
        .unwrap();

    Ok(())
}

async fn parent_to_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let raw_data = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(raw_data, now);
    let items = base_data.get_items();
    let active_items = base_data.get_active_items();
    let time_spent_log = base_data.get_time_spent_log();
    let item_nodes = active_items
        .iter()
        .map(|x| ItemNode::new(x, items, time_spent_log))
        .collect::<Vec<_>>();
    let list = DisplayItemNode::make_list(&item_nodes);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item_node: &ItemNode = display_item.get_item_node();
            let higher_importance_than_this = if item_node.has_children(Filter::Active) {
                let items = item_node
                    .get_children(Filter::Active)
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_importance_than_this(&items, None)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_record_id().clone(),
                    parent: item_node.get_surreal_record_id().clone(),
                    higher_importance_than_this,
                })
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            parent_to_new_item(parent_this, send_to_data_storage_layer).await
        }
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) enum ItemTypeSelection {
    Action,
    Goal,
    ResponsiveGoal,
    MotivationCore,
    MotivationNonCore,
    ResponsiveMotivationCore,
    ResponsiveMotivationNonCore,
    NormalHelp,
    ResponsiveHelp,
}

impl Display for ItemTypeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action => write!(f, "Action 🪜"),
            Self::Goal => write!(f, "Multi-Step Goal 🪧"),
            Self::ResponsiveGoal => write!(f, "Responsive Multi-Step Goal 🪧"),
            Self::MotivationCore => {
                write!(f, "Motivational Core Reason 🎯🏢")
            }
            Self::MotivationNonCore => {
                write!(f, "Motivational Non-Core Reason 🎯🏞")
            }
            Self::ResponsiveMotivationCore => {
                write!(f, "Responsive Motivational Core Reason 🎯🏢")
            }
            Self::ResponsiveMotivationNonCore => {
                write!(f, "Responsive Motivational Non-Core Reason 🎯🏞")
            }
            Self::NormalHelp | Self::ResponsiveHelp => write!(f, "Help"),
        }
    }
}

impl ItemTypeSelection {
    pub(crate) fn create_list() -> Vec<Self> {
        vec![
            Self::Action,
            Self::Goal,
            Self::MotivationCore,
            Self::MotivationNonCore,
            Self::NormalHelp,
        ]
    }

    pub(crate) fn create_list_goals_and_motivations() -> Vec<Self> {
        vec![
            Self::Goal,
            Self::MotivationCore,
            Self::MotivationNonCore,
            Self::ResponsiveGoal,
            Self::ResponsiveMotivationCore,
            Self::ResponsiveMotivationNonCore,
            Self::ResponsiveHelp,
        ]
    }

    pub(crate) fn create_new_item_prompt_user_for_summary(&self) -> new_item::NewItem {
        let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
        self.create_new_item(summary)
    }

    pub(crate) fn create_new_item(&self, summary: String) -> new_item::NewItem {
        let mut new_item_builder = new_item::NewItemBuilder::default();
        let new_item_builder = new_item_builder.summary(summary);
        let new_item_builder = match self {
            ItemTypeSelection::Action => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Action),
            ItemTypeSelection::Goal => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default())),
            ItemTypeSelection::ResponsiveGoal => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default())),
            ItemTypeSelection::MotivationCore => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Motivation(SurrealMotivationKind::CoreWork)),
            ItemTypeSelection::MotivationNonCore => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Motivation(
                    SurrealMotivationKind::NonCoreWork,
                )),
            ItemTypeSelection::ResponsiveMotivationCore => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(SurrealItemType::Motivation(SurrealMotivationKind::CoreWork)),
            ItemTypeSelection::ResponsiveMotivationNonCore => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(SurrealItemType::Motivation(
                    SurrealMotivationKind::NonCoreWork,
                )),
            ItemTypeSelection::NormalHelp => {
                panic!("NormalHelp should be handled before this point")
            }
            ItemTypeSelection::ResponsiveHelp => {
                panic!("ResponsiveHelp should be handled before this point")
            }
        };
        new_item_builder
            .build()
            .expect("Filled out required fields")
    }

    pub(crate) fn print_normal_help() {
        println!("{}Action{}", Style::default().bold(), Style::default());
        println!("A thing to do and an action or step to take.");
        println!(
            "{}The emoji is a ladder 🪜 with steps.{}",
            Style::default().italic(),
            Style::default()
        );
        println!();
        println!(
            "{}Multi-Step Goal{}",
            Style::default().bold(),
            Style::default()
        );
        println!("A milestone or hopeful outcome that should be broken down to smaller steps to accomplish.");
        println!(
            "{}The emoji is a Milestone sign 🪧 or goal post.{}",
            Style::default().italic(),
            Style::default()
        );
        println!();
        println!(
            "{}Motivational Reason{}",
            Style::default().bold(),
            Style::default()
        );
        println!(
            "For stating that the item captured is a reason for doing something. Because there is"
        );
        println!("almost always a diverse number of benefits to doing something the word motivational is");
        println!("also used. The test to know if a reason is motivational is to ask the question if this");
        println!("was not true would that significantly change the priority or cancel the work.");
        println!(
            "{}Emoji is a target 🎯 that provides something to aim for.{}",
            Style::default().italic(),
            Style::default()
        );
        println!();
    }

    pub(crate) fn print_responsive_help() {
        println!(
            "The word responsive means do {}not{} prompt for a next step but do be searchable so",
            Style::default().bold(),
            Style::default()
        );
        println!(
            "work can be parented to this. {}Responsive{} should be used when the work to do is or",
            Style::default().bold(),
            Style::default()
        );
        println!(
            "will be in response to something that has or might come up. A {}Responsive Goal or ",
            Style::default().bold()
        );
        println!("Motivation{} does not need the user to define a next step. Rather this is considered as", Style::default());
        println!(
            "a scenario that if it occurs will require your time to address and take care of but"
        );
        println!("otherwise there is nothing to do. This is supportive work.");
        println!();
    }
}

pub(crate) async fn parent_to_new_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(parent_to_new_item(parent_this, send_to_data_storage_layer)).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            Box::pin(parent_to_new_item(parent_this, send_to_data_storage_layer)).await
        }
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentNewItemWithAnExistingChildItem {
                    child: parent_this.get_surreal_record_id().clone(),
                    parent_new_item: new_item,
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

pub(crate) async fn declare_item_type(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::Action) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    SurrealItemType::Action,
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::Goal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::ResponsiveGoal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    SurrealItemType::Goal(SurrealHowMuchIsInMyControl::default()),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::MotivationCore) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    SurrealItemType::Motivation(SurrealMotivationKind::CoreWork),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::MotivationNonCore) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    SurrealItemType::Motivation(SurrealMotivationKind::NonCoreWork),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::ResponsiveMotivationCore) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    SurrealItemType::Motivation(SurrealMotivationKind::CoreWork),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::ResponsiveMotivationNonCore) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    SurrealItemType::Motivation(SurrealMotivationKind::NonCoreWork),
                ))
                .await
                .unwrap();
            Ok(())
        }
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(declare_item_type(item, send_to_data_storage_layer)).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            Box::pin(declare_item_type(item, send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}

enum IsAPersonOrGroupAroundSelection {
    Yes,
    No,
}

impl Display for IsAPersonOrGroupAroundSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsAPersonOrGroupAroundSelection::Yes => write!(f, "Yes"),
            IsAPersonOrGroupAroundSelection::No => write!(f, "No"),
        }
    }
}

impl IsAPersonOrGroupAroundSelection {
    fn create_list() -> Vec<Self> {
        vec![Self::Yes, Self::No]
    }
}

pub(crate) async fn present_is_person_or_group_around_menu(
    person_or_group_node: &ItemNode<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let list = IsAPersonOrGroupAroundSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(IsAPersonOrGroupAroundSelection::Yes) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::FinishItem {
                    item: person_or_group_node.get_surreal_record_id().clone(),
                    when_finished: Utc::now().into(),
                })
                .await
                .unwrap();
            Ok(())
        }
        Ok(IsAPersonOrGroupAroundSelection::No) => todo!(),
        Err(InquireError::OperationCanceled) => todo!(),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected {}", err),
    }
}
