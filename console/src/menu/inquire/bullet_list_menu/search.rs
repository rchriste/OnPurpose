use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use inquire::{InquireError, Select};
use tokio::sync::mpsc::Sender;

use crate::{
    display::display_item_status::DisplayItemStatus,
    menu::inquire::bullet_list_menu::bullet_list_single_item::present_bullet_list_item_selected,
    node::{
        item_status::{ItemStatus, UrgencyPlanWithItemNode},
        Filter, IsTriggered,
    },
    surrealdb_layer::{data_layer_commands::DataLayerCommands, surreal_item::SurrealUrgency},
    systems::bullet_list::BulletList,
};

enum SearchMenuUrgencyItem<'e> {
    MoreUrgentThanAnythingIncludingScheduled {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    ScheduledAnyMode {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    MoreUrgentThanMode {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    InTheModeScheduled {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    InTheModeDefinitelyUrgent {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    InTheModeMaybeUrgent {
        ready: Vec<&'e ItemStatus<'e>>,
        not_ready: Vec<&'e ItemStatus<'e>>,
        coming_later: Vec<&'e ItemStatus<'e>>,
    },
    HighestImportance {
        ready_highest_importance: Vec<&'e ItemStatus<'e>>,
        when_ready_will_be_highest_importance: Vec<&'e ItemStatus<'e>>,
        nothing_is_ready: Vec<&'e ItemStatus<'e>>,
    },
    Item {
        item: &'e ItemStatus<'e>,
    },
}

impl Display for SearchMenuUrgencyItem<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(f, "More urgent than anything including scheduled (ready: {}, not ready: {}, coming later: {})", ready.len(), not_ready.len(), coming_later.len())
            }
            SearchMenuUrgencyItem::ScheduledAnyMode {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(
                    f,
                    "Scheduled any mode (ready: {}, not ready: {}, coming later: {})",
                    ready.len(),
                    not_ready.len(),
                    coming_later.len()
                )
            }
            SearchMenuUrgencyItem::MoreUrgentThanMode {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(
                    f,
                    "More urgent than mode (ready: {}, not ready: {}, coming later: {})",
                    ready.len(),
                    not_ready.len(),
                    coming_later.len()
                )
            }
            SearchMenuUrgencyItem::InTheModeScheduled {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(
                    f,
                    "In the mode, scheduled (ready: {}, not ready: {}, coming later: {})",
                    ready.len(),
                    not_ready.len(),
                    coming_later.len()
                )
            }
            SearchMenuUrgencyItem::InTheModeDefinitelyUrgent {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(
                    f,
                    "In the mode, definitely urgent (ready: {}, not ready: {}, coming later: {})",
                    ready.len(),
                    not_ready.len(),
                    coming_later.len()
                )
            }
            SearchMenuUrgencyItem::InTheModeMaybeUrgent {
                ready,
                not_ready,
                coming_later,
            } => {
                write!(
                    f,
                    "In the mode, maybe urgent (ready: {}, not ready: {}, coming later: {})",
                    ready.len(),
                    not_ready.len(),
                    coming_later.len()
                )
            }
            SearchMenuUrgencyItem::HighestImportance {
                when_ready_will_be_highest_importance,
                ready_highest_importance,
                nothing_is_ready,
            } => {
                write!(
                    f,
                    "Highest importance (ready highest importance: {}, when ready will be highest importance: {} nothing is ready: {})",
                    ready_highest_importance.len(),
                    when_ready_will_be_highest_importance.len(),
                    nothing_is_ready.len()
                )
            }
            SearchMenuUrgencyItem::Item { item } => {
                let display_item_status = DisplayItemStatus::new(item);
                write!(f, "{}", display_item_status)
            }
        }
    }
}

impl<'e> SearchMenuUrgencyItem<'e> {
    pub(crate) fn push_ready(&mut self, to_push: &'e ItemStatus<'e>) {
        match self {
            SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled { ready, .. }
            | SearchMenuUrgencyItem::ScheduledAnyMode { ready, .. }
            | SearchMenuUrgencyItem::MoreUrgentThanMode { ready, .. }
            | SearchMenuUrgencyItem::InTheModeScheduled { ready, .. }
            | SearchMenuUrgencyItem::InTheModeDefinitelyUrgent { ready, .. }
            | SearchMenuUrgencyItem::InTheModeMaybeUrgent { ready, .. }
            | SearchMenuUrgencyItem::HighestImportance {
                ready_highest_importance: ready,
                ..
            } => ready.push(to_push),
            SearchMenuUrgencyItem::Item { .. } => panic!("Programming error. Can't push onto item"),
        }
    }

    pub(crate) fn push_not_ready(&mut self, to_push: &'e ItemStatus<'e>) {
        match self {
            SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
                not_ready, ..
            }
            | SearchMenuUrgencyItem::ScheduledAnyMode { not_ready, .. }
            | SearchMenuUrgencyItem::MoreUrgentThanMode { not_ready, .. }
            | SearchMenuUrgencyItem::InTheModeScheduled { not_ready, .. }
            | SearchMenuUrgencyItem::InTheModeDefinitelyUrgent { not_ready, .. }
            | SearchMenuUrgencyItem::InTheModeMaybeUrgent { not_ready, .. }
            | SearchMenuUrgencyItem::HighestImportance {
                when_ready_will_be_highest_importance: not_ready,
                ..
            } => not_ready.push(to_push),
            SearchMenuUrgencyItem::Item { .. } => panic!("Programming error. Can't push onto item"),
        }
    }

    pub(crate) fn push_coming_later(&mut self, to_push: &'e ItemStatus<'e>) {
        match self {
            SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
                coming_later,
                ..
            }
            | SearchMenuUrgencyItem::ScheduledAnyMode { coming_later, .. }
            | SearchMenuUrgencyItem::MoreUrgentThanMode { coming_later, .. }
            | SearchMenuUrgencyItem::InTheModeScheduled { coming_later, .. }
            | SearchMenuUrgencyItem::InTheModeDefinitelyUrgent { coming_later, .. }
            | SearchMenuUrgencyItem::InTheModeMaybeUrgent { coming_later, .. } => {
                coming_later.push(to_push)
            }
            SearchMenuUrgencyItem::HighestImportance { nothing_is_ready, ..  } => {
                nothing_is_ready.push(to_push)
            }
            SearchMenuUrgencyItem::Item { .. } => panic!("Programming error. Can't push onto item"),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::ScheduledAnyMode {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::MoreUrgentThanMode {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::InTheModeScheduled {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::InTheModeDefinitelyUrgent {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::InTheModeMaybeUrgent {
                ready,
                not_ready,
                coming_later,
            }
            | SearchMenuUrgencyItem::HighestImportance {
                when_ready_will_be_highest_importance: not_ready,
                ready_highest_importance: ready,
                nothing_is_ready: coming_later,
            } => ready.is_empty() && not_ready.is_empty() && coming_later.is_empty(),
            SearchMenuUrgencyItem::Item { item: _item } => false,
        }
    }
}

enum UrgencyDrillDownOption<'e> {
    Ready(Vec<&'e ItemStatus<'e>>),
    NotReady(Vec<&'e ItemStatus<'e>>),
    ComingLater(Vec<&'e ItemStatus<'e>>),
}

impl Display for UrgencyDrillDownOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UrgencyDrillDownOption::Ready(items) => {
                write!(f, "Ready ({} items)", items.len())
            }
            UrgencyDrillDownOption::NotReady(items) => {
                write!(f, "Not ready ({} items)", items.len())
            }
            UrgencyDrillDownOption::ComingLater(items) => {
                write!(f, "Coming later ({} items)", items.len())
            }
        }
    }
}

impl<'e> UrgencyDrillDownOption<'e> {
    pub(crate) fn unwrap(self) -> Vec<&'e ItemStatus<'e>> {
        match self {
            UrgencyDrillDownOption::Ready(items) => items,
            UrgencyDrillDownOption::NotReady(items) => items,
            UrgencyDrillDownOption::ComingLater(items) => items,
        }
    }
}

enum HighestImportanceDrillDownOption<'e> {
    ReadyHighestImportance(Vec<&'e ItemStatus<'e>>),
    WhenReadyWillBeHighestImportance(Vec<&'e ItemStatus<'e>>),
    NothingIsReady(Vec<&'e ItemStatus<'e>>),
}

impl Display for HighestImportanceDrillDownOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HighestImportanceDrillDownOption::ReadyHighestImportance(items) => {
                write!(f, "Ready highest importance ({} items)", items.len())
            }
            HighestImportanceDrillDownOption::WhenReadyWillBeHighestImportance(items) => {
                write!(
                    f,
                    "When ready will be highest importance ({} items)",
                    items.len()
                )
            }
            HighestImportanceDrillDownOption::NothingIsReady(items) => {
                write!(f, "Nothing is ready ({} items)", items.len())
            }
        }
    }
}

impl<'e> HighestImportanceDrillDownOption<'e> {
    pub(crate) fn unwrap(self) -> Vec<&'e ItemStatus<'e>> {
        match self {
            HighestImportanceDrillDownOption::ReadyHighestImportance(items) => items,
            HighestImportanceDrillDownOption::WhenReadyWillBeHighestImportance(items) => items,
            HighestImportanceDrillDownOption::NothingIsReady(items) => items,
        }
    }
}

enum UrgencyChanges<'e> {
    NotSet,
    AtFinalValue(&'e SurrealUrgency),
    WillChange {
        now: &'e SurrealUrgency,
        later: &'e SurrealUrgency,
    },
}

pub(crate) async fn present_search_menu(
    bullet_list: &BulletList,
    send_to_data_storage_layer: &Sender<DataLayerCommands>,
) -> Result<(), ()> {
    let items = bullet_list.get_all_items_status();

    let mut more_urgent_than_anything_including_scheduled =
        SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
            ready: Vec::default(),
            not_ready: Vec::default(),
            coming_later: Vec::default(),
        };
    let mut scheduled_any_mode = SearchMenuUrgencyItem::ScheduledAnyMode {
        ready: Vec::default(),
        not_ready: Vec::default(),
        coming_later: Vec::default(),
    };
    let mut more_urgent_than_mode = SearchMenuUrgencyItem::MoreUrgentThanMode {
        ready: Vec::default(),
        not_ready: Vec::default(),
        coming_later: Vec::default(),
    };
    let mut in_the_mode_scheduled = SearchMenuUrgencyItem::InTheModeScheduled {
        ready: Vec::default(),
        not_ready: Vec::default(),
        coming_later: Vec::default(),
    };
    let mut in_the_mode_definitely_urgent = SearchMenuUrgencyItem::InTheModeDefinitelyUrgent {
        ready: Vec::default(),
        not_ready: Vec::default(),
        coming_later: Vec::default(),
    };
    let mut in_the_mode_maybe_urgent = SearchMenuUrgencyItem::InTheModeMaybeUrgent {
        ready: Vec::default(),
        not_ready: Vec::default(),
        coming_later: Vec::default(),
    };

    for item in items {
        let urgency_plan = item.get_urgency_plan();
        let urgency_changes = match urgency_plan {
            Some(UrgencyPlanWithItemNode::StaysTheSame(urgency)) => {
                UrgencyChanges::AtFinalValue(urgency)
            }
            Some(UrgencyPlanWithItemNode::WillEscalate {
                initial,
                triggers,
                later,
            }) => {
                if triggers.is_triggered() {
                    UrgencyChanges::AtFinalValue(later)
                } else {
                    UrgencyChanges::WillChange {
                        now: initial,
                        later,
                    }
                }
            }
            None => UrgencyChanges::NotSet,
        };

        match urgency_changes {
            UrgencyChanges::AtFinalValue(urgency) => {
                match urgency {
                    SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => {
                        if item.has_dependencies(Filter::Active) {
                            more_urgent_than_anything_including_scheduled.push_not_ready(item);
                        } else {
                            more_urgent_than_anything_including_scheduled.push_ready(item);
                        }
                    }
                    SurrealUrgency::ScheduledAnyMode(..) => {
                        //has_dependencies is true if a scheduled item has not started yet
                        if item.has_dependencies(Filter::Active) {
                            scheduled_any_mode.push_not_ready(item);
                        } else {
                            scheduled_any_mode.push_ready(item);
                        }
                    }
                    SurrealUrgency::MoreUrgentThanMode => {
                        if item.has_dependencies(Filter::Active) {
                            more_urgent_than_mode.push_not_ready(item);
                        } else {
                            more_urgent_than_mode.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeScheduled(..) => {
                        //has_dependencies is true if a scheduled item has not started yet
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_scheduled.push_not_ready(item);
                        } else {
                            in_the_mode_scheduled.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeDefinitelyUrgent => {
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_definitely_urgent.push_not_ready(item);
                        } else {
                            in_the_mode_definitely_urgent.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeMaybeUrgent => {
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_maybe_urgent.push_not_ready(item);
                        } else {
                            in_the_mode_maybe_urgent.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeByImportance => {
                        //Nothing is concerning about this urgency level so we don't need to surface it, nothing to do
                    }
                }
            }
            UrgencyChanges::WillChange { now, later } => {
                match now {
                    SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => {
                        if item.has_dependencies(Filter::Active) {
                            more_urgent_than_anything_including_scheduled.push_not_ready(item);
                        } else {
                            more_urgent_than_anything_including_scheduled.push_ready(item);
                        }
                    }
                    SurrealUrgency::ScheduledAnyMode(..) => {
                        //has_dependencies is true if a scheduled item has not started yet
                        if item.has_dependencies(Filter::Active) {
                            scheduled_any_mode.push_not_ready(item);
                        } else {
                            scheduled_any_mode.push_ready(item);
                        }
                    }
                    SurrealUrgency::MoreUrgentThanMode => {
                        if item.has_dependencies(Filter::Active) {
                            more_urgent_than_mode.push_not_ready(item);
                        } else {
                            more_urgent_than_mode.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeScheduled(..) => {
                        //has_dependencies is true if a scheduled item has not started yet
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_scheduled.push_not_ready(item);
                        } else {
                            in_the_mode_scheduled.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeDefinitelyUrgent => {
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_definitely_urgent.push_not_ready(item);
                        } else {
                            in_the_mode_definitely_urgent.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeMaybeUrgent => {
                        if item.has_dependencies(Filter::Active) {
                            in_the_mode_maybe_urgent.push_not_ready(item);
                        } else {
                            in_the_mode_maybe_urgent.push_ready(item);
                        }
                    }
                    SurrealUrgency::InTheModeByImportance => {
                        //Nothing is concerning about this urgency level so we don't need to surface it, nothing to do
                    }
                }
                match later {
                    SurrealUrgency::MoreUrgentThanAnythingIncludingScheduled => {
                        more_urgent_than_anything_including_scheduled.push_coming_later(item)
                    }
                    SurrealUrgency::ScheduledAnyMode(_) => {
                        scheduled_any_mode.push_coming_later(item)
                    }
                    SurrealUrgency::MoreUrgentThanMode => {
                        more_urgent_than_mode.push_coming_later(item)
                    }
                    SurrealUrgency::InTheModeScheduled(_) => {
                        in_the_mode_scheduled.push_coming_later(item)
                    }
                    SurrealUrgency::InTheModeDefinitelyUrgent => {
                        in_the_mode_definitely_urgent.push_coming_later(item)
                    }
                    SurrealUrgency::InTheModeMaybeUrgent => {
                        in_the_mode_maybe_urgent.push_coming_later(item)
                    }
                    SurrealUrgency::InTheModeByImportance => {
                        //Nothing is concerning about this urgency level so we don't need to surface it, nothing to do
                    }
                }
            }
            UrgencyChanges::NotSet => {}
        }
    }

    let everything_that_has_no_parent = bullet_list
        .get_all_items_status()
        .iter()
        .filter(|x| !x.has_parents(Filter::Active) && x.is_active())
        .collect::<Vec<_>>();

    let mut highest_importance = SearchMenuUrgencyItem::HighestImportance {
        ready_highest_importance: Vec::default(),
        when_ready_will_be_highest_importance: Vec::default(),
        nothing_is_ready: Vec::default(),
    };

    for no_parent in everything_that_has_no_parent {
        let most_important_and_blocked =
            no_parent.recursive_get_most_important_both_ready_and_blocked(items, Vec::default());
        match most_important_and_blocked.ready {
            Some(most_important) => {
                highest_importance.push_ready(most_important);
            }
            None => {
                highest_importance.push_coming_later(no_parent);
            }
        }

        for blocked in most_important_and_blocked.blocked {
            highest_importance.push_not_ready(blocked);
        }
    }

    let mut list = Vec::default();
    if !more_urgent_than_anything_including_scheduled.is_empty() {
        list.push(more_urgent_than_anything_including_scheduled);
    }
    if !scheduled_any_mode.is_empty() {
        list.push(scheduled_any_mode);
    }
    if !more_urgent_than_mode.is_empty() {
        list.push(more_urgent_than_mode);
    }
    if !in_the_mode_scheduled.is_empty() {
        list.push(in_the_mode_scheduled);
    }
    if !in_the_mode_definitely_urgent.is_empty() {
        list.push(in_the_mode_definitely_urgent);
    }
    if !in_the_mode_maybe_urgent.is_empty() {
        list.push(in_the_mode_maybe_urgent);
    }
    if !highest_importance.is_empty() {
        list.push(highest_importance);
    }

    for item in items {
        list.push(SearchMenuUrgencyItem::Item { item });
    }

    let selection = Select::new("Select an item to view", list).prompt();

    match selection {
        Ok(SearchMenuUrgencyItem::MoreUrgentThanAnythingIncludingScheduled {
            ready,
            not_ready,
            coming_later,
        })
        | Ok(SearchMenuUrgencyItem::ScheduledAnyMode {
            ready,
            not_ready,
            coming_later,
        })
        | Ok(SearchMenuUrgencyItem::MoreUrgentThanMode {
            ready,
            not_ready,
            coming_later,
        })
        | Ok(SearchMenuUrgencyItem::InTheModeScheduled {
            ready,
            not_ready,
            coming_later,
        })
        | Ok(SearchMenuUrgencyItem::InTheModeDefinitelyUrgent {
            ready,
            not_ready,
            coming_later,
        })
        | Ok(SearchMenuUrgencyItem::InTheModeMaybeUrgent {
            ready,
            not_ready,
            coming_later,
        }) => {
            let mut list = Vec::new();
            if !ready.is_empty() {
                list.push(UrgencyDrillDownOption::Ready(ready));
            }
            if !not_ready.is_empty() {
                list.push(UrgencyDrillDownOption::NotReady(not_ready));
            }
            if !coming_later.is_empty() {
                list.push(UrgencyDrillDownOption::ComingLater(coming_later));
            }
            let selection = if list.is_empty() {
                panic!("Programming error. Ready should not be empty")
            } else if list.len() <= 1 {
                list.into_iter()
                    .next()
                    .expect("len() <= 1 so first() should be Some")
            } else {
                Select::new("Select an item to view", list)
                    .prompt()
                    .unwrap()
            };
            let list = selection
                .unwrap()
                .into_iter()
                .map(|x| SearchMenuUrgencyItem::Item { item: x })
                .collect::<Vec<_>>();

            let selection = Select::new("Select an item to view", list)
                .prompt()
                .unwrap();
            match selection {
                SearchMenuUrgencyItem::Item { item } => {
                    present_bullet_list_item_selected(
                        item,
                        Utc::now(),
                        bullet_list,
                        send_to_data_storage_layer,
                    )
                    .await
                }
                _ => panic!("Programming error. Expected item"),
            }
        }
        Ok(SearchMenuUrgencyItem::HighestImportance {
            ready_highest_importance,
            when_ready_will_be_highest_importance,
            nothing_is_ready,
        }) => {
            let mut list = Vec::new();
            if !ready_highest_importance.is_empty() {
                list.push(HighestImportanceDrillDownOption::ReadyHighestImportance(
                    ready_highest_importance,
                ));
            }
            if !when_ready_will_be_highest_importance.is_empty() {
                list.push(
                    HighestImportanceDrillDownOption::WhenReadyWillBeHighestImportance(
                        when_ready_will_be_highest_importance,
                    ),
                );
            }
            if !nothing_is_ready.is_empty() {
                list.push(HighestImportanceDrillDownOption::NothingIsReady(
                    nothing_is_ready,
                ));
            }
            let selection = if list.is_empty() {
                panic!("Programming error. Ready should not be empty")
            } else if list.len() <= 1 {
                list.into_iter()
                    .next()
                    .expect("len() <= 1 so first() should be Some")
            } else {
                Select::new("Select an item to view", list)
                    .prompt()
                    .unwrap()
            };
            let list = selection
                .unwrap()
                .into_iter()
                .map(|x| SearchMenuUrgencyItem::Item { item: x })
                .collect::<Vec<_>>();

            let selection = Select::new("Select an item to view", list)
                .prompt()
                .unwrap();

            match selection {
                SearchMenuUrgencyItem::Item { item } => {
                    present_bullet_list_item_selected(
                        item,
                        Utc::now(),
                        bullet_list,
                        send_to_data_storage_layer,
                    )
                    .await
                }
                _ => panic!("Programming error. Expected item"),
            }
        }
        Ok(SearchMenuUrgencyItem::Item { item }) => {
            present_bullet_list_item_selected(
                item,
                Utc::now(),
                bullet_list,
                send_to_data_storage_layer,
            )
            .await
        }
        Err(InquireError::OperationCanceled) => Ok(()),
        Err(InquireError::OperationInterrupted) => Err(()),
        Err(InquireError::InvalidConfiguration(_)) => {
            println!();
            println!("There are no items to search. Capture an item first.");
            println!();
            Ok(())
        }
        Err(err) => {
            todo!("Handle error: {:?}", err);
        }
    }
}
