pub(crate) mod give_this_item_a_parent;
pub(crate) mod log_worked_on_this;
pub(crate) mod prompt_priority_for_new_item;
mod something_else_should_be_done_first;
pub(crate) mod state_a_smaller_action;
pub(crate) mod urgency_plan;

use std::fmt::Display;

use better_term::Style;
use chrono::{DateTime, Utc};
use inquire::{InquireError, Select, Text};
use tokio::sync::mpsc::Sender;
use urgency_plan::present_set_ready_and_urgency_plan_menu;

use crate::{
    base_data::{item::Item, BaseData},
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_item::{
            Responsibility, SurrealHowMuchIsInMyControl, SurrealItemType, SurrealMotivationKind,
            SurrealUrgency,
        },
        surreal_tables::SurrealTables,
    },
    display::{
        display_item::DisplayItem, display_item_node::DisplayItemNode,
        display_item_type::DisplayItemType, display_urgency_plan::DisplayUrgency, DisplayStyle,
    },
    menu::inquire::{
        bullet_list_menu::{
            bullet_list_single_item::{
                give_this_item_a_parent::give_this_item_a_parent,
                something_else_should_be_done_first::something_else_should_be_done_first,
                state_a_smaller_action::state_a_smaller_action,
            },
            review_item,
        },
        select_higher_importance_than_this::select_higher_importance_than_this,
        top_menu::capture,
        update_item_summary::update_item_summary,
    },
    new_item,
    node::{
        item_node::{DependencyWithItem, ItemNode},
        item_status::ItemStatus,
        Filter,
    },
    systems::bullet_list::BulletList,
};

pub(crate) enum LogTime {
    SeparateTaskLogTheTime,
    PartOfAnotherTaskDoNotLogTheTime,
}

enum BulletListSingleItemSelection<'e> {
    ChangeItemType { current: &'e SurrealItemType },
    CaptureNewItem,
    GiveThisItemAParent,
    ChangeReadyAndUrgencyPlan,
    UnableToDoThisRightNow,
    SomethingElseShouldBeDoneFirst,
    ReviewItem,
    StateASmallerAction,
    WorkedOnThis,
    Finished,
    ReturnToBulletList,
    UpdateSummary,
    SwitchToParentItem(DisplayItem<'e>, &'e ItemStatus<'e>),
    ParentToItem,
    RemoveParent(DisplayItem<'e>, &'e ItemStatus<'e>),
    SwitchToChildItem(DisplayItem<'e>, &'e ItemStatus<'e>),
    RemoveChild(DisplayItem<'e>, &'e ItemStatus<'e>),
    DebugPrintItem,
}

impl Display for BulletListSingleItemSelection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureNewItem => write!(f, "Capture New Item"),
            Self::UpdateSummary => write!(f, "Update Summary"),
            Self::SwitchToParentItem(parent_item, _) => {
                write!(f, "⇄ Select larger Reason: {}", parent_item)
            }
            Self::StateASmallerAction => {
                write!(f, "State a smaller Action")
            }
            Self::ReviewItem => write!(f, "Review Item"),
            Self::ParentToItem => {
                write!(f, "⭱ Pick another larger Reason")
            }
            Self::SwitchToChildItem(child_item, _) => {
                write!(f, "⇄ Select smaller Action: {}", child_item)
            }
            Self::RemoveChild(child_item, _) => write!(f, "🚫 Remove action: {}", child_item),
            Self::RemoveParent(parent_item, _) => write!(f, "🚫 Remove reason: {}", parent_item),
            Self::DebugPrintItem => write!(f, "Debug Print Item"),
            Self::SomethingElseShouldBeDoneFirst => {
                write!(f, "Something else should be done first")
            }
            Self::ChangeItemType { current } => {
                let current_item_type = DisplayItemType::new(DisplayStyle::Full, current);
                write!(f, "Change Item Type (Currently: {})", current_item_type)
            }
            Self::GiveThisItemAParent => write!(f, "Pick a larger reason"),
            Self::UnableToDoThisRightNow => write!(f, "I am unable to do this right now"),
            Self::WorkedOnThis => write!(f, "I worked on this"),
            Self::Finished => write!(f, "I finished"),
            Self::ReturnToBulletList => write!(f, "Return to the Bullet List Menu"),
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

        if has_no_parent {
            list.push(Self::GiveThisItemAParent);
        }

        list.push(Self::CaptureNewItem);
        list.push(Self::WorkedOnThis);

        list.push(Self::Finished);

        list.push(Self::UnableToDoThisRightNow);

        list.push(Self::StateASmallerAction);

        list.push(Self::SomethingElseShouldBeDoneFirst);

        list.push(Self::ReviewItem);

        let parent_items = item_node
            .get_parents(Filter::Active)
            .map(|x| x.get_item())
            .collect::<Vec<_>>();
        if !has_no_parent {
            list.push(Self::ParentToItem);
        }
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

        list.push(Self::ChangeItemType {
            current: item_node.get_type(),
        });
        list.push(Self::ChangeReadyAndUrgencyPlan);

        list.extend(vec![
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
    println!();
    println!("Selected Item:");
    println!("  {}", DisplayItem::new(menu_for.get_item()));
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
        Ok(BulletListSingleItemSelection::StateASmallerAction) => {
            state_a_smaller_action(menu_for.get_item_node(), send_to_data_storage_layer).await?;
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
        Ok(BulletListSingleItemSelection::UnableToDoThisRightNow) => {
            present_set_ready_and_urgency_plan_menu(
                menu_for,
                menu_for.get_urgency_now().cloned(),
                LogTime::PartOfAnotherTaskDoNotLogTheTime,
                send_to_data_storage_layer,
            )
            .await
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
        Ok(BulletListSingleItemSelection::ChangeReadyAndUrgencyPlan) => {
            present_set_ready_and_urgency_plan_menu(
                menu_for,
                menu_for.get_urgency_now().cloned(),
                LogTime::PartOfAnotherTaskDoNotLogTheTime,
                send_to_data_storage_layer,
            )
            .await
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
        Ok(BulletListSingleItemSelection::ReturnToBulletList)
        | Err(InquireError::OperationCanceled) => Ok(()), //Nothing to do we just want to return to the bullet list
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

fn print_completed_children(menu_for: &ItemStatus<'_>) {
    let mut completed_children = menu_for
        .get_children(Filter::Finished)
        .map(|x| x.get_item())
        .collect::<Vec<_>>();
    completed_children.sort_by(|a, b| a.get_finished_at().cmp(b.get_finished_at()));
    if !completed_children.is_empty() {
        println!("Completed Actions:",);
        for child in completed_children.iter().take(8) {
            println!("  ✅{}", DisplayItem::new(child));
        }
        if completed_children.len() > 8 {
            println!("  {} more ✅", completed_children.len() - 8);
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
        println!("Smaller Actions:");
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

            state_a_smaller_action(&updated_parent, send_to_data_storage_layer).await?;

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
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
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
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

pub(crate) enum ItemTypeSelection {
    Action,
    Goal,
    MotivationCore,
    MotivationNonCore,
    NormalHelp,
}

impl Display for ItemTypeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action => write!(f, "Step 🪜"),
            Self::Goal => write!(f, "Project 🪧"),
            Self::MotivationCore => {
                write!(f, "Core Motivational Purpose 🎯🏢")
            }
            Self::MotivationNonCore => {
                write!(f, "Non-Core Motivational Purpose 🎯🏞")
            }
            Self::NormalHelp => write!(f, "Help"),
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
            ItemTypeSelection::MotivationCore => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Motivation(SurrealMotivationKind::CoreWork)),
            ItemTypeSelection::MotivationNonCore => new_item_builder
                .responsibility(Responsibility::ProactiveActionToTake)
                .item_type(SurrealItemType::Motivation(
                    SurrealMotivationKind::NonCoreWork,
                )),
            ItemTypeSelection::NormalHelp => {
                panic!("NormalHelp should be handled before this point")
            }
        };
        new_item_builder
            .build()
            .expect("Filled out required fields")
    }

    pub(crate) fn print_normal_help() {
        println!("{}Step{}", Style::default().bold(), Style::default());
        println!("A thing to do and an action or step to take.");
        println!(
            "{}The emoji is a ladder 🪜 with steps.{}",
            Style::default().italic(),
            Style::default()
        );
        println!();
        println!(
            "{}Multi-Step Project{}",
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
            "{}Motivational Purpose or Reason{}",
            Style::default().bold(),
            Style::default()
        );
        println!("For stating that the item captured is a reason for doing something.");
        println!(
            "{}Emoji is a target 🎯 that provides something to aim for.{}",
            Style::default().italic(),
            Style::default()
        );
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
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
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
        Ok(ItemTypeSelection::NormalHelp) => {
            ItemTypeSelection::print_normal_help();
            Box::pin(declare_item_type(item, send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationCanceled) => todo!(),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
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
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}
