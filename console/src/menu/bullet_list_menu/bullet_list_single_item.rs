mod cover_bullet_item;
mod parent_to_a_goal;
mod something_else_should_be_done_first;
mod state_a_smaller_next_step;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{item::Item, BaseData},
    display::{display_item::DisplayItem, display_item_node::DisplayItemNode},
    menu::{
        bullet_list_menu::bullet_list_single_item::{
            cover_bullet_item::cover_bullet_item, parent_to_a_goal::parent_to_a_goal,
            something_else_should_be_done_first::something_else_should_be_done_first,
            state_a_smaller_next_step::state_a_smaller_next_step,
        },
        unable_to_work_on_item_right_now::unable_to_work_on_item_right_now,
    },
    new_item,
    node::item_node::ItemNode,
    surrealdb_layer::{
        surreal_item::{ItemType, Responsibility},
        surreal_tables::SurrealTables,
        DataLayerCommands,
    },
    update_item_summary, UnexpectedNextMenuAction,
};

use self::parent_to_a_goal::parent_to_a_motivation;

enum BulletListSingleItemSelection<'e> {
    ASimpleThingICanDoRightNow,
    ASimpleThingButICanDoItWhenever,
    ICannotDoThisSimpleThingRightNowRemindMeLater,
    DeclareItemType,
    StateASmallerNextStep,
    ParentToAGoal,
    ParentToAMotivation,
    PlanWhenToDoThis,
    DoInAFocusPeriod,
    EstimateHowManyFocusPeriodsThisWillTake,
    UnableToDoThisRightNow,
    NotInTheMoodToDoThisRightNow,
    SomethingElseShouldBeDoneFirst,
    DefineChildActions,
    UpdateChildActions,
    DefineChildHopes, //For a motivation
    UpdateChildHopes, //For a motivation
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
    Cover,
    UpdateSummary,
    //Take a DisplayItem rather than a reference because it is felt that this type is only created
    //for this scenario rather than kept around.
    SwitchToParentItem(DisplayItem<'e>, &'e ItemNode<'e>),
    ParentToItem,
    CaptureAFork,
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::Cover => write!(f, "Cover ⼍"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item, _) => write!(f, "Switch to: {}", parent_item),
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
            Self::ASimpleThingICanDoRightNow => {
                write!(f, "This is a simple thing I can do right now")
            }
            Self::ASimpleThingButICanDoItWhenever => {
                write!(f, "This is a simple thing but I can do it whenever")
            }
            Self::ICannotDoThisSimpleThingRightNowRemindMeLater => {
                write!(f, "I cannot do this right now, remind me later")
            }
            Self::SomethingElseShouldBeDoneFirst => {
                write!(f, "Something else should be done first")
            }
            Self::DeclareItemType => write!(f, "Declare Item Type"),
            Self::ParentToAGoal => write!(f, "Parent this to a Goal"),
            Self::ParentToAMotivation => write!(f, "Parent this to a Motivation"),
            Self::DoInAFocusPeriod => write!(f, "This should be done in a Focus Period"),
            Self::EstimateHowManyFocusPeriodsThisWillTake => {
                write!(f, "Estimate how many Focus Periods this will take")
            }
            Self::UnableToDoThisRightNow => write!(f, "I am unable to do this right now"),
            Self::NotInTheMoodToDoThisRightNow => {
                write!(f, "I am not in the mood to do this right now")
            }
            Self::DefineChildActions => write!(f, "Define actions to take"),
            Self::UpdateChildActions => write!(f, "Update actions to take"),
            Self::DefineChildHopes => write!(f, "Define hopes to have"),
            Self::UpdateChildHopes => write!(f, "Update hopes to have"),
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
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(item_node: &'e ItemNode<'e>, all_nodes: &'e [ItemNode<'e>]) -> Vec<Self> {
        let mut list = Vec::default();

        let is_type_action = item_node.is_type_action();
        let has_no_parent = item_node.get_larger().is_empty();
        let is_type_goal = item_node.is_type_goal();
        let is_type_motivation = item_node.is_type_motivation();
        let is_type_undeclared = item_node.is_type_undeclared();

        if is_type_action && has_no_parent {
            list.push(Self::ParentToAGoal);
        }

        if is_type_goal && has_no_parent {
            list.push(Self::ParentToAMotivation);
        }

        if !is_type_goal && !is_type_motivation {
            list.push(Self::PlanWhenToDoThis);
        }

        if is_type_undeclared {
            list.push(Self::ASimpleThingICanDoRightNow);
            list.push(Self::ASimpleThingButICanDoItWhenever);
            list.push(Self::DeclareItemType);
        }

        if is_type_action || is_type_goal || is_type_motivation {
            list.push(Self::WorkedOnThis);
        }

        list.push(Self::Finished);

        if is_type_action {
            list.push(Self::UnableToDoThisRightNow);
            list.push(Self::NotInTheMoodToDoThisRightNow);
        }

        let has_active_children = item_node.has_active_children();
        if is_type_action || is_type_goal && !has_active_children {
            list.push(Self::StateASmallerNextStep);
        }

        if !is_type_undeclared {
            list.push(Self::SomethingElseShouldBeDoneFirst);
        }

        let is_circumstance_focus_time = item_node.is_circumstance_focus_time();
        if is_type_action {
            if !is_circumstance_focus_time {
                list.push(Self::DoInAFocusPeriod);
            } else if item_node.get_estimated_focus_periods().is_none() {
                list.push(Self::EstimateHowManyFocusPeriodsThisWillTake)
            }
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
                list.push(Self::UpdateChildHopes);
            } else {
                list.push(Self::DefineChildHopes);
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
        if is_type_action || is_type_goal || is_type_motivation {
            if has_no_parent {
                list.push(Self::ParentToItem);
            } else {
                list.extend(parent_items.iter().map(|x: &&'e Item<'e>| {
                    let item_node = all_nodes.iter().find(|y| y.get_item() == *x).unwrap();
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
        }

        let is_type_simple = item_node.is_type_simple();
        if !is_type_simple && !is_type_undeclared {
            list.extend(vec![
                Self::ProcessAndFinish,
                Self::Cover,
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
    all_items: &[&Item<'_>],
    all_item_nodes: &[ItemNode<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = BulletListSingleItemSelection::create_list(menu_for, all_item_nodes);

    let selection = Select::new("", list).with_page_size(14).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ASimpleThingICanDoRightNow) => {
            todo!("TODO: Implement ASimpleThingICanDoRightNOw");
        }
        Ok(BulletListSingleItemSelection::ASimpleThingButICanDoItWhenever) => {
            todo!("TODO: Implement ASimpleThingButICanDoItWhenever");
        }
        Ok(BulletListSingleItemSelection::ICannotDoThisSimpleThingRightNowRemindMeLater) => {
            todo!("TODO: Implement ICannotDoThisSimpleThingRightNowRemindMeLater");
        }
        Ok(BulletListSingleItemSelection::DeclareItemType) => {
            declare_item_type(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::StateASmallerNextStep) => {
            state_a_smaller_next_step(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::ParentToAGoal) => {
            parent_to_a_goal(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::PlanWhenToDoThis) => {
            todo!("TODO: Implement PlanWhenToDoThis");
        }
        Ok(BulletListSingleItemSelection::ParentToAMotivation) => {
            parent_to_a_motivation(menu_for.get_item(), send_to_data_storage_layer).await
        }
        Ok(BulletListSingleItemSelection::DoInAFocusPeriod) => {
            todo!("TODO: Implement DoInAFocusPeriod");
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
        Ok(BulletListSingleItemSelection::DefineChildHopes) => {
            todo!("TODO: Implement DefineChildHopes");
        }
        Ok(BulletListSingleItemSelection::UpdateChildHopes) => {
            todo!("TODO: Implement UpdateChildHopes");
        }
        Ok(BulletListSingleItemSelection::DefineMilestones) => {
            todo!("TODO: Implement DefineMilestones");
        }
        Ok(BulletListSingleItemSelection::UpdateMilestones) => {
            todo!("TODO: Implement UpdateMilestones");
        }
        Ok(BulletListSingleItemSelection::WorkedOnThis) => {
            todo!("TODO: Implement WorkedOnThis");
        }
        Ok(BulletListSingleItemSelection::Finished) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::FinishItem(
                    menu_for.get_surreal_item().clone(),
                ))
                .await
                .unwrap();

            let mut parents_iter = menu_for.get_larger().iter();
            let next_item = parents_iter.next();
            if let Some(next_item) = next_item {
                let next_item = all_item_nodes
                    .iter()
                    .find(|x| x.get_item() == next_item.get_item())
                    .unwrap();
                let display_item = DisplayItemNode::new(next_item);
                println!("{}", display_item);
                present_bullet_list_item_selected(
                    next_item,
                    all_items,
                    all_item_nodes,
                    send_to_data_storage_layer,
                )
                .await
            }
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
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            let r = cover_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    present_bullet_list_item_selected(
                        menu_for,
                        all_items,
                        all_item_nodes,
                        send_to_data_storage_layer,
                    )
                    .await
                }
                Err(UnexpectedNextMenuAction::Close) => todo!(),
            }
        }
        Ok(BulletListSingleItemSelection::UpdateSummary) => {
            update_item_summary(
                menu_for.get_surreal_item().clone(),
                send_to_data_storage_layer,
            )
            .await
        }
        Ok(BulletListSingleItemSelection::SwitchToParentItem(_, selected)) => {
            present_bullet_list_item_parent_selected(
                selected,
                all_items,
                all_item_nodes,
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

async fn process_and_finish_bullet_item(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    //I should probably be processing and finishing all of the children next steps but this requires some thought
    //because sometimes or if there are multiple children next steps that that shouldn't happen rather the user
    //should be prompted to pick which children to also process and finish.
    let user_processed_text = Editor::new("Process text").prompt().unwrap();

    let surreal_item = item.get_surreal_item();
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
    all_items: &[&Item<'_>],
    all_item_nodes: &[ItemNode<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    match selected_item.get_type() {
        ItemType::ToDo | ItemType::Hope => {
            present_bullet_list_item_selected(
                selected_item,
                all_items,
                all_item_nodes,
                send_to_data_storage_layer,
            )
            .await
        }
        ItemType::Motivation => todo!(),
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
    let base_data = BaseData::new_from_surreal_tables(raw_data);
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
                    child: parent_this.get_surreal_item().clone(),
                    parent: item.get_surreal_item().clone(),
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
    let base_data = BaseData::new_from_surreal_tables(raw_data);
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
                    child: item.get_surreal_item().clone(),
                    parent: parent_this.get_surreal_item().clone(),
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

enum ItemTypeSelection {
    ProactiveAction,
    ResponsiveAction,
    ProactiveGoal,
    ResponsiveGoal,
    ProactiveMotivation,
    ResponsiveMotivation,
}

impl Display for ItemTypeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProactiveAction => write!(f, "New Proactive Action"),
            Self::ResponsiveAction => write!(f, "New Responsive Action"),
            Self::ProactiveGoal => write!(f, "New Proactive Goal"),
            Self::ResponsiveGoal => write!(f, "New Responsive Goal"),
            Self::ProactiveMotivation => write!(f, "New Proactive Motivation"),
            Self::ResponsiveMotivation => write!(f, "New Responsive Motivation"),
        }
    }
}

impl ItemTypeSelection {
    pub(crate) fn create_list() -> Vec<Self> {
        vec![
            Self::ProactiveAction,
            Self::ResponsiveAction,
            Self::ProactiveGoal,
            Self::ResponsiveGoal,
            Self::ProactiveMotivation,
            Self::ResponsiveMotivation,
        ]
    }

    pub(crate) fn create_list_just_goals() -> Vec<Self> {
        vec![Self::ProactiveGoal, Self::ResponsiveGoal]
    }

    pub(crate) fn create_list_just_motivations() -> Vec<Self> {
        vec![Self::ProactiveMotivation, Self::ResponsiveMotivation]
    }

    pub(crate) fn create_new_item_prompt_user_for_summary(&self) -> new_item::NewItem {
        let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
        self.create_new_item_prompt(summary)
    }

    pub(crate) fn create_new_item_prompt(&self, summary: String) -> new_item::NewItem {
        let mut new_item_builder = new_item::NewItemBuilder::default();
        let new_item_builder = new_item_builder.summary(summary);
        let new_item_builder = match self {
            ItemTypeSelection::ProactiveAction => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(ItemType::ToDo),
            ItemTypeSelection::ResponsiveAction => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(ItemType::ToDo),
            ItemTypeSelection::ProactiveGoal => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(ItemType::Hope),
            ItemTypeSelection::ResponsiveGoal => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(ItemType::Hope),
            ItemTypeSelection::ProactiveMotivation => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(ItemType::Motivation),
            ItemTypeSelection::ResponsiveMotivation => new_item_builder
                .responsibility(Responsibility::ReactiveBeAvailableToAct)
                .item_type(ItemType::Motivation),
        };
        new_item_builder
            .build()
            .expect("Filled out required fields")
    }
}

pub(crate) async fn parent_to_new_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}

pub(crate) async fn cover_with_new_item(
    cover_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(item_type_selection) => {
            let new_item = item_type_selection.create_new_item_prompt_user_for_summary();
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverItemWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}

async fn declare_item_type(
    item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = ItemTypeSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(ItemTypeSelection::ProactiveAction) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::ToDo,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ResponsiveAction) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    ItemType::ToDo,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ProactiveGoal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::Hope,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ResponsiveGoal) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    ItemType::Hope,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ProactiveMotivation) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ProactiveActionToTake,
                    ItemType::Motivation,
                ))
                .await
                .unwrap();
        }
        Ok(ItemTypeSelection::ResponsiveMotivation) => {
            send_to_data_storage_layer
                .send(DataLayerCommands::UpdateResponsibilityAndItemType(
                    item.get_surreal_item().clone(),
                    Responsibility::ReactiveBeAvailableToAct,
                    ItemType::Motivation,
                ))
                .await
                .unwrap();
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

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(IsAPersonOrGroupAroundSelection::Yes) => send_to_data_storage_layer
            .send(DataLayerCommands::FinishItem(
                person_or_group_node.get_surreal_item().clone(),
            ))
            .await
            .unwrap(),
        Ok(IsAPersonOrGroupAroundSelection::No) => todo!(),
        Err(InquireError::OperationCanceled) => todo!(),
        Err(err) => todo!("Unexpected {}", err),
    }
}
