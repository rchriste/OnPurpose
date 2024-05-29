use std::{cmp::Ordering, ops::Sub, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

use crate::{
    base_data::{item::Item, time_spent::TimeSpent},
    surrealdb_layer::surreal_item::{
        EnterListReason, EqF32, InRelationToRatioType, ItemType, SurrealLap, SurrealStaging,
    },
};

use super::{
    item_node::{GrowingItemNode, ItemNode, ShrinkingItemNode},
    Filter,
};

#[derive(Clone, Debug)]
pub(crate) struct ItemStatus<'s> {
    item_node: ItemNode<'s>,
    lap_count: LapCount,
    is_snoozed: bool,
}

impl PartialEq for ItemStatus<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item_node == other.item_node
    }
}

impl<'s> ItemStatus<'s> {
    pub(crate) fn new(
        item_node: ItemNode<'s>,
        all_nodes: &[ItemNode<'_>],
        time_spent_log: &[TimeSpent<'_>],
        current_date_time: &DateTime<Utc>,
    ) -> Self {
        let mut lap_count =
            calculate_lap_count(&item_node, all_nodes, time_spent_log, current_date_time);
        let is_snoozed = calculate_is_snoozed(&item_node, all_nodes, current_date_time);
        if is_snoozed {
            lap_count = LapCount::F32(0.0);
        }
        Self {
            item_node,
            lap_count,
            is_snoozed,
        }
    }

    pub(crate) fn get_lap_count(&'s self) -> &'s LapCount {
        &self.lap_count
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.is_snoozed
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.item_node.is_finished()
    }

    pub(crate) fn get_item_node(&'s self) -> &'s ItemNode<'s> {
        &self.item_node
    }

    pub(crate) fn get_staging(&self) -> &SurrealStaging {
        self.item_node.get_staging()
    }

    pub(crate) fn is_staging_not_set(&self) -> bool {
        self.item_node.is_staging_not_set()
    }

    pub(crate) fn get_thing(&self) -> &Thing {
        self.item_node.get_thing()
    }

    pub(crate) fn is_responsibility_reactive(&self) -> bool {
        self.item_node.is_responsibility_reactive()
    }

    pub(crate) fn is_type_undeclared(&self) -> bool {
        self.item_node.is_type_undeclared()
    }

    pub(crate) fn is_type_motivation(&self) -> bool {
        self.item_node.is_type_motivation()
    }

    pub(crate) fn is_staging_mentally_resident(&self) -> bool {
        self.item_node.is_staging_mentally_resident()
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_node.is_person_or_group()
    }

    pub(crate) fn get_item(&self) -> &Item<'s> {
        self.item_node.get_item()
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item_node.get_summary()
    }

    pub(crate) fn has_children(&self, filter: Filter) -> bool {
        self.item_node.has_children(filter)
    }

    pub(crate) fn get_smaller(
        &'s self,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = &ShrinkingItemNode<'s>> + 's> {
        self.item_node.get_smaller(filter)
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item_node.get_type()
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item_node.get_surreal_record_id()
    }

    pub(crate) fn has_larger(&self, filter: Filter) -> bool {
        self.item_node.has_larger(filter)
    }

    pub(crate) fn get_larger(
        &'s self,
        filter: Filter,
    ) -> impl Iterator<Item = &GrowingItemNode<'s>> {
        self.item_node.get_larger(filter)
    }

    pub(crate) fn get_self_and_larger_flattened(&'s self, filter: Filter) -> Vec<&'s Item<'s>> {
        self.item_node.get_self_and_larger(filter)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.item_node.is_active()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LapCountGreaterOrLess {
    GreaterThan,
    LessThan,
}

impl From<TimeDelta> for LapCountGreaterOrLess {
    fn from(time_delta: TimeDelta) -> Self {
        if time_delta > TimeDelta::zero() {
            LapCountGreaterOrLess::GreaterThan
        } else {
            LapCountGreaterOrLess::LessThan
        }
    }
}

impl From<EqF32> for LapCountGreaterOrLess {
    fn from(eq_f32: EqF32) -> Self {
        if eq_f32 > 0.0 {
            LapCountGreaterOrLess::GreaterThan
        } else {
            LapCountGreaterOrLess::LessThan
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LapCount {
    F32(f32),
    Ratio {
        other_item: RecordId,
        greater_or_less: LapCountGreaterOrLess,
    },
    MaxOf(Vec<RecordId>),
}

impl PartialOrd for LapCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (LapCount::F32(a), LapCount::F32(b)) => a.partial_cmp(b),
            (LapCount::MaxOf(..), _) => todo!(),
            (_, LapCount::MaxOf(..)) => todo!(),
            (LapCount::Ratio { .. }, LapCount::F32(_)) => todo!(),
            (LapCount::F32(_), LapCount::Ratio { .. }) => todo!(),
            (
                LapCount::Ratio {
                    other_item: a,
                    greater_or_less: a_g,
                },
                LapCount::Ratio {
                    other_item: b,
                    greater_or_less: b_g,
                },
            ) => {
                if a == b {
                    match (a_g, b_g) {
                        (LapCountGreaterOrLess::GreaterThan, LapCountGreaterOrLess::LessThan) => {
                            Some(Ordering::Greater)
                        }
                        (LapCountGreaterOrLess::LessThan, LapCountGreaterOrLess::GreaterThan) => {
                            Some(Ordering::Less)
                        }
                        _ => Some(Ordering::Equal),
                    }
                } else {
                    todo!()
                }
            }
        }
    }
}

impl LapCount {
    pub(crate) fn resolve(&self, all_status: &[ItemStatus<'_>]) -> f32 {
        self.resolve_internal(all_status, Vec::default())
    }

    fn resolve_internal<'a>(
        &self,
        all_status: &'a [ItemStatus<'a>],
        mut call_chain: Vec<&'a ItemStatus<'a>>,
    ) -> f32 {
        match self {
            LapCount::F32(float) => *float,
            LapCount::Ratio {
                other_item,
                greater_or_less,
            } => {
                match greater_or_less {
                    LapCountGreaterOrLess::GreaterThan => {
                        let other_item = all_status
                            .iter()
                            .find(|x| x.get_surreal_record_id() == other_item)
                            .expect("other_item should be in all_status");
                        if call_chain.contains(&other_item) {
                            todo!("We have a loop, what I want to do is something sensible here but I'm not sure what that is so put in this to do as a placeholder")
                        } else {
                            let other_item_lap_count = other_item.get_lap_count();
                            call_chain.push(other_item);
                            other_item_lap_count.resolve_internal(all_status, call_chain) * 1.1
                            //Have a lap count 10% higher
                        }
                    }
                    LapCountGreaterOrLess::LessThan => 0.0,
                }
            }
            LapCount::MaxOf(record_ids) => {
                let max = record_ids
                    .iter()
                    .map(|record_id| {
                        let item_status = all_status
                            .iter()
                            .find(|x| x.get_surreal_record_id() == record_id)
                            .expect("record_id should be in all_status");
                        if call_chain.contains(&item_status) {
                            todo!("We have a loop, what I want to do is something sensible here but I'm not sure what that is so put in this to do as a placeholder")
                        } else {
                            let mut call_chain = call_chain.clone();
                            call_chain.push(item_status);
                            let lap_count = item_status.get_lap_count();
                            lap_count.resolve_internal(all_status, call_chain)
                        }
                    })
                    .max_by(|a, b| a.partial_cmp(b).expect("Should be able to compare"));
                match max {
                    Some(max) => max,
                    None => todo!(
                        "Number of items should be at least one not {}",
                        record_ids.len()
                    ),
                }
            }
        }
    }
}

fn calculate_lap_count(
    item_node: &ItemNode<'_>,
    all_nodes: &[ItemNode<'_>],
    time_spent_log: &[TimeSpent<'_>],
    current_date_time: &DateTime<Utc>,
) -> LapCount {
    match item_node.get_staging() {
        SurrealStaging::InRelationTo {
            start,
            other_item,
            ratio,
        } => {
            let start: DateTime<Utc> = start.clone().into();
            let time_spent_on_other_item = time_spent_log.iter().filter(|x| {
                x.get_started_at() > &start && x.worked_towards().iter().any(|y| y == other_item)
            });

            let this_item = item_node.get_surreal_record_id();
            let time_spent_on_this_item = time_spent_log.iter().filter(|x| {
                x.get_started_at() > &start && x.worked_towards().iter().any(|y| y == this_item)
            });

            match ratio {
                InRelationToRatioType::AmountOfTimeSpent { multiplier } => {
                    let time_spent_on_other_item = time_spent_on_other_item
                        .map(|x| x.get_time_delta())
                        .sum::<TimeDelta>();
                    let time_spent_on_this_item = time_spent_on_this_item
                        .map(|x| x.get_time_delta())
                        .sum::<TimeDelta>();

                    let allowance = time_spent_on_other_item * multiplier;
                    let remaining = time_spent_on_this_item - allowance;
                    LapCount::Ratio {
                        other_item: other_item.clone(),
                        greater_or_less: remaining.into(),
                    }
                }
                InRelationToRatioType::IterationCount { multiplier } => {
                    let count_on_other_item = time_spent_on_other_item.count() as f32;
                    let count_on_this_item = time_spent_on_this_item.count() as f32;

                    let allowance = count_on_other_item * multiplier;
                    let remaining = count_on_this_item - allowance;

                    LapCount::Ratio {
                        other_item: other_item.clone(),
                        greater_or_less: remaining.into(),
                    }
                }
            }
        }
        SurrealStaging::NotSet => LapCount::F32(0.0),
        SurrealStaging::OnDeck { enter_list, lap }
        | SurrealStaging::MentallyResident { enter_list, lap } => {
            match enter_list {
                EnterListReason::DateTime(enter_time) => {
                    let enter_time: DateTime<Utc> = enter_time.clone().into();
                    match lap {
                        SurrealLap::AlwaysTimer(lap) => {
                            let lap: Duration = (*lap).into();
                            let elapsed = current_date_time.sub(enter_time);
                            let elapsed = elapsed.num_seconds() as f32;
                            let lap = lap.as_secs_f32();
                            LapCount::F32(elapsed / lap)
                        }
                        SurrealLap::LoggedTimer(lap) => {
                            let logged_worked =
                                get_worked_on_since_elapsed(enter_time, time_spent_log);
                            let elapsed = logged_worked.num_seconds() as f32;
                            let lap = lap.as_secs_f32();
                            LapCount::F32(elapsed / lap)
                        }
                        SurrealLap::WorkedOnCounter { stride } => {
                            let worked_on_since = get_worked_on_since(enter_time, time_spent_log);
                            let stride: f32 = *stride as f32;
                            LapCount::F32(1.0 / stride * worked_on_since)
                        }
                        SurrealLap::InherentFromParent => {
                            let parents = item_node
                                .get_larger(Filter::Active)
                                .map(|x| x.get_surreal_record_id().clone())
                                .collect::<Vec<_>>();
                            LapCount::MaxOf(parents)
                        }
                    }
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after,
                } => {
                    if current_date_time < earliest {
                        LapCount::F32(0.0)
                    } else if current_date_time > review_after {
                        let enter_time: DateTime<Utc> = review_after.clone().into();
                        match lap {
                            SurrealLap::AlwaysTimer(lap) => {
                                let lap: Duration = (*lap).into();
                                let elapsed = current_date_time.sub(enter_time);
                                let elapsed = elapsed.num_seconds() as f32;
                                let lap = lap.as_secs_f32();
                                LapCount::F32(elapsed / lap)
                            }
                            SurrealLap::LoggedTimer(lap) => {
                                let logged_worked =
                                    get_worked_on_since_elapsed(enter_time, time_spent_log);
                                let elapsed = logged_worked.num_seconds() as f32;
                                let lap = lap.as_secs_f32();
                                LapCount::F32(elapsed / lap)
                            }
                            SurrealLap::WorkedOnCounter { stride } => {
                                let worked_on_since =
                                    get_worked_on_since(enter_time, time_spent_log);
                                let stride: f32 = *stride as f32;
                                LapCount::F32(1.0 / stride * worked_on_since)
                            }
                            SurrealLap::InherentFromParent => {
                                let parents = item_node
                                    .get_larger(Filter::Active)
                                    .map(|x| x.get_surreal_record_id().clone())
                                    .collect::<Vec<_>>();
                                LapCount::MaxOf(parents)
                            }
                        }
                    } else {
                        let all_larger = item_node.get_larger(Filter::All);
                        let all_larger = all_larger
                            .map(|x| x.get_node(all_nodes))
                            .collect::<Vec<_>>();
                        let mut all_larger_iter = all_larger.iter();
                        let (highest_uncovered, uncovered_when) = loop {
                            let larger = all_larger_iter.next();
                            match larger {
                                Some(larger) => {
                                    let (highest_uncovered, uncovered_when) =
                                        find_highest_uncovered_child_with_when_uncovered(
                                            larger,
                                            current_date_time,
                                            Vec::default(),
                                        );
                                    if let Some(highest_uncovered) = highest_uncovered {
                                        break (Some(highest_uncovered), uncovered_when);
                                    }
                                }
                                None => break (None, None),
                            }
                        };
                        match highest_uncovered {
                            Some(highest_uncovered) => {
                                let item = item_node.get_item();
                                if highest_uncovered == item {
                                    let mut uncovered_when: DateTime<Utc> =
                                        uncovered_when.unwrap_or_else(|| earliest.clone().into());
                                    if &uncovered_when < item.get_created() {
                                        // If the uncovered_when is before the item was created, then we'll just use the item's created time
                                        uncovered_when = *item.get_created();
                                    }
                                    let elapsed = current_date_time.sub(uncovered_when);
                                    let elapsed = elapsed.num_seconds() as f32;
                                    match lap {
                                        SurrealLap::AlwaysTimer(lap) => {
                                            let lap = lap.as_secs_f32();
                                            LapCount::F32(elapsed / lap)
                                        }
                                        SurrealLap::LoggedTimer(lap) => {
                                            let logged_worked = get_worked_on_since_elapsed(
                                                uncovered_when,
                                                time_spent_log,
                                            );
                                            let elapsed = logged_worked.num_seconds() as f32;
                                            let lap = lap.as_secs_f32();
                                            LapCount::F32(elapsed / lap)
                                        }
                                        SurrealLap::WorkedOnCounter { stride } => {
                                            let worked_on_since =
                                                get_worked_on_since(uncovered_when, time_spent_log);
                                            let stride: f32 = *stride as f32;
                                            LapCount::F32(1.0 / stride * worked_on_since)
                                        }
                                        SurrealLap::InherentFromParent => {
                                            let parents = item_node
                                                .get_larger(Filter::Active)
                                                .map(|x| x.get_surreal_record_id().clone())
                                                .collect::<Vec<_>>();
                                            LapCount::MaxOf(parents)
                                        }
                                    }
                                } else {
                                    LapCount::F32(0.0)
                                }
                            }
                            None => LapCount::F32(0.0),
                        }
                    }
                }
            }
        }
        SurrealStaging::Planned => LapCount::F32(0.0),
        SurrealStaging::ThinkingAbout => LapCount::F32(0.0),
        SurrealStaging::Released => LapCount::F32(0.0),
    }
}

/// The goal is to get the number of times something has been worked on since a certain time. There is
/// no regard for how long each session was, just that it was worked on.
pub(crate) fn get_worked_on_since(
    enter_time: DateTime<Utc>,
    time_spent_log: &[TimeSpent<'_>],
) -> f32 {
    time_spent_log
        .iter()
        .filter(|x| x.get_started_at() > &enter_time)
        .count() as f32
}

pub(crate) fn get_worked_on_since_elapsed(
    enter_time: DateTime<Utc>,
    time_spent_log: &[TimeSpent<'_>],
) -> TimeDelta {
    time_spent_log
        .iter()
        .filter(|x| x.get_started_at() > &enter_time)
        .map(|x| x.get_time_delta())
        .sum()
}

/// You can be snoozed if you are covered or if you just haven't reached the starting on staging yet
fn calculate_is_snoozed(
    item_node: &ItemNode<'_>,
    all_nodes: &[ItemNode<'_>],
    now: &DateTime<Utc>,
) -> bool {
    if item_node.has_children(Filter::Active)
        || item_node.get_snoozed_until().iter().any(|x| x > &now)
    {
        true
    } else {
        let staging = item_node.get_staging();
        match staging {
            SurrealStaging::InRelationTo { .. } => false,
            SurrealStaging::NotSet => false,
            SurrealStaging::OnDeck { enter_list, .. }
            | SurrealStaging::MentallyResident { enter_list, .. } => match enter_list {
                EnterListReason::DateTime(enter_list) => {
                    let enter_list: DateTime<Utc> = enter_list.clone().into();
                    &enter_list > now
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after,
                } => {
                    if now < earliest {
                        true
                    } else if now > review_after {
                        false
                    } else {
                        let active_larger = item_node.get_larger(Filter::Active);
                        let active_larger = active_larger
                            .map(|x| x.get_node(all_nodes))
                            .collect::<Vec<_>>();
                        let mut active_larger_iter = active_larger.iter();
                        let (highest_uncovered, _) = loop {
                            let larger = active_larger_iter.next();
                            match larger {
                                Some(larger) => {
                                    let (highest_uncovered, uncovered_when) =
                                        find_highest_uncovered_child_with_when_uncovered(
                                            larger,
                                            now,
                                            Vec::default(),
                                        );
                                    if let Some(highest_uncovered) = highest_uncovered {
                                        break (Some(highest_uncovered), uncovered_when);
                                    }
                                }
                                None => break (None, None),
                            }
                        };
                        match highest_uncovered {
                            Some(highest_uncovered) => highest_uncovered != item_node.get_item(),
                            None => true,
                        }
                    }
                }
            },
            SurrealStaging::Planned => false,
            SurrealStaging::ThinkingAbout => false,
            SurrealStaging::Released => false,
        }
    }
}

fn find_highest_uncovered_child_with_when_uncovered<'a>(
    item_node: &'a ItemNode<'a>,
    now: &DateTime<Utc>,
    visited: Vec<&'a Item<'a>>,
) -> (Option<&'a Item<'a>>, Option<DateTime<Utc>>) {
    let now: Datetime = (*now).into();
    let mut when_uncovered = None;
    for child in item_node.get_smaller(Filter::All) {
        if child.has_smaller(Filter::All) {
            if visited.contains(&child.get_item()) {
                continue;
            } else {
                let mut visited = visited.clone();
                visited.push(child.get_item());
                let (highest_uncovered, when) =
                    find_highest_uncovered_child_with_when_uncovered(item_node, &now, visited);
                if highest_uncovered.is_some() {
                    return (highest_uncovered, when);
                } else {
                    if when > when_uncovered {
                        when_uncovered = when;
                    }
                    continue;
                }
            }
        } else if child.is_finished() {
            let when_child_finished = child.when_finished().expect("is_finished() is true");
            match when_uncovered {
                Some(uncovered) => {
                    if when_child_finished > uncovered {
                        // We want the latest finished child
                        when_uncovered = Some(when_child_finished)
                    }
                }
                None => when_uncovered = Some(when_child_finished),
            }
            continue;
        }
        let staging = child.get_staging();
        match staging {
            SurrealStaging::InRelationTo { .. } => todo!(),
            SurrealStaging::NotSet => continue,
            SurrealStaging::MentallyResident { enter_list, .. } => {
                match enter_list {
                    EnterListReason::DateTime(datetime) => {
                        if datetime < &now {
                            return (Some(child.get_item()), when_uncovered);
                        } else {
                            // Is currently covered
                            continue;
                        }
                    }
                    EnterListReason::HighestUncovered {
                        earliest,
                        review_after: _review_after,
                    } => {
                        if earliest > &now {
                            println!("Do I need to update latest_uncovered2? I think I can just remove this");
                            continue;
                        }
                        todo!();
                    }
                }
            }
            SurrealStaging::OnDeck { enter_list, .. } => match enter_list {
                EnterListReason::DateTime(datetime) => {
                    if datetime < &now {
                        return (Some(child.get_item()), when_uncovered);
                    } else {
                        // Is currently covered
                        continue;
                    }
                }
                EnterListReason::HighestUncovered {
                    earliest,
                    review_after: _review_after,
                } => {
                    if earliest > &now {
                        println!("Do I need to update latest_uncovered4?");
                        continue;
                    } else {
                        return (Some(child.get_item()), when_uncovered);
                    }
                }
            },
            SurrealStaging::Planned => continue,
            SurrealStaging::ThinkingAbout => continue,
            SurrealStaging::Released => continue,
        }
    }
    (None, when_uncovered)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Days, Utc};
    use surrealdb::sql::{Datetime, Duration};
    use tokio::sync::mpsc;

    use crate::{
        base_data::BaseData,
        calculated_data::CalculatedData,
        new_item::{NewItem, NewItemBuilder},
        node::item_status::LapCount,
        surrealdb_layer::{
            data_storage_start_and_run,
            surreal_item::{EnterListReason, SurrealLap, SurrealStaging},
            surreal_tables::SurrealTables,
            DataLayerCommands,
        },
    };

    #[tokio::test]
    async fn parent_node_1_child_configured_for_highest_uncovered_if_before_earliest_time_it_should_remain_snoozed(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("New Child Item")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(10))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        // Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let child = items_highest_lap_count
            .iter()
            .find(|x| x.get_summary() == "New Child Item")
            .unwrap();
        let child = child.get_item_status();

        // Assert
        assert_eq!(child.is_snoozed(), true);
        assert_eq!(child.get_lap_count(), &LapCount::F32(0.0));

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_1_child_configured_for_highest_uncovered_it_should_be_ready_immediately(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("New Child Item")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        // Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "New Child Item")
            .unwrap();
        let child = child.get_item_status();

        // Assert
        assert_eq!(child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = child.get_lap_count() {
            assert!(*lap_count > 0.0);
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_both_configured_for_highest_uncovered_higher_one_becomes_ready_and_lower_one_stays_covered(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Should Become Ready")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Stay Covered")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let higher_child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Higher Child That Should Become Ready")
            .unwrap();
        let higher_child = higher_child.get_item_status();

        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Lower Child That Should Stay Covered")
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(higher_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = higher_child.get_lap_count() {
            assert!(*lap_count > 0.0);
        } else {
            panic!("Expected LapCount::F32");
        }
        assert_eq!(lower_child.is_snoozed(), true);
        assert_eq!(lower_child.get_lap_count(), &LapCount::F32(0.0));

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_finished_lower_child_configured_for_highest_uncovered_it_should_be_ready_when_the_other_item_is_finished(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Was On Deck But Becomes Finished")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::DateTime(Datetime(DateTime::from(
                            Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing"),
                        ))),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Uncover When The Higher Child Is Finished")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(2))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let lower_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Higher Child That Was On Deck But Becomes Finished")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: lower_child.get_surreal_record_id().clone(),
                when_finished: now.into(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let a_day_later_so_a_lap_passes = now
            .checked_add_days(Days::new(1))
            .expect("Far from overflowing");
        let calculated_data =
            CalculatedData::new_from_base_data(base_data, &a_day_later_so_a_lap_passes);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Uncover When The Higher Child Is Finished"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_finished_a_long_time_ago_lower_child_configured_for_highest_uncovered_it_should_be_ready_with_a_lap_time_starting_when_it_was_filed(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Was On Deck But Becomes Finished")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::DateTime(Datetime(DateTime::from(
                            Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing"),
                        ))),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(Datetime(DateTime::from(
                        Utc::now()
                            .checked_sub_days(Days::new(2))
                            .expect("Far from overflowing"),
                    )))
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Uncover With A Starting Lap Time of When This Item Was Created")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(2))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let a_day_ago = now
            .checked_sub_days(Days::new(1))
            .expect("Far from overflowing");
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let lower_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Higher Child That Was On Deck But Becomes Finished")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: lower_child.get_surreal_record_id().clone(),
                when_finished: a_day_ago.into(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let a_day_later_so_a_lap_passes = now
            .checked_add_days(Days::new(1))
            .expect("Far from overflowing");
        let calculated_data =
            CalculatedData::new_from_base_data(base_data, &a_day_later_so_a_lap_passes);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Uncover With A Starting Lap Time of When This Item Was Created"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_covered_and_lower_child_configured_for_highest_uncovered_it_should_be_ready_when_highest_became_covered(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Is On Deck And Becomes Covered")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::DateTime(Datetime(DateTime::from(
                            Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing"),
                        ))),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Be Ready When The Higher Child Is Covered")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(2))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let new_item = NewItem::new(
            "Item that the higher child will be waiting on to finish first".into(),
            Utc::now(),
        );
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let wait_on = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Item that the higher child will be waiting on to finish first")
            .unwrap();
        let higher_child = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Higher Child That Is On Deck And Becomes Covered")
            .unwrap();

        sender
            .send(DataLayerCommands::CoverItemWithAnExistingItem {
                item_to_be_covered: higher_child.id.as_ref().unwrap().clone(),
                item_that_should_do_the_covering: wait_on.id.as_ref().unwrap().clone(),
            })
            .await
            .unwrap();

        let now = Utc::now();
        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let parent = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "New Parent Item")
            .unwrap();
        let parent = parent.get_item_status();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Be Ready When The Higher Child Is Covered"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(parent.is_snoozed(), true);
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_one_mentally_resident_lower_item_after_review_by_date_it_should_show_up_as_needing_review(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Should Become Ready")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Show Up As Needing Review")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(2))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let higher_child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Higher Child That Should Become Ready")
            .unwrap();
        let higher_child = higher_child.get_item_status();

        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary() == "Lower Child That Should Show Up As Needing Review"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(higher_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = higher_child.get_lap_count() {
            assert!(
                *lap_count > 0.0,
                "Lap count was {:?}",
                higher_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.0,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_one_unset_the_other_highest_mentally_resident_should_be_on_deck_immediately(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Higher Child That Should Be Unset")
                    .staging(SurrealStaging::NotSet)
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Be On Deck Immediately")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(1))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary() == "Lower Child That Should Be On Deck Immediately"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_on_deck_covered_by_datetime_lower_child_is_not_snoozed(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Highest Child That is Covered By DateTime")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::DateTime(
                            Utc::now()
                                .checked_add_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                        ),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Be Available")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Lower Child That Should Be Available")
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_2_children_highest_mentally_resident_covered_by_datetime_lower_child_is_not_snoozed(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Highest Child That is Covered By DateTime")
                    .staging(SurrealStaging::MentallyResident {
                        enter_list: EnterListReason::DateTime(
                            Utc::now()
                                .checked_add_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                        ),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Be Available")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Lower Child That Should Be Available")
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_3_children_two_finished_top_first_lap_count_starts_from_the_last_finished(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Highest Child That Finishes First")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Middle Child That Finishes Second")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Uncover When The Other Two Are Finished")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(3))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(3))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let highest_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Highest Child That Finishes First")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: highest_child.get_surreal_record_id().clone(),
                when_finished: now
                    .checked_sub_days(Days::new(2))
                    .expect("Far from overflowing")
                    .into(),
            })
            .await
            .unwrap();
        let middle_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Middle Child That Finishes Second")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: middle_child.get_surreal_record_id().clone(),
                when_finished: now
                    .checked_sub_days(Days::new(1))
                    .expect("Far from overflowing")
                    .into(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Uncover When The Other Two Are Finished"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn parent_node_with_3_children_two_finished_middle_first_lap_count_starts_from_the_last_finished(
    ) {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("New Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "New Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Highest Child That Finishes Second")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(1))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Middle Child That Finishes First")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Utc::now()
                                .checked_sub_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                            review_after: Utc::now()
                                .checked_add_days(Days::new(2))
                                .expect("Far from overflowing")
                                .into(),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Lower Child That Should Uncover When The Other Two Are Finished")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::HighestUncovered {
                            earliest: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_sub_days(Days::new(3))
                                    .expect("Far from overflowing"),
                            )),
                            review_after: Datetime(DateTime::from(
                                Utc::now()
                                    .checked_add_days(Days::new(3))
                                    .expect("Far from overflowing"),
                            )),
                        },
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let highest_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Highest Child That Finishes Second")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: highest_child.get_surreal_record_id().clone(),
                when_finished: now
                    .checked_sub_days(Days::new(1))
                    .expect("Far from overflowing")
                    .into(),
            })
            .await
            .unwrap();
        let middle_child = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Middle Child That Finishes First")
            .unwrap();
        sender
            .send(DataLayerCommands::FinishItem {
                item: middle_child.get_surreal_record_id().clone(),
                when_finished: now
                    .checked_sub_days(Days::new(2))
                    .expect("Far from overflowing")
                    .into(),
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let lower_child = items_highest_lap_count
            .iter()
            .find(|x| {
                x.get_item().get_summary()
                    == "Lower Child That Should Uncover When The Other Two Are Finished"
            })
            .unwrap();
        let lower_child = lower_child.get_item_status();

        //Assert
        assert_eq!(lower_child.is_snoozed(), false);
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count > 0.9,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }
        if let LapCount::F32(lap_count) = lower_child.get_lap_count() {
            assert!(
                *lap_count < 1.1,
                "Lap count was {:?}",
                lower_child.get_lap_count()
            );
        } else {
            panic!("Expected LapCount::F32");
        }

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }

    #[tokio::test]
    async fn child_with_parent_that_is_covered_until_tomorrow_is_snoozed() {
        // Arrange
        let (sender, receiver) = mpsc::channel(1);
        let data_storage_join_handle =
            tokio::spawn(async move { data_storage_start_and_run(receiver, "mem://").await });

        let new_item = NewItem::new("Parent Item".into(), Utc::now());
        sender
            .send(DataLayerCommands::NewItem(new_item))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let parent = surreal_tables
            .surreal_items
            .iter()
            .find(|x| x.summary == "Parent Item")
            .unwrap();

        sender
            .send(DataLayerCommands::ParentItemWithANewChildItem {
                child: NewItemBuilder::default()
                    .summary("Child Item That Should Be Snoozed")
                    .staging(SurrealStaging::OnDeck {
                        enter_list: EnterListReason::DateTime(Datetime(DateTime::from(
                            Utc::now()
                                .checked_sub_days(Days::new(1))
                                .expect("Far from overflowing"),
                        ))),
                        lap: SurrealLap::AlwaysTimer(Duration::from_days(1)),
                    })
                    .created(
                        Utc::now()
                            .checked_sub_days(Days::new(30))
                            .expect("Far from overflowing"),
                    )
                    .build()
                    .expect("valid new item"),
                parent: parent.id.as_ref().unwrap().clone(),
                higher_priority_than_this: None,
            })
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let now = Utc::now();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);
        let child_to_snooze = base_data
            .get_active_items()
            .iter()
            .find(|x| x.get_summary() == "Child Item That Should Be Snoozed")
            .unwrap();

        sender
            .send(DataLayerCommands::CoverItemUntilAnExactDateTime(
                child_to_snooze.get_surreal_record_id().clone(),
                Utc::now()
                    .checked_add_days(Days::new(1))
                    .expect("Far from overflowing"),
            ))
            .await
            .unwrap();

        let surreal_tables = SurrealTables::new(&sender).await.unwrap();
        let base_data = BaseData::new_from_surreal_tables(surreal_tables, now);

        //Act
        let calculated_data = CalculatedData::new_from_base_data(base_data, &now);
        let items_highest_lap_count = calculated_data.get_items_highest_lap_count();
        let child = items_highest_lap_count
            .iter()
            .find(|x| x.get_item().get_summary() == "Child Item That Should Be Snoozed")
            .unwrap();
        let child = child.get_item_status();

        //Assert
        assert_eq!(child.is_snoozed(), true);

        drop(sender);
        data_storage_join_handle.await.unwrap();
    }
}
