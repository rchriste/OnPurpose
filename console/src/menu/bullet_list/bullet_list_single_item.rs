mod cover_bullet_item;

use std::fmt::Display;

use async_recursion::async_recursion;
use inquire::{Editor, InquireError, Select, Text};
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{
        item::{Item, ItemVecExtensions},
        to_do::ToDo,
        ItemType,
    },
    display::display_item::DisplayItem,
    menu::{
        bullet_list::bullet_list_single_item::cover_bullet_item::cover_bullet_item,
        unable_to_work_on_item_right_now::unable_to_work_on_item_right_now,
    },
    new_item,
    node::person_or_group_node::PersonOrGroupNode,
    surrealdb_layer::{surreal_item::Responsibility, DataLayerCommands},
    update_item_summary, UnexpectedNextMenuAction,
};

enum BulletListSingleItemSelection<'e> {
    ASimpleThingICanDoQuickly,
    ICannotDoThisSimpleThingRightNowRemindMeLater,
    DeclareItemType,
    ParentToAGoal,
    ParentToAMotivation,
    DoInAFocusPeriod,
    EstimateHowManyFocusPeriodsThisWillTake,
    UnableToDoThisRightNow,
    NotInTheMoodToDoThisRightNow,
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
    ReturnToBulletList,
    ProcessAndFinish,
    Cover,
    UpdateSummary,
    //Take a DisplayItem rather than a reference because it is felt that this type is only created
    //for this scenario rather than kept around.
    SwitchToParentItem(DisplayItem<'e>),
    ParentToItem,
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessAndFinish => write!(f, "Process & Finish 📕"),
            Self::Cover => write!(f, "Cover ⼍"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item) => write!(f, "Switch to: {}", parent_item),
            Self::ParentToItem => {
                write!(f, "⭱ Parent to a new or existing Item")
            }
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
            Self::ASimpleThingICanDoQuickly => write!(f, "This is a simple thing I can do quickly"),
            Self::ICannotDoThisSimpleThingRightNowRemindMeLater => {
                write!(f, "I cannot do this right now, remind me later")
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
        }
    }
}

impl<'e> BulletListSingleItemSelection<'e> {
    fn create_list(item: &'e Item<'e>, parent_items: &[&'e Item<'e>]) -> Vec<Self> {
        let mut list = Vec::default();

        if item.is_type_undeclared() {
            list.push(Self::ASimpleThingICanDoQuickly);
            list.push(Self::DeclareItemType);
        }

        if item.is_type_action() || item.is_type_hope() || item.is_type_motivation() {
            list.push(Self::WorkedOnThis);
        }

        list.push(Self::Finished);

        if item.is_type_action() || item.is_type_hope() || item.is_type_motivation() {
            list.push(Self::UnableToDoThisRightNow);
            list.push(Self::NotInTheMoodToDoThisRightNow);
        }

        if !item.is_circumstance_focus_time() {
            list.push(Self::DoInAFocusPeriod);
        } else if item.get_estimated_focus_periods().is_none() {
            list.push(Self::EstimateHowManyFocusPeriodsThisWillTake)
        }

        if item.is_type_action() || item.is_type_hope() {
            list.push(Self::WaitUntilSimilarWorkIsDone);
            list.push(Self::SearchForSimilarWork);
        }

        if item.is_there_notes() {
            list.push(Self::OpenNotesForThisItem);
        } else {
            list.push(Self::CreateNotesForThisItem);
            list.push(Self::LinkNotesForThisItem);
        }

        if item.is_type_simple_thing() {
            list.push(Self::ICannotDoThisSimpleThingRightNowRemindMeLater);
        }

        if item.is_type_action() {
            list.push(Self::ParentToAGoal);
        }

        if item.is_type_action() || item.is_type_hope() {
            list.push(Self::ParentToAMotivation);
        }

        if item.is_type_action() || item.is_type_hope() || item.is_type_motivation() {
            if item.has_children() {
                list.push(Self::UpdateChildActions);
            } else {
                list.push(Self::DefineChildActions);
            }
        }

        if item.is_type_motivation() {
            if item.has_children() {
                list.push(Self::UpdateChildHopes);
            } else {
                list.push(Self::DefineChildHopes);
            }
        }

        if item.is_type_hope() {
            if item.has_children() {
                list.push(Self::UpdateMilestones);
            } else {
                list.push(Self::DefineMilestones);
            }
        }

        if parent_items.is_empty() {
            list.push(Self::ParentToItem);
        } else {
            list.extend(
                parent_items
                    .iter()
                    .map(|x: &&'e Item<'e>| Self::SwitchToParentItem(DisplayItem::new(x))),
            );
        }

        for parent in parent_items {
            if parent.is_there_notes() {
                list.push(Self::OpenNotesForParentItem {
                    item_in_chain_with_notes: DisplayItem::new(parent),
                });
            }
        }

        if item.is_type_action() || item.is_type_hope() {
            list.push(Self::ThisIsARepeatingItem);
        }

        list.extend(vec![
            Self::ProcessAndFinish,
            Self::Cover,
            Self::UpdateSummary,
            Self::DebugPrintItem,
            Self::ReturnToBulletList,
        ]);

        list
    }
}

#[async_recursion]
pub(crate) async fn present_bullet_list_item_selected(
    menu_for: &ToDo<'_>, //TODO: This should take an Item and the other functions that take a Hope should be folded into this
    parents: &[&Item<'_>],
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = BulletListSingleItemSelection::create_list(menu_for.get_item(), parents);

    let selection = Select::new("", list).with_page_size(14).prompt();

    match selection {
        Ok(BulletListSingleItemSelection::ASimpleThingICanDoQuickly) => {
            todo!("TODO: Implement ASimpleThingICanDoQuickly");
        }
        Ok(BulletListSingleItemSelection::ICannotDoThisSimpleThingRightNowRemindMeLater) => {
            todo!("TODO: Implement ICannotDoThisSimpleThingRightNowRemindMeLater");
        }
        Ok(BulletListSingleItemSelection::DeclareItemType) => {
            todo!("TODO: Implement DeclareItemType");
        }
        Ok(BulletListSingleItemSelection::ParentToAGoal) => {
            todo!("TODO: Implement ParentToAGoal");
        }
        Ok(BulletListSingleItemSelection::ParentToAMotivation) => {
            todo!("TODO: Implement ParentToAMotivation");
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
                .send(DataLayerCommands::FinishItem(menu_for.get_surreal_item().clone()))
                .await
                .unwrap();
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
        Ok(BulletListSingleItemSelection::ProcessAndFinish) => {
            process_and_finish_bullet_item(menu_for.get_item(), send_to_data_storage_layer).await;
        }
        Ok(BulletListSingleItemSelection::Cover) => {
            let r = cover_bullet_item(menu_for, send_to_data_storage_layer).await;
            match r {
                Ok(()) => (),
                Err(UnexpectedNextMenuAction::Back) => {
                    present_bullet_list_item_selected(menu_for, parents, send_to_data_storage_layer)
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
        Ok(BulletListSingleItemSelection::SwitchToParentItem(item)) => {
            present_bullet_list_item_parent_selected(item.into(), send_to_data_storage_layer).await
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
    selected_item: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    match selected_item.item_type {
        ItemType::ToDo => {
            let to_do = ToDo::new(selected_item);
            let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
                .await
                .unwrap();
            let items = raw_data.make_items();
            let active_items = items.filter_active_items();
            let coverings = raw_data.make_coverings(&items);
            let visited = vec![];
            let parents = selected_item.find_parents(&coverings, &active_items, &visited);
            present_bullet_list_item_selected(&to_do, &parents, send_to_data_storage_layer).await
        }
        ItemType::Hope => todo!(),
        ItemType::Motivation => todo!(),
        ItemType::Undeclared => todo!(),
        ItemType::SimpleThing => todo!(),
        ItemType::PersonOrGroup => todo!(),
    }
}

async fn parent_to_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items: Vec<Item> = raw_data
        .make_items()
        .into_iter()
        .filter(|x| !x.is_finished())
        .collect();

    let list = DisplayItem::make_list(&items);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item: &Item = display_item.into();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent: item.get_surreal_item().clone(),
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
    let raw_data = DataLayerCommands::get_raw_data(send_to_data_storage_layer)
        .await
        .unwrap();
    let items: Vec<Item> = raw_data
        .make_items()
        .into_iter()
        .filter(|x| !x.is_finished())
        .collect();

    let list = DisplayItem::make_list(&items);

    let selection = Select::new("Type to Search or Press Esc to enter a new one", list).prompt();
    match selection {
        Ok(display_item) => {
            let item: &Item = display_item.into();
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithExistingItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent: item.get_surreal_item().clone(),
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

enum NewItemSelection {
    ProactiveToDo,
    ResponsiveToDo,
    ProactiveHope,
    ProactiveMilestone,
    ResponsiveHope,
    ProactiveMotivation,
    ResponsiveMotivation,
}

impl Display for NewItemSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProactiveToDo => write!(f, "New Proactive To Do"),
            Self::ResponsiveToDo => write!(f, "New Responsive To Do"),
            Self::ProactiveHope => write!(f, "New Proactive Hope"),
            Self::ResponsiveHope => write!(f, "New Responsive Hope"),
            Self::ProactiveMotivation => write!(f, "New Proactive Motivation"),
            Self::ResponsiveMotivation => write!(f, "New Responsive Motivation"),
            Self::ProactiveMilestone => write!(f, "New Proactive Milestone"),
        }
    }
}

impl NewItemSelection {
    fn create_list() -> Vec<Self> {
        vec![
            Self::ProactiveToDo,
            Self::ResponsiveToDo,
            Self::ProactiveHope,
            Self::ProactiveMilestone,
            Self::ResponsiveHope,
            Self::ProactiveMotivation,
            Self::ResponsiveMotivation,
        ]
    }
}

pub(crate) async fn parent_to_new_item(
    parent_this: &Item<'_>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) {
    let list = NewItemSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(NewItemSelection::ProactiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMilestone) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let _new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            todo!("I need to also set this hope to be a milestone");
            // send_to_data_storage_layer.send(DataLayerCommands::ParentItemWithANewItem{
            //     child: parent_this.get_surreal_item().clone(),
            //     parent_new_item: new_item,
            // }).await.unwrap();
        }
        Ok(NewItemSelection::ResponsiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::ParentItemWithANewItem {
                    child: parent_this.get_surreal_item().clone(),
                    parent_new_item: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Motivation,
            };
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
    let list = NewItemSelection::create_list();

    let selection = Select::new("", list).prompt();
    match selection {
        Ok(NewItemSelection::ProactiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveToDo) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::ToDo,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMilestone) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let _new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Hope,
            };
            todo!("I need to also set this hope to be a milestone");
            // send_to_data_storage_layer.send(DataLayerCommands::CoverWithANewItem{
            //     cover_this: cover_this.get_surreal_item().clone(),
            //     cover_with: new_item,
            // }).await.unwrap();
        }
        Ok(NewItemSelection::ResponsiveHope) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Hope,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ProactiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ProactiveActionToTake,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
                    cover_this: cover_this.get_surreal_item().clone(),
                    cover_with: new_item,
                })
                .await
                .unwrap();
        }
        Ok(NewItemSelection::ResponsiveMotivation) => {
            let summary = Text::new("Enter Summary ⍠").prompt().unwrap();
            let new_item = new_item::NewItem {
                summary,
                finished: None,
                responsibility: Responsibility::ReactiveBeAvailableToAct,
                item_type: ItemType::Motivation,
            };
            send_to_data_storage_layer
                .send(DataLayerCommands::CoverWithANewItem {
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
    person_or_group_node: &PersonOrGroupNode<'_>,
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
