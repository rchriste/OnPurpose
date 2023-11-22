mod define_child_goal;
pub(crate) mod parent_to_a_goal_or_motivation;
pub(crate) mod set_staging;
mod something_else_should_be_done_first;
mod starting_to_work_on_this_now;
mod state_a_smaller_next_step;

use std::fmt::Display;

use async_recursion::async_recursion;
use better_term::Style;
use chrono::{DateTime, Utc};
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        covering::Covering, covering_until_date_time::CoveringUntilDateTime, item::Item, BaseData,
    },
    display::{display_item::DisplayItem, display_item_node::DisplayItemNode},
    menu::{
        bullet_list_menu::bullet_list_single_item::{
            define_child_goal::define_child_goals,
            parent_to_a_goal_or_motivation::parent_to_a_goal_or_motivation,
            something_else_should_be_done_first::something_else_should_be_done_first,
            starting_to_work_on_this_now::starting_to_work_on_this_now,
            state_a_smaller_next_step::state_a_smaller_next_step,
        },
        select_higher_priority_than_this::select_higher_priority_than_this,
        staging_query::mentally_resident_query,
        unable_to_work_on_item_right_now::unable_to_work_on_item_right_now,
    },
    new_item,
    node::item_node::ItemNode,
    surrealdb_layer::{
        surreal_item::{HowMuchIsInMyControl, ItemType, Responsibility},
        surreal_tables::SurrealTables,
        DataLayerCommands,
    },
    update_item_summary,
};

use self::{
    parent_to_a_goal_or_motivation::parent_to_a_motivation, set_staging::present_set_staging_menu,
};

enum BulletListSingleItemSelection<'e> {
    ICannotDoThisSimpleThingRightNowRemindMeLater,
    DeclareItemType,
    StateASmallerNextStep,
    StartingToWorkOnThisNow,
    ParentToAGoalOrMotivation,
    ParentToAMotivation,
    PlanWhenToDoThis,
    ChangeStaging,
    EstimateHowManyFocusPeriodsThisWillTake,
    UnableToDoThisRightNow,
    NotInTheMoodToDoThisRightNow,
    SomethingElseShouldBeDoneFirst,
    DefineChildActions,
    UpdateChildActions,
    DefineChildGoals, //For a motivation
    UpdateChildGoals, //For a motivation
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
    WaitUntilSimilarWorkIsDone,
    SearchForSimilarWork,
    ChangeType,
    ReturnToBulletList,
    ProcessAndFinish,
    UpdateSummary,
    SwitchToParentItem(DisplayItem<'e>, ItemNode<'e>),
    ParentToItem,
    CaptureAFork,
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item, _) => write!(f, "Switch to: {}", parent_item),
            Self::StartingToWorkOnThisNow => write!(f, "I'm starting to work on this now"),
            Self::StateASmallerNextStep => {
                write!(f, "State a smaller next step")
            }
            Self::ParentToItem => {
                write!(f, "⭱ Parent to a new or existing Item")
            }
            Self::PlanWhenToDoThis => {
                write!(f, "Plan when to do this")
            }
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
            Self::ICannotDoThisSimpleThingRightNowRemindMeLater => {
                write!(f, "I cannot do this right now, remind me later")
            }
            Self::SomethingElseShouldBeDoneFirst => {
                write!(f, "Something else should be done first")
            }
            Self::DeclareItemType => write!(f, "Declare Item Type"),
            Self::ParentToAGoalOrMotivation => write!(f, "Parent this to a Goal or Motivation"),
            Self::ParentToAMotivation => write!(f, "Parent this to a Motivation"),
            Self::EstimateHowManyFocusPeriodsThisWillTake => {
                write!(f, "Estimate how many Focus Periods this will take")
            }
            Self::UnableToDoThisRightNow => write!(f, "I am unable to do this right now"),
            Self::NotInTheMoodToDoThisRightNow => {
                write!(f, "I am not in the mood to do this right now")
            }
            Self::DefineChildActions => write!(f, "Define actions to take"),
            Self::UpdateChildActions => write!(f, "Update actions to take"),
            Self::DefineChildGoals => write!(f, "Define goals to have"),
            Self::UpdateChildGoals => write!(f, "Update goals to have"),
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
            Self::WaitUntilSimilarWorkIsDone => {
                write!(f, "Wait to do this until similar work is done")
            }
            Self::SearchForSimilarWork => write!(f, "Look for similar work to also do"),
            Self::ReturnToBulletList => write!(f, "Return to the Bullet List Menu"),
            Self::CaptureAFork => write!(f, "Capture a fork"),
            Self::ChangeType => write!(f, "Change Type"),
            Self::ChangeStaging => write!(f, "Change Staging"),
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(
        item_node: &'e ItemNode<'e>,
        all_coverings: &'e [Covering<'e>],
        all_items: &'e [&Item<'e>],
        all_snoozed: &'e [&'e CoveringUntilDateTime<'e>],
    ) -> Vec<Self> {
        let mut list = Vec::default();

        let is_type_action = item_node.is_type_action();
        let has_no_parent = item_node.get_larger().is_empty();
        let is_type_goal = item_node.is_type_goal();
        let is_type_motivation = item_node.is_type_motivation();
        let is_type_undeclared = item_node.is_type_undeclared();
        let has_active_children = item_node.has_active_children();

        if (is_type_action) && has_no_parent {
            list.push(Self::ParentToAGoalOrMotivation);
        } else if is_type_goal && !has_active_children {
            list.push(Self::StateASmallerNextStep);
        }

        if is_type_goal && has_no_parent {
            list.push(Self::ParentToAMotivation);
        }

        if !is_type_goal && !is_type_motivation && !is_type_undeclared {
            list.push(Self::PlanWhenToDoThis);
        }

        if is_type_undeclared {
            list.push(Self::DeclareItemType);
        }

        if is_type_action || is_type_goal || is_type_motivation {
            list.push(Self::StartingToWorkOnThisNow);
            list.push(Self::WorkedOnThis);
        }

        list.push(Self::Finished);

        if is_type_action {
            list.push(Self::UnableToDoThisRightNow);
            list.push(Self::NotInTheMoodToDoThisRightNow);
        }

        if is_type_action {
            list.push(Self::StateASmallerNextStep);
        }

        if !is_type_undeclared {
            list.push(Self::SomethingElseShouldBeDoneFirst);
        }

        if is_type_action || is_type_goal {
            list.push(Self::EstimateHowManyFocusPeriodsThisWillTake)
        }

        if is_type_action || is_type_goal {
            list.push(Self::WaitUntilSimilarWorkIsDone);
            list.push(Self::SearchForSimilarWork);
        }

        if is_type_action || is_type_goal || is_type_motivation {
            if item_node.is_there_notes() {
                list.push(Self::OpenNotesForThisItem);
            } else {
                list.push(Self::CreateNotesForThisItem);
                list.push(Self::LinkNotesForThisItem);
            }
        }

        if item_node.is_type_simple() {
            list.push(Self::ICannotDoThisSimpleThingRightNowRemindMeLater);
        }

        if is_type_action || is_type_goal || is_type_motivation {
            if has_active_children {
                list.push(Self::UpdateChildActions);
            } else {
                list.push(Self::DefineChildActions);
            }
        }

        if is_type_motivation {
            if has_active_children {
                list.push(Self::UpdateChildGoals);
            } else {
                list.push(Self::DefineChildGoals);
            }
        }

        if is_type_goal {
            if has_active_children {
                list.push(Self::UpdateMilestones);
            } else {
                list.push(Self::DefineMilestones);
            }
        }

        let parent_items = item_node.create_parent_chain();
        if is_type_action || is_type_goal || is_type_motivation || is_type_undeclared {
            if has_no_parent {
                list.push(Self::ParentToItem);
            } else {
                list.extend(parent_items.iter().map(|x: &&'e Item<'e>| {
                    let item_node = ItemNode::new(x, all_coverings, all_snoozed, all_items);
                    Self::SwitchToParentItem(DisplayItem::new(x), item_node)
                }));
            }
        }

        for parent in parent_items {
            if parent.is_there_notes() {
                list.push(Self::OpenNotesForParentItem {
                    item_in_chain_with_notes: DisplayItem::new(parent),
                });
            }
        }

        if is_type_action || is_type_goal || is_type_motivation {
            list.push(Self::CaptureAFork);
        }

        if is_type_action || is_type_goal {
            list.push(Self::ThisIsARepeatingItem);
        }

        if is_type_action || is_type_goal || is_type_motivation {
            list.push(Self::ChangeType);
            list.push(Self::ChangeStaging);
        }

        let is_type_simple = item_node.is_type_simple();
        if !is_type_simple && !is_type_undeclared {
            list.extend(vec![
                Self::ProcessAndFinish,
                Self::UpdateSummary,
                Self::DebugPrintItem,
                Self::ReturnToBulletList,
            ]);
        }

        list
    }
}

#[async_recursion]
pub(crate) async fn present_bullet_list_item_selected(
    menu_for: &ItemNode<'_>,
    current_date_time: &DateTime<Utc>,
    all_coverings: &[Covering<'_>],
    all_snoozed: &[&CoveringUntilDateTime<'_>],
    all_items: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list =
        BulletListSingleItemSelection::create_list(menu_for, all_coverings, all_items, all_snoozed);

    let selection = Select::new("Select from the below list|", list)
        .with_page_size(14)
        .prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ICannotDoThisSimpleThingRightNowRemindMeLater) => {
            todo!("TODO: Implement ICannotDoThisSimpleThingRightNowRemindMeLater");
        }
        Ok(BulletListSingleItemSelection::DeclareItemType) => {
            declare_item_type(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::StartingToWorkOnThisNow) => {
            starting_to_work_on_this_now(
                menu_for,
                current_date_time,
                all_coverings,
                all_snoozed,
                all_items,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::StateASmallerNextStep) => {
            state_a_smaller_next_step(menu_for, send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::ParentToAGoalOrMotivation) => {
            parent_to_a_goal_or_motivation(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::PlanWhenToDoThis) => {
            todo!("TODO: Implement PlanWhenToDoThis");
        }
        Ok(BulletListSingleItemSelection::ParentToAMotivation) => {
            parent_to_a_motivation(menu_for.get_item(), send_to_data_storage_layer).await
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
        Ok(BulletListSingleItemSelection::DefineChildActions) => {
            todo!("TODO: Implement DefineChildActions");
        }
        Ok(BulletListSingleItemSelection::UpdateChildActions) => {
            todo!("TODO: Implement UpdateChildActions");
        }
        Ok(BulletListSingleItemSelection::DefineChildGoals) => {
            define_child_goals(menu_for, send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::UpdateChildGoals) => {
            todo!("TODO: Implement UpdateChildGoals");
        }
        Ok(BulletListSingleItemSelection::DefineMilestones) => {
            todo!("TODO: Implement DefineMilestones");
        }
        Ok(BulletListSingleItemSelection::UpdateMilestones) => {
            todo!("TODO: Implement UpdateMilestones");
        }
        Ok(BulletListSingleItemSelection::WorkedOnThis) => {
            let new_mentally_resident_staging = mentally_resident_query().await.unwrap();
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateItemStaging(
                    menu_for.get_surreal_record_id().clone(),
                    new_mentally_resident_staging,
                ))
                .await
                .unwrap();
        }
        Ok(BulletListSingleItemSelection::Finished) => {
            finish_bullet_item(
                menu_for,
                all_coverings,
                all_snoozed,
                all_items,
                current_date_time,
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
        Ok(BulletListSingleItemSelection::WaitUntilSimilarWorkIsDone) => {
            todo!("TODO: Implement WaitUntilSimilarWorkIsDone");
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
        Ok(BulletListSingleItemSelection::ChangeType) => {
            declare_item_type(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::ChangeStaging) => {
            present_set_staging_menu(menu_for, send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(
                menu_for.get_surreal_record_id().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::SwitchToParentItem(_, selected)) => {
            present_bullet_list_item_parent_selected(
                &selected,
                current_date_time,
                all_coverings,
                all_snoozed,
                all_items,
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::ParentToItem) => {
            parent_to_item(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::DebugPrintItem) => {
            println!("{:?}", menu_for);
        }
        Err(InquireError::OperationCanceled) => (), //Nothing to do we just want to return to the bullet list
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn finish_bullet_item(
    finish_this: &ItemNode<'_>,
    all_coverings: &[Covering<'_>],
    all_snoozed: &[&CoveringUntilDateTime<'_>],
    all_items: &[&Item<'_>],
    current_date_time: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    send_to_data_storage_layer
        .send(DataLayerCommands::FinishItem(
            finish_this.get_surreal_record_id().clone(),
        ))
        .await
        .unwrap();

    let mut parents_iter = finish_this.get_larger().iter();
    let next_item = parents_iter.next();
    if let Some(next_item) = next_item {
        let next_item = ItemNode::new(next_item.get_item(), all_coverings, all_snoozed, all_items);
        let display_item = DisplayItemNode::new(&next_item, Some(current_date_time));
        println!("{}", display_item);
        present_bullet_list_item_selected(
            &next_item,
            current_date_time,
            all_coverings,
            all_snoozed,
            all_items,
            send_to_data_storage_layer,
        )
        .await
    }
}

async fn process_and_finish_bullet_item(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
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
        .send(DataLayerCommands::FinishItem(surreal_item.clone()))
        .await
        .unwrap();
}

async fn present_bullet_list_item_parent_selected(
    selected_item: &ItemNode<'_>,
    current_date_time: &DateTime<Utc>,
    all_coverings: &[Covering<'_>],
    all_snoozed: &[&CoveringUntilDateTime<'_>],
    all_items: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    match selected_item.get_type() {
        ItemType::Action | ItemType::Goal(..) | ItemType::Motivation => {
            present_bullet_list_item_selected(
                selected_item,
                current_date_time,
                all_coverings,
                all_snoozed,
                all_items,
                send_to_data_storage_layer,
            )
            .await
        }
        ItemType::IdeaOrThought => todo!(),
        ItemType::Undeclared => todo!(),
        ItemType::Simple => todo!(),
        ItemType::PersonOrGroup => todo!(),
    }
}

async fn parent_to_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let raw_data = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(raw_data, now);
    let items = base_data.get_active_items();
    let item_nodes = items
        .iter()
        .map(|x| {
            ItemNode::new(
                x,
                base_data.get_coverings(),
                base_data.get_active_snoozed(),
                items,
            )
        })
        .collect::<Vec<_>>();
    let list = DisplayItemNode::make_list(&item_nodes, None);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item_node: &ItemNode = display_item.get_item_node();
            let higher_priority_than_this = if item_node.has_active_children() {
                let items = item_node
                    .get_smaller()
                    .iter()
                    .map(|x| x.get_item())
                    .collect::<Vec<_>>();
                select_higher_priority_than_this(&items)
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_record_id().clone(),
                    parent: item_node.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            parent_to_new_item(parent_this, send_to_data_storage_layer).await
        }
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) async fn cover_with_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    //TODO: cover_to_item and parent_to_item are the same except for the command sent to the data storage layer, refactor to reduce duplicated code
    let raw_data = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(raw_data, now);
    let items = base_data.get_active_items();

    let list = DisplayItem::make_list(items);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item: &Item = display_item.into();
            let higher_priority_than_this = if item.has_children() {
                todo!("User needs to pick what item this should be before. Although if all of the children are finished then it should be fine to just put it at the end. Also there is probably common menu code to call for this purpose")
            } else {
                None
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: item.get_surreal_record_id().clone(),
                    parent: parent_this.get_surreal_record_id().clone(),
                    higher_priority_than_this,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled | InquireError::InvalidConfiguration(_)) => {
            cover_with_new_item(parent_this, send_to_data_storage_layer).await
        }
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) enum ItemTypeSelection {
    Action,
    Goal,
    ResponsiveGoal,
    Motivation,
    ResponsiveMotivation,
    NormalHelp,
    ResponsiveHelp,
}

impl Display for ItemTypeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action => write!(f, "Action 🪜"),
            Self::Goal => write!(f, "Multi-Step Goal 🪧"),
            Self::ResponsiveGoal => write!(f, "Responsive Multi-Step Goal 🪧"),
            Self::Motivation => {
                write!(f, "Motivational Reason 🎯")
            }
            Self::ResponsiveMotivation => {
                write!(f, "Responsive Motivational Reason 🎯")
            }
            Self::NormalHelp | Self::ResponsiveHelp => write!(f, "Help"),
        }
    }
}

impl ItemTypeSelection {
    pub(crate) fn create_list() -> Vec<Self> {
        vec![Self::Action, Self::Goal, Self::Motivation, Self::NormalHelp]
    }

    pub(crate) fn create_list_just_goals() -> Vec<Self> {
        vec![Self::Goal, Self::ResponsiveGoal, Self::ResponsiveHelp]
    }

    pub(crate) fn create_list_goals_and_motivations() -> Vec<Self> {
        vec![
            Self::Goal,
            Self::Motivation,
            Self::ResponsiveGoal,
            Self::ResponsiveMotivation,
            Self::ResponsiveHelp,
        ]
    }

    pub(crate) fn create_list_just_motivations() -> Vec<Self> {
        vec![
            Self::Motivation,
            Self::ResponsiveMotivation,
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
                .item_type(ItemType::Action),
            ItemTypeSelection::Goal => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(ItemType::Goal(HowMuchIsInMyControl::default())),
            ItemTypeSelection::ResponsiveGoal => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(ItemType::Goal(HowMuchIsInMyControl::default())),
            ItemTypeSelection::Motivation => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(ItemType::Motivation),
            ItemTypeSelection::ResponsiveMotivation => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(ItemType::Motivation),
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

#[async_recursion]
pub(crate) async fn parent_to_new_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            parent_to_new_item(parent_this, send_to_data_storage_layer).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            parent_to_new_item(parent_this, send_to_data_storage_layer).await
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
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}

#[async_recursion]
pub(crate) async fn cover_with_new_item(
    cover_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            cover_with_new_item(cover_this, send_to_data_storage_layer).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            cover_with_new_item(cover_this, send_to_data_storage_layer).await
        }
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemWithANewItem {
                    cover_this: cover_this.get_surreal_record_id().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}

#[async_recursion]
pub(crate) async fn declare_item_type(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(ItemTypeSelection::Action) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::Action,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::Goal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::Goal(HowMuchIsInMyControl::default()),
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ResponsiveGoal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    ItemType::Goal(HowMuchIsInMyControl::default()),
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::Motivation) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::Motivation,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ResponsiveMotivation) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_record_id().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    ItemType::Motivation,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            declare_item_type(item, send_to_data_storage_layer).await
        }
        Ok(ItemTypeSelection::ResponsiveHelp) => {
            ItemTypeSelection::print_responsive_help();
            declare_item_type(item, send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => todo!(),
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
) {
    let list = IsAPersonOrGroupAroundSelection::create_list();

    let selection = Select::new("Select from the below list|", list).prompt();
    match selection {
        Ok(IsAPersonOrGroupAroundSelection::Yes) => send_to_data_storage_layer
            .send(DataLayerCommands::FinishItem(
                person_or_group_node.get_surreal_record_id().clone(),
            ))
            .await
            .unwrap(),
        Ok(IsAPersonOrGroupAroundSelection::No) => todo!(),
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}
