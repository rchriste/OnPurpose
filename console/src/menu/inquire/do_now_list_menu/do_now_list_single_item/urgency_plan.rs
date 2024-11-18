use ahash::HashSet;
use chrono::{DateTime, Utc};
use duration_str::parse;
use tokio::sync::mpsc::Sender;

use crate::{
    base_data::BaseData,
    calculated_data::CalculatedData,
    data_storage::surrealdb_layer::{
        data_layer_commands::DataLayerCommands,
        surreal_in_the_moment_priority::SurrealAction,
        surreal_item::{SurrealDependency, SurrealScheduled, SurrealUrgency, SurrealUrgencyPlan},
        surreal_tables::SurrealTables,
        SurrealItemsInScope, SurrealTrigger,
    },
    display::{
        display_dependencies_with_item_node::DisplayDependenciesWithItemNode,
        display_item_node::DisplayFormat,
    },
    menu::inquire::do_now_list_menu::do_now_list_single_item::state_a_smaller_action::{
        select_an_item, SelectAnItemSortingOrder,
    },
    new_time_spent::NewTimeSpent,
    node::{
        item_status::{DependencyWithItemNode, ItemStatus},
        why_in_scope_and_action_with_item_status::ToSurreal,
        Filter, Urgency,
    },
};
use inquire::{InquireError, Select, Text};
use std::fmt::{Display, Formatter};

use super::{LogTime, WhyInScope};

enum UrgencyPlanSelection {
    StaysTheSame,
    WillEscalate,
}

impl Display for UrgencyPlanSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrgencyPlanSelection::StaysTheSame => write!(f, "Stays the same"),
            UrgencyPlanSelection::WillEscalate => write!(f, "Escalate at trigger"),
        }
    }
}

enum ReadySelection {
    Now,
    NothingElse,
    AfterDateTime,
    AfterItem,
}

impl Display for ReadySelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadySelection::Now => write!(f, "Ready, now"),
            ReadySelection::NothingElse => write!(f, "Nothing else"),
            ReadySelection::AfterDateTime => {
                write!(f, "Cannot do, yet, wait an amount of wall clock time")
            }
            ReadySelection::AfterItem => {
                write!(f, "Cannot do, yet, wait until another item finishes")
            }
        }
    }
}

pub(crate) async fn prompt_for_dependencies_and_urgency_plan(
    currently_selected: Option<&ItemStatus<'_>>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> (Vec<(AddOrRemove, SurrealDependency)>, SurrealUrgencyPlan) {
    let ready = prompt_for_dependencies(currently_selected, send_to_data_storage_layer).await;
    let now = Utc::now();
    let urgency_plan = prompt_for_urgency_plan(&now, send_to_data_storage_layer).await;
    (ready.unwrap(), urgency_plan)
}

pub(crate) enum AddOrRemove {
    Add,
    Remove,
}

enum RemoveOrKeep {
    Remove,
    Keep,
}

impl Display for RemoveOrKeep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveOrKeep::Remove => write!(f, "Remove"),
            RemoveOrKeep::Keep => write!(f, "Keep"),
        }
    }
}

pub(crate) async fn prompt_for_dependencies(
    currently_selected: Option<&ItemStatus<'_>>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<Vec<(AddOrRemove, SurrealDependency)>, ()> {
    let mut result = Vec::default();
    let mut user_choose_to_keep = false;
    if let Some(currently_selected) = currently_selected {
        let currently_waiting_on = currently_selected.get_dependencies(Filter::Active);
        for currently_waiting_on in currently_waiting_on {
            match currently_waiting_on {
                DependencyWithItemNode::AfterDateTime { .. }
                | DependencyWithItemNode::AfterItem(..)
                | DependencyWithItemNode::DuringItem(..) => {
                    println!(
                        "{}",
                        DisplayDependenciesWithItemNode::new(
                            &vec![&currently_waiting_on],
                            Filter::Active,
                            DisplayFormat::SingleLine
                        )
                    );

                    let selection = Select::new(
                        "Do you want to keep or remove this dependency?",
                        vec![RemoveOrKeep::Keep, RemoveOrKeep::Remove],
                    )
                    .prompt()
                    .unwrap();
                    match selection {
                        RemoveOrKeep::Keep => {
                            //keep is default so do nothing
                            user_choose_to_keep = true;
                        }
                        RemoveOrKeep::Remove => {
                            result.push((AddOrRemove::Remove, currently_waiting_on.clone().into()));
                        }
                    }
                }
                DependencyWithItemNode::UntilScheduled { .. }
                | DependencyWithItemNode::AfterChildItem(..) => {
                    //Not stored in SurrealDependencies so just skip over
                }
            }
        }
    }
    let mut list = Vec::default();
    if user_choose_to_keep {
        list.push(ReadySelection::NothingElse);
    } else {
        list.push(ReadySelection::Now);
    }

    list.push(ReadySelection::AfterDateTime);
    list.push(ReadySelection::AfterItem);

    let ready = Select::new("When is this item ready?", list)
        .prompt()
        .unwrap();
    match ready {
        ReadySelection::Now | ReadySelection::NothingElse => {
            //do nothing
        }
        ReadySelection::AfterDateTime => {
            let exact_start = Text::new("Enter a date or an amount of time to wait, for example \"1/4/2025 3:00pm\", \"30m\", \"1h\", or \"1d\"\n|").prompt().unwrap();
            let exact_start = match parse(&exact_start) {
                Ok(exact_start) => {
                    let now = Utc::now();
                    now + exact_start
                }
                Err(_) => match dateparser::parse(&exact_start) {
                    Ok(exact_start) => exact_start,
                    Err(e) => {
                        todo!("Error: {:?}", e)
                    }
                },
            };
            result.push((
                AddOrRemove::Add,
                SurrealDependency::AfterDateTime(exact_start.into()),
            ));
        }
        ReadySelection::AfterItem => {
            let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
                .await
                .unwrap();
            let now = Utc::now();
            let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
            let calculated_data = CalculatedData::new_from_base_data(base_data);
            let excluded = if currently_selected.is_some() {
                vec![currently_selected.expect("is some").get_item()]
            } else {
                vec![]
            };
            let selected = select_an_item(
                excluded,
                SelectAnItemSortingOrder::NewestFirst,
                &calculated_data,
            )
            .await;
            match selected {
                Ok(Some(after_item)) => result.push((
                    AddOrRemove::Add,
                    SurrealDependency::AfterItem(after_item.get_surreal_record_id().clone()),
                )),
                Ok(None) => {
                    println!("Canceled");
                    todo!()
                }
                Err(()) => {
                    return Err(());
                }
            }
        }
    };
    Ok(result)
}

pub(crate) async fn prompt_for_urgency_plan(
    now: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> SurrealUrgencyPlan {
    println!("Initial Urgency");
    let initial_urgency = prompt_for_urgency();

    let urgency_plan = Select::new(
        "Does the urgency escalate?|",
        vec![
            UrgencyPlanSelection::StaysTheSame,
            UrgencyPlanSelection::WillEscalate,
        ],
    )
    .prompt()
    .unwrap();

    match urgency_plan {
        UrgencyPlanSelection::StaysTheSame => SurrealUrgencyPlan::StaysTheSame(initial_urgency),
        UrgencyPlanSelection::WillEscalate => {
            let triggers = prompt_for_triggers(now, send_to_data_storage_layer).await;

            println!("Later Urgency");
            let later_urgency = prompt_for_urgency();

            SurrealUrgencyPlan::WillEscalate {
                initial: initial_urgency,
                triggers,
                later: later_urgency,
            }
        }
    }
}

enum TriggerType {
    WallClockDateTime,
    LoggedInvocationCount,
    LoggedAmountOfTimeSpent,
}

impl Display for TriggerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::WallClockDateTime => write!(f, "After a wall clock date time"),
            TriggerType::LoggedInvocationCount => write!(f, "After a logged invocation count"),
            TriggerType::LoggedAmountOfTimeSpent => {
                write!(f, "After a logged amount of time spent")
            }
        }
    }
}

enum AddAnotherTrigger {
    AllDone,
    AddAnother,
}

impl Display for AddAnotherTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddAnotherTrigger::AllDone => write!(f, "Done adding triggers (recommended)"),
            AddAnotherTrigger::AddAnother => {
                write!(f, "Add another trigger, (only one trigger needs to happen)")
            }
        }
    }
}

pub(crate) async fn prompt_for_triggers(
    now: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Vec<SurrealTrigger> {
    let mut result = Vec::default();
    loop {
        let trigger = prompt_for_trigger(now, send_to_data_storage_layer).await;
        result.push(trigger);
        let more = Select::new(
            "Is there anything else that should also trigger?",
            vec![AddAnotherTrigger::AllDone, AddAnotherTrigger::AddAnother],
        )
        .prompt()
        .unwrap();
        match more {
            AddAnotherTrigger::AllDone => break,
            AddAnotherTrigger::AddAnother => continue,
        }
    }

    result
}

async fn prompt_for_trigger(
    now: &DateTime<Utc>,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> SurrealTrigger {
    let trigger_type = Select::new(
        "What type of trigger?",
        vec![
            TriggerType::WallClockDateTime,
            TriggerType::LoggedInvocationCount,
            TriggerType::LoggedAmountOfTimeSpent,
        ],
    )
    .prompt()
    .unwrap();

    match trigger_type {
        TriggerType::WallClockDateTime => {
            let exact_start = Text::new("Enter when you want to trigger")
                .prompt()
                .unwrap();
            let exact_start = match parse(&exact_start) {
                Ok(exact_start) => {
                    let now = Utc::now();
                    now + exact_start
                }
                Err(_) => match dateparser::parse(&exact_start) {
                    Ok(exact_start) => exact_start,
                    Err(e) => {
                        todo!("Error: {:?}", e)
                    }
                },
            };
            SurrealTrigger::WallClockDateTime(exact_start.into())
        }
        TriggerType::LoggedInvocationCount => {
            let count_needed = Text::new("Enter the count needed").prompt().unwrap();
            let count_needed = match count_needed.parse::<u32>() {
                Ok(count_needed) => count_needed,
                Err(e) => {
                    todo!("Error: {:?}", e)
                }
            };
            let items_in_scope = prompt_for_items_in_scope(send_to_data_storage_layer).await;

            SurrealTrigger::LoggedInvocationCount {
                starting: (*now).into(),
                count: count_needed,
                items_in_scope,
            }
        }
        TriggerType::LoggedAmountOfTimeSpent => {
            let amount_of_time = Text::new("Enter the amount of time").prompt().unwrap();
            let amount_of_time = match parse(&amount_of_time) {
                Ok(amount_of_time) => amount_of_time,
                Err(e) => {
                    todo!("Error: {:?}", e)
                }
            };
            let items_in_scope = prompt_for_items_in_scope(send_to_data_storage_layer).await;

            SurrealTrigger::LoggedAmountOfTime {
                starting: (*now).into(),
                duration: amount_of_time.into(),
                items_in_scope,
            }
        }
    }
}

enum ItemInScopeSelection {
    All,
    Include,
    Exclude,
}

impl Display for ItemInScopeSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemInScopeSelection::All => write!(f, "Any/all items"),
            ItemInScopeSelection::Include => write!(f, "Declare items to include"),
            ItemInScopeSelection::Exclude => write!(f, "Declare items to exclude"),
        }
    }
}

async fn prompt_for_items_in_scope(
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> SurrealItemsInScope {
    let selection = Select::new(
        "What items are in scope?",
        vec![
            ItemInScopeSelection::All,
            ItemInScopeSelection::Include,
            ItemInScopeSelection::Exclude,
        ],
    )
    .prompt()
    .unwrap();

    let surreal_tables = SurrealTables::new(send_to_data_storage_layer)
        .await
        .unwrap();
    let now = Utc::now();
    let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
    let calculated_data = CalculatedData::new_from_base_data(base_data);

    match selection {
        ItemInScopeSelection::All => SurrealItemsInScope::All,
        ItemInScopeSelection::Include => {
            let selected_items = prompt_for_items_to_select(&calculated_data).await;

            SurrealItemsInScope::Include(
                selected_items
                    .into_iter()
                    .map(|x| x.get_surreal_record_id().clone())
                    .collect(),
            )
        }
        ItemInScopeSelection::Exclude => {
            let selected_items = prompt_for_items_to_select(&calculated_data).await;

            SurrealItemsInScope::Exclude(
                selected_items
                    .into_iter()
                    .map(|x| x.get_surreal_record_id().clone())
                    .collect(),
            )
        }
    }
}

enum SelectAnother {
    SelectAnother,
    Done,
}

impl Display for SelectAnother {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectAnother::SelectAnother => write!(f, "Select another"),
            SelectAnother::Done => write!(f, "Done"),
        }
    }
}

async fn prompt_for_items_to_select(calculated_data: &CalculatedData) -> Vec<&ItemStatus> {
    let mut result: Vec<&ItemStatus> = Vec::default();

    loop {
        let dont_show_these_items = result.iter().map(|x| x.get_item()).collect();
        let selected = select_an_item(
            dont_show_these_items,
            SelectAnItemSortingOrder::MotivationsFirst,
            calculated_data,
        )
        .await
        .unwrap()
        .unwrap();

        result.push(selected);

        let select_another = Select::new(
            "Do you want to select another item?",
            vec![SelectAnother::SelectAnother, SelectAnother::Done],
        )
        .prompt()
        .unwrap();
        match select_another {
            SelectAnother::SelectAnother => {
                //do nothing, continue
            }
            SelectAnother::Done => {
                break;
            }
        }
    }
    result
}

impl Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Urgency::MoreUrgentThanAnythingIncludingScheduled => {
                write!(f, "🚨  More urgent than anything including scheduled")
            }
            Urgency::ScheduledAnyMode => {
                write!(f, "🗓️❗Schedule, to do, no matter your mode")
            }
            Urgency::MoreUrgentThanMode => {
                write!(f, "🔥  More urgent than your mode")
            }
            Urgency::InTheModeScheduled => write!(f, "🗓️⭳ When in the mode, scheduled"),
            Urgency::InTheModeDefinitelyUrgent => {
                write!(f, "🔴  When in the mode, definitely urgent")
            }
            Urgency::InTheModeMaybeUrgent => write!(f, "🟡  When in the mode, maybe urgent"),
            Urgency::InTheModeByImportance => write!(f, "🔝  Not immediately urgent"),
        }
    }
}

fn prompt_for_urgency() -> SurrealUrgency {
    let urgency = Select::new(
        "Select immediate urgency|",
        vec![
            Urgency::MoreUrgentThanAnythingIncludingScheduled,
            Urgency::ScheduledAnyMode,
            Urgency::MoreUrgentThanMode,
            Urgency::InTheModeScheduled,
            Urgency::InTheModeDefinitelyUrgent,
            Urgency::InTheModeMaybeUrgent,
            Urgency::InTheModeByImportance,
        ],
    )
    .with_starting_cursor(6)
    .prompt()
    .unwrap();
    match urgency {
        Urgency::MoreUrgentThanAnythingIncludingScheduled => {
            SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled
        }
        Urgency::ScheduledAnyMode => {
            SurrealUrgency::ScheduledAnyMode(prompt_to_schedule().unwrap().unwrap())
        }
        Urgency::MoreUrgentThanMode => SurrealUrgency::MoreUrgentThanMode,
        Urgency::InTheModeScheduled => {
            SurrealUrgency::InTheModeScheduled(prompt_to_schedule().unwrap().unwrap())
        }
        Urgency::InTheModeDefinitelyUrgent => SurrealUrgency::InTheModeDefinitelyUrgent,
        Urgency::InTheModeMaybeUrgent => SurrealUrgency::InTheModeMaybeUrgent,
        Urgency::InTheModeByImportance => SurrealUrgency::InTheModeByImportance,
    }
}

enum StartWhenOption {
    ExactTime,
    TimeRange,
}

impl Display for StartWhenOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartWhenOption::ExactTime => write!(f, "Exact Time"),
            StartWhenOption::TimeRange => write!(f, "Time Range"),
        }
    }
}

pub(crate) enum StartWhen {
    ExactTime(DateTime<Utc>),
    TimeRange(DateTime<Utc>, DateTime<Utc>),
}

fn prompt_to_schedule() -> Result<Option<SurrealScheduled>, ()> {
    let start_when = vec![StartWhenOption::ExactTime, StartWhenOption::TimeRange];
    let start_when = Select::new("When do you want to start this item?", start_when).prompt();
    let start_when = match start_when {
        Ok(StartWhenOption::ExactTime) => loop {
            let exact_start = Text::new("Enter the exact time you want to start this item")
                .prompt()
                .unwrap();
            let exact_start = match parse(&exact_start) {
                Ok(exact_start) => {
                    let now = Utc::now();
                    now + exact_start
                }
                Err(_) => match dateparser::parse(&exact_start) {
                    Ok(exact_start) => exact_start,
                    Err(_) => {
                        println!("Invalid date or duration, please try again");
                        continue;
                    }
                },
            };
            break StartWhen::ExactTime(exact_start);
        },
        Ok(StartWhenOption::TimeRange) => {
            let range_start = loop {
                let range_start = Text::new("Enter the start of the range").prompt().unwrap();
                let range_start = match parse(&range_start) {
                    Ok(range_start) => {
                        let now = Utc::now();
                        now + range_start
                    }
                    Err(_) => match dateparser::parse(&range_start) {
                        Ok(range_start) => range_start,
                        Err(_) => {
                            println!("Invalid date or duration, please try again");
                            continue;
                        }
                    },
                };
                break range_start;
            };
            let range_end = loop {
                let range_end = Text::new("Enter the end of the range").prompt().unwrap();
                let range_end = match parse(&range_end) {
                    Ok(range_end) => {
                        let now = Utc::now();
                        now + range_end
                    }
                    Err(_) => match dateparser::parse(&range_end) {
                        Ok(range_end) => range_end,
                        Err(_) => {
                            println!("Invalid date or duration, please try again");
                            continue;
                        }
                    },
                };
                break range_end;
            };
            StartWhen::TimeRange(range_start, range_end)
        }
        Err(InquireError::OperationCanceled) => {
            println!("Operation canceled");
            return Ok(None);
        }
        Err(InquireError::OperationInterrupted) => return Err(()),
        Err(err) => panic!("Unexpected error, try restarting the terminal: {}", err),
    };

    let time_boxed = Text::new("Time box how much time for this item")
        .prompt()
        .unwrap();
    let time_boxed = match parse(time_boxed) {
        Ok(time_boxed) => time_boxed,
        Err(e) => {
            todo!("Error: {:?}", e)
        }
    };

    let surreal_scheduled = match start_when {
        StartWhen::ExactTime(exact_start) => SurrealScheduled::Exact {
            start: exact_start.into(),
            duration: time_boxed.into(),
        },
        StartWhen::TimeRange(range_start, range_end) => SurrealScheduled::Range {
            start_range: (range_start.into(), range_end.into()),
            duration: time_boxed.into(),
        },
    };

    Ok(Some(surreal_scheduled))
}

pub(crate) async fn present_set_ready_and_urgency_plan_menu(
    selected: &ItemStatus<'_>,
    why_in_scope: &HashSet<WhyInScope>,
    current_urgency: Option<SurrealUrgency>,
    log_time: LogTime,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let start_present_set_ready_and_urgency_plan_menu = Utc::now();
    let (dependencies, urgency_plan) =
        prompt_for_dependencies_and_urgency_plan(Some(selected), send_to_data_storage_layer).await;

    for (command, dependency) in dependencies.into_iter() {
        match command {
            AddOrRemove::Add => {
                send_to_data_storage_layer
                    .send(DataLayerCommands::AddItemDependency(
                        selected.get_surreal_record_id().clone(),
                        dependency,
                    ))
                    .await
                    .unwrap();
            }
            AddOrRemove::Remove => {
                send_to_data_storage_layer
                    .send(DataLayerCommands::RemoveItemDependency(
                        selected.get_surreal_record_id().clone(),
                        dependency,
                    ))
                    .await
                    .unwrap();
            }
        }
    }

    send_to_data_storage_layer
        .send(DataLayerCommands::UpdateUrgencyPlan(
            selected.get_surreal_record_id().clone(),
            Some(urgency_plan),
        ))
        .await
        .unwrap();

    match log_time {
        LogTime::SeparateTaskLogTheTime => {
            let new_time_spent = NewTimeSpent {
                why_in_scope: why_in_scope.to_surreal(),
                working_on: vec![SurrealAction::SetReadyAndUrgency(
                    selected.get_surreal_record_id().clone(),
                )], //TODO: Should this also be logging onto all the parent items that this is making progress towards the goal?
                when_started: start_present_set_ready_and_urgency_plan_menu,
                when_stopped: Utc::now(),
                dedication: None,
                urgency: current_urgency,
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
