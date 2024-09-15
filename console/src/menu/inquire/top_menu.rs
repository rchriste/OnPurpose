use std::{cmp::Ordering, collections::HashMap, fmt::Display};

use chrono::{DateTime, Local, Utc};
use duration_str::parse;
use inquire::{InquireError, Select, Text};
use surrealdb::opt::RecordId;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::{time_spent::TimeSpent, BaseData},
    calculated_data::CalculatedData,
    change_routine::change_routine,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands, surreal_tables::SurrealTables,
    },
    display::{
        display_duration::DisplayDuration, display_item_node::DisplayItemNode,
        display_item_status::DisplayItemStatus,
    },
    menu::{inquire::expectations::view_expectations, ratatui::view_priorities},
    new_item::NewItem,
    node::{item_node::ItemNode, item_status::ItemStatus, Filter},
};

use super::{
    bullet_list_menu::present_normal_bullet_list_menu, update_item_summary::update_item_summary,
};

enum TopMenuSelection {
    ChangeRoutine,
    Reflection,
    ViewBulletList,
    ViewExpectations,
    ViewMotivations,
    ViewPriorities,
    ViewPrioritiesRatatui,
    DebugViewAllItems,
}

impl Display for TopMenuSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopMenuSelection::ChangeRoutine => write!(f, "↝ ↝  Change Routine            ↜"),
            TopMenuSelection::Reflection => write!(f, "      Reflection                 "),
            TopMenuSelection::ViewBulletList => {
                write!(f, "👁 🗒️  View Bullet List          👁")
            }
            TopMenuSelection::ViewExpectations => {
                write!(f, "👁 🙏 View Expectations          👁")
            }
            TopMenuSelection::ViewMotivations => {
                write!(f, "👁 🎯 View Motivations           👁")
            }
            TopMenuSelection::ViewPriorities => write!(f, "👁 ⚖️  View Priorities           👁"),
            TopMenuSelection::ViewPrioritiesRatatui => {
                write!(f, "👁 ⚖️  View Priorities (Ratatui) 👁")
            }
            TopMenuSelection::DebugViewAllItems => {
                write!(f, "👁 🗒️  Debug View All Items      👁")
            }
        }
    }
}

impl TopMenuSelection {
    fn make_list() -> Vec<TopMenuSelection> {
        vec![
            Self::ViewPriorities,
            Self::ViewPrioritiesRatatui,
            Self::ChangeRoutine,
            Self::Reflection,
            Self::ViewBulletList,
            Self::ViewExpectations,
            Self::ViewMotivations,
            Self::DebugViewAllItems,
        ]
    }
}

pub(crate) async fn present_top_menu(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let top_menu = TopMenuSelection::make_list();

    let selection = Select::new("Select from the below list|", top_menu).prompt();
    match selection {
        Ok(TopMenuSelection::ChangeRoutine) => change_routine(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::Reflection) => present_reflection(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::ViewExpectations) => {
            view_expectations(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewBulletList) => {
            present_normal_bullet_list_menu(send_to_data_storage_layer).await
        }
        Ok(TopMenuSelection::ViewMotivations) => view_motivations().await,
        Ok(TopMenuSelection::ViewPriorities) => view_priorities(send_to_data_storage_layer).await,
        Ok(TopMenuSelection::ViewPrioritiesRatatui) => {
            view_priorities::view_priorities().map_err(|_| ())
        }
        Ok(TopMenuSelection::DebugViewAllItems) => {
            debug_view_all_items(send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled) => Err(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

pub(crate) async fn capture(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let new_item_summary = Text::new("Enter New Item ⍠").prompt();

    match new_item_summary {
        Ok(new_item_summary) => {
            let new_item = NewItem::new(new_item_summary, Utc::now());
            send_to_data_storage_layer
                .send(DataLayerCommands::NewItem(new_item))
                .await
                .unwrap();
            Ok(())
        }
        Err(InquireError::OperationCanceled) => Ok(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

async fn view_motivations() -> Result<(), ()> {
    todo!()
}

async fn view_priorities(send_to_data_storage_layer: &Sender<DataLayerCommands>) -> Result<(), ()> {
    let before_db_query = Local::now();
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let elapsed = Local::now() - before_db_query;
    if elapsed > chrono::Duration::try_seconds(1).expect("valid") {
        println!("Slow to get data from database. Time taken: {}", elapsed);
    }
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data);

    let mut all_top_nodes = calculated_data
        .get_items_status()
        .iter()
        .filter(|x| !x.is_finished())
        //Person or group items without a parent, meaning a reason for being on the list,
        // should be filtered out.
        .filter(|x| x.has_children(Filter::Active) && !x.has_parents(Filter::Active))
        .cloned()
        .collect::<Vec<_>>();

    all_top_nodes.sort_by(|a, b| {
        (if a.is_type_motivation() {
            if b.is_type_motivation() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if b.is_type_motivation() {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
        .then_with(|| a.get_summary().cmp(b.get_summary()))
    });

    let list = all_top_nodes
        .iter()
        .map(DisplayItemStatus::new)
        .collect::<Vec<_>>();

    let selection = Select::new("Select a priority to view...", list).prompt();
    match selection {
        Ok(display_item_status) => {
            view_priorities_of_item_status(
                display_item_status,
                Vec::new(),
                &calculated_data,
                &now,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_top_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

async fn view_priorities_of_item_status(
    display_item_status: DisplayItemStatus<'_>,
    mut parent: Vec<DisplayItemStatus<'_>>,
    calculated_data: &CalculatedData,
    now: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    println!("{}", display_item_status);
    let item_status = display_item_status.get_item_status();
    println!("Active children (Is this in priority order?):");
    let list = item_status
        .get_children(Filter::Active)
        .map(|x| {
            let item_status = calculated_data
                .get_items_status()
                .iter()
                .find(|y| y.get_item() == x.get_item())
                .expect("Comes from this list so will be found");
            DisplayItemStatus::new(item_status)
        })
        .collect();
    let selection = Select::new("Select a child to view...", list).prompt();
    match selection {
        Ok(display_priority) => {
            println!("{}", display_priority);
            parent.push(display_item_status);
            if display_priority.has_children(Filter::Active) {
                let display_item_status =
                    DisplayItemStatus::new(display_priority.get_item_status());
                Box::pin(view_priorities_of_item_status(
                    display_item_status,
                    parent,
                    calculated_data,
                    now,
                    send_to_data_storage_layer,
                ))
                .await
            } else {
                view_priorities_single_item_no_children(
                    display_priority.get_item_status(),
                    parent,
                    calculated_data,
                    now,
                    send_to_data_storage_layer,
                )
                .await
            }
        }
        Err(InquireError::OperationCanceled) => {
            if parent.is_empty() {
                Box::pin(view_priorities(send_to_data_storage_layer)).await
            } else {
                let top_item = parent.pop().expect("is not empty so will always succeed");
                Box::pin(view_priorities_of_item_status(
                    top_item,
                    parent,
                    calculated_data,
                    now,
                    send_to_data_storage_layer,
                ))
                .await
            }
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

enum ViewPrioritiesSingleItemNoChildrenChoice {
    Back,
    Finish,
    EditSummary,
}

impl Display for ViewPrioritiesSingleItemNoChildrenChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewPrioritiesSingleItemNoChildrenChoice::Finish => write!(f, "Finish"),
            ViewPrioritiesSingleItemNoChildrenChoice::EditSummary => write!(f, "Edit Summary"),
            ViewPrioritiesSingleItemNoChildrenChoice::Back => write!(f, "Back"),
        }
    }
}

async fn view_priorities_single_item_no_children(
    item_status: &ItemStatus<'_>,
    mut parent: Vec<DisplayItemStatus<'_>>,
    calculated_data: &CalculatedData,
    now: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let choices = vec![
        ViewPrioritiesSingleItemNoChildrenChoice::Back,
        ViewPrioritiesSingleItemNoChildrenChoice::EditSummary,
        ViewPrioritiesSingleItemNoChildrenChoice::Finish,
    ];
    let selection = Select::new("Select an action...", choices).prompt();
    match selection {
        Ok(ViewPrioritiesSingleItemNoChildrenChoice::Finish) => {
            let now = Utc::now();
            send_to_data_storage_layer
                .send(DataLayerCommands::FinishItem {
                    item: item_status.get_item().get_id().clone(),
                    when_finished: now.into(),
                })
                .await
                .unwrap();
            Ok(())
        }
        Ok(ViewPrioritiesSingleItemNoChildrenChoice::EditSummary) => {
            update_item_summary(item_status.get_item(), send_to_data_storage_layer).await
        }
        Err(InquireError::OperationCanceled)
        | Ok(ViewPrioritiesSingleItemNoChildrenChoice::Back) => {
            let top_item = parent.pop().expect("is not empty so will always succeed");
            Box::pin(view_priorities_of_item_status(
                top_item,
                parent,
                calculated_data,
                now,
                send_to_data_storage_layer,
            ))
            .await
        }
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    }
}

#[allow(clippy::mutable_key_type)]
async fn present_reflection(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let now = Local::now();
    let start = match Text::new("Enter Staring Time").prompt() {
        Ok(when_started) => match parse(&when_started) {
            Ok(duration) => now - duration,
            Err(_) => match dateparser::parse(&when_started) {
                Ok(when_started) => when_started.into(),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    return Box::pin(present_reflection(send_to_data_storage_layer)).await;
                }
            },
        },
        Err(InquireError::OperationCanceled) => return Ok(()),
        Err(InquireError::OperationInterrupted) => return Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    };

    let end = match Text::new("Enter Ending Time").prompt() {
        Ok(when_finished) => match parse(&when_finished) {
            Ok(duration) => now - duration,
            Err(_) => match dateparser::parse(&when_finished) {
                Ok(when_finished) => when_finished.into(),
                Err(_) => {
                    println!("Invalid input. Please try again.");
                    return Box::pin(present_reflection(send_to_data_storage_layer)).await;
                }
            },
        },
        Err(InquireError::OperationCanceled) => {
            return Box::pin(present_reflection(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => return Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    };

    println!("Time spent between {} and {}", start, end);

    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let start_utc = start.with_timezone(&Utc);
    let end_utc = end.with_timezone(&Utc);

    let logs_in_range: Vec<_> = surreal_tables
        .make_time_spent_log()
        .filter(|x| x.is_within(&start_utc, &end_utc))
        .collect();

    let mut things_done: HashMap<RecordId, Vec<&TimeSpent>> = HashMap::default();
    for log in logs_in_range.iter() {
        for worked_towards in log.worked_towards().iter() {
            let h = things_done.entry(worked_towards.clone()).or_default();
            h.push(log);
        }
    }

    let base_data = BaseData::new_from_surreal_tables(surreal_tables.clone(), Utc::now());
    let calculated_data = CalculatedData::new_from_base_data(base_data);
    let items_status = calculated_data.get_items_status();

    let mut items_in_range: Vec<ItemTimeSpent<'_>> = things_done
        .into_iter()
        .map(|(k, v)| {
            let item_status = items_status
                .iter()
                .find(|x| x.get_item().get_id() == &k)
                .expect("All items in the log should be in the item status");
            ItemTimeSpent {
                item_status,
                time_spent: v,
            }
        })
        .collect();

    items_in_range.sort();
    for item in items_in_range.iter() {
        println!("{}", DisplayItemNode::new(item.item_status.get_item_node()));
        let iteration_count = item.time_spent.len();
        let total_time: chrono::Duration = item.time_spent.iter().map(|x| x.get_time_delta()).sum();
        let total_time: std::time::Duration = total_time.to_std().expect("valid");
        let display_duration = DisplayDuration::new(&total_time);
        println!("\t{} - {}", iteration_count, display_duration);
    }

    Ok(())
}

struct ItemTimeSpent<'s> {
    item_status: &'s ItemStatus<'s>,
    time_spent: Vec<&'s TimeSpent<'s>>,
}

impl PartialEq for ItemTimeSpent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item_status.get_item() == other.item_status.get_item()
    }
}

impl Eq for ItemTimeSpent<'_> {}

impl PartialOrd for ItemTimeSpent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemTimeSpent<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item_status
            .get_summary()
            .cmp(other.item_status.get_summary())
    }
}

enum DebugViewItem<'e> {
    Item(DisplayItemNode<'e>),
}

impl Display for DebugViewItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugViewItem::Item(item) => write!(f, "{}", item),
        }
    }
}

impl<'e> DebugViewItem<'e> {
    fn make_list(items: &'e [&'e ItemNode<'e>]) -> Vec<DebugViewItem<'e>> {
        items.iter().copied().map(DebugViewItem::new).collect()
    }

    fn new(item: &'e ItemNode<'e>) -> Self {
        Self::Item(DisplayItemNode::new(item))
    }
}

async fn debug_view_all_items(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();

    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let all_items = base_data.get_items();
    let active_items = base_data.get_active_items();
    let time_spent_log = base_data.get_time_spent_log();

    let item_nodes = active_items
        .iter()
        .map(|x| ItemNode::new(x, all_items, time_spent_log))
        .collect::<Vec<_>>();

    let item_nodes = item_nodes.iter().collect::<Vec<_>>();
    let list = DebugViewItem::make_list(&item_nodes);

    let selection = Select::new("Select an item to show the debug view of...", list).prompt();
    match selection {
        Ok(DebugViewItem::Item(item)) => {
            println!("{}", item);
            let item_node = item.get_item_node();
            println!("{:#?}", item_node);
            Ok(())
        }
        Err(InquireError::OperationCanceled) => {
            Box::pin(present_top_menu(send_to_data_storage_layer)).await
        }
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(err) => todo!("Unexpected InquireError of {}", err),
    }
}
