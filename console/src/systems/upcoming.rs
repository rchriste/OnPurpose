pub(crate) mod scheduled_item;

use std::time::Duration;

use chrono::{DateTime, Utc};
use scheduled_item::{Scheduled, ScheduledItem};

use crate::{
    base_data::item::Item, calculated_data::CalculatedData,
    surrealdb_layer::surreal_item::SurrealScheduled,
};

pub(crate) struct Upcoming<'s> {
    order: Order<'s>,
}

pub(crate) enum Conflict<'e> {
    NoConflict,
    Conflict(Vec<Item<'e>>),
}

impl<'s> Upcoming<'s> {
    pub(crate) fn new(
        calculated_data: &'s CalculatedData,
        earliest_starting_time: &DateTime<Utc>,
    ) -> Self {
        let items = calculated_data
            .get_items_highest_lap_count()
            .iter()
            .filter_map(|x| {
                if x.is_scheduled() && x.is_active() {
                    Some(x.get_item())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let order = find_a_valid_order(&items, earliest_starting_time, Vec::default());
        Self { order }
    }

    pub(crate) fn get_ordered_scheduled_items(&self) -> &Option<Vec<ScheduledItem<'s>>> {
        &self.order.sorted_best_order
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.order.sorted_best_order.is_none()
    }
}

#[derive(Default)]
struct Order<'s> {
    sorted_best_order: Option<Vec<ScheduledItem<'s>>>,
    gap_penalty: f64,
    big_to_little_count: u32,
    conflicts: Vec<&'s Item<'s>>,
}

impl<'s> Order<'s> {
    fn new(mut order: Vec<ScheduledItem<'s>>) -> Self {
        order.sort_by(|a, b| a.get_scheduled_start().cmp(b.get_scheduled_start()));
        let gap_penalty = order.calculate_gap_penalty();
        let big_to_little_count = order.calculate_big_to_little_count();
        Self {
            sorted_best_order: Some(order),
            gap_penalty,
            big_to_little_count,
            conflicts: Vec::default(),
        }
    }

    fn is_none(&self) -> bool {
        self.sorted_best_order.is_none()
    }

    fn add_conflict_if_new(&mut self, item: &'s Item<'s>) {
        if !self.conflicts.contains(&item) {
            self.conflicts.push(item);
        }
    }

    fn add_conflicts_if_new(&mut self, other: &Order<'s>) {
        for conflict in other.conflicts.iter() {
            self.add_conflict_if_new(conflict);
        }
    }

    fn keep_best_order(&mut self, proposal: Order<'s>) {
        if self.sorted_best_order.is_none() {
            self.take_proposal(proposal);
        } else if proposal.sorted_best_order.is_none() {
            //Do nothing
        } else if (proposal.gap_penalty - self.gap_penalty).abs() < 0.1 {
            if proposal.big_to_little_count < self.big_to_little_count {
                self.take_proposal(proposal);
            }
        } else if proposal.gap_penalty < self.gap_penalty {
            self.take_proposal(proposal);
        }
    }

    fn take_proposal(&mut self, proposal: Order<'s>) {
        self.sorted_best_order = proposal.sorted_best_order;
        self.gap_penalty = proposal.gap_penalty;
        self.big_to_little_count = proposal.big_to_little_count;
    }
}

fn find_a_valid_order<'s>(
    items: &[&'s Item<'s>],
    earliest_starting_time: &DateTime<Utc>,
    scheduled: Vec<ScheduledItem<'s>>,
) -> Order<'s> {
    //Go through each item as the next item and see if it fits, this is a brute force algorithm. Scheduled items are dealt
    //with right away so I don't expect there to be too many of them, hence the brute force approach.

    let mut result = Order::default();
    for item in items {
        if scheduled.contains(item) {
            continue;
        }
        let mut scheduled = scheduled.clone();
        let to_schedule = item.get_scheduled();
        let (mut start, duration): (DateTime<Utc>, Duration) = match to_schedule {
            SurrealScheduled::ScheduledExact {
                start, duration, ..
            } => (start.clone().into(), (*duration).into()),
            SurrealScheduled::ScheduledRange {
                start_range,
                duration,
                ..
            } => (start_range.0.clone().into(), (*duration).into()),
            SurrealScheduled::NotScheduled => panic!(
                "We should only be dealing with scheduled items, this code should never execute"
            ),
        };
        //I'm looking for the earliest available time that fits to schedule this item
        if earliest_starting_time > &start {
            start = *earliest_starting_time;
        }
        let next_available = scheduled.find_next_available_time(start, start + duration);
        if to_schedule.is_this_a_valid_starting_time(next_available) {
            scheduled.push(ScheduledItem::new(
                item,
                next_available,
                next_available + duration,
            ));
            if scheduled.len() == items.len() {
                //We have scheduled all of the items
                let new_ordering = Order::new(scheduled);
                result.keep_best_order(new_ordering);
            } else {
                //We have scheduled this item, now we need to schedule the rest of the items
                let valid_order = find_a_valid_order(items, earliest_starting_time, scheduled);
                result.add_conflicts_if_new(&valid_order);
                if valid_order.is_none() {
                    //This ordering won't work
                } else {
                    result.keep_best_order(valid_order);
                }
            }
        } else {
            //This ordering won't work
            result.add_conflict_if_new(item);
        }
    }
    result
}

impl SurrealScheduled {
    fn is_this_a_valid_starting_time(&self, proposed: DateTime<Utc>) -> bool {
        match self {
            SurrealScheduled::ScheduledExact {
                start: scheduled_start,
                ..
            } => proposed == scheduled_start.clone().into(),
            SurrealScheduled::ScheduledRange {
                start_range: (start, end),
                ..
            } => proposed >= start.clone().into() && proposed <= end.clone().into(),
            SurrealScheduled::NotScheduled => false,
        }
    }
}
