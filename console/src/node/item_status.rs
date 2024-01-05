use std::{ops::Sub, time::Duration};

use chrono::{DateTime, Utc};
use surrealdb::{opt::RecordId, sql::Thing};

use crate::{
    base_data::item::Item,
    surrealdb_layer::surreal_item::{EnterListReason, ItemType, Staging},
};

use super::item_node::{GrowingItemNode, ItemNode, ShrinkingItemNode};

#[derive(Clone, Debug)]
pub(crate) struct ItemStatus<'s> {
    item_node: ItemNode<'s>,
    lap_count: f32,
    is_snoozed: bool,
}

impl<'s> ItemStatus<'s> {
    pub(crate) fn new(
        item_node: ItemNode<'s>,
        all_nodes: &[ItemNode<'_>],
        current_date_time: &DateTime<Utc>,
    ) -> Self {
        let lap_count = calculate_lap_count(&item_node, current_date_time);
        let is_snoozed = calculate_is_snoozed(&item_node, current_date_time);
        Self {
            item_node,
            lap_count,
            is_snoozed,
        }
    }

    pub(crate) fn get_lap_count(&self) -> f32 {
        self.lap_count
    }

    pub(crate) fn is_snoozed(&self) -> bool {
        self.is_snoozed
    }

    pub(crate) fn is_first_lap_finished(&self) -> bool {
        self.get_lap_count() > 1.0
    }

    pub(crate) fn get_item_node(&'s self) -> &'s ItemNode<'s> {
        &self.item_node
    }

    pub(crate) fn get_staging(&self) -> &Staging {
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

    pub(crate) fn is_staging_mentally_resident(&self) -> bool {
        self.item_node.is_staging_mentally_resident()
    }

    pub(crate) fn is_person_or_group(&self) -> bool {
        self.item_node.is_person_or_group()
    }

    pub(crate) fn get_item(&self) -> &Item<'s> {
        self.item_node.get_item()
    }

    pub(crate) fn has_active_children(&self) -> bool {
        self.item_node.has_active_children()
    }

    pub(crate) fn get_smaller(&'s self) -> &[ShrinkingItemNode<'s>] {
        self.item_node.get_smaller()
    }

    pub(crate) fn get_type(&self) -> &ItemType {
        self.item_node.get_type()
    }

    pub(crate) fn get_surreal_record_id(&self) -> &RecordId {
        self.item_node.get_surreal_record_id()
    }

    pub(crate) fn get_larger(&'s self) -> &[GrowingItemNode<'s>] {
        self.item_node.get_larger()
    }
}

fn calculate_lap_count(item_node: &ItemNode<'_>, current_date_time: &DateTime<Utc>) -> f32 {
    match item_node.get_staging() {
        Staging::NotSet => 0.0,
        Staging::OnDeck { enter_list, lap } | Staging::MentallyResident { enter_list, lap } => {
            match enter_list {
                EnterListReason::DateTime(enter_time) => {
                    let enter_time: DateTime<Utc> = enter_time.clone().into();
                    let lap: Duration = lap.clone().into();
                    let elapsed = current_date_time.sub(enter_time);
                    let elapsed = elapsed.num_seconds() as f32;
                    let lap = lap.as_secs_f32();
                    elapsed / lap
                }
                EnterListReason::HighestUncovered { review_after } => {
                    todo!("review_after:{:?}", review_after)
                }
            }
        }
        Staging::Planned => 0.0,
        Staging::ThinkingAbout => 0.0,
        Staging::Released => 0.0,
    }
}

/// You can be snoozed if you are covered or if you just haven't reached the starting on staging yet
fn calculate_is_snoozed(item_node: &ItemNode<'_>, now: &DateTime<Utc>) -> bool {
    let staging = item_node.get_staging();
    let snoozed_from_staging = match staging {
        Staging::NotSet => false,
        Staging::OnDeck { enter_list, .. } | Staging::MentallyResident { enter_list, .. } => {
            match enter_list {
                EnterListReason::DateTime(enter_list) => {
                    let enter_list: DateTime<Utc> = enter_list.clone().into();
                    &enter_list > now
                }
                EnterListReason::HighestUncovered { .. } => todo!(),
            }
        }
        Staging::Planned => false,
        Staging::ThinkingAbout => false,
        Staging::Released => false,
    };

    item_node.get_snoozed_until().iter().any(|x| x > &&now) || snoozed_from_staging
}
