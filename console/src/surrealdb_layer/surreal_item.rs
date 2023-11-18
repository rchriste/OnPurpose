use std::cmp::Ordering;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb_extra::table::Table;

use crate::{base_data::item::Item, new_item::NewItem};

use super::surreal_required_circumstance::SurrealRequiredCircumstance;

//derive Builder is only for tests, I tried adding it just for cfg_attr(test... but that
//gave me false errors in the editor (rust-analyzer) so I am just going to try including
//it always to see if that addresses these phantom errors. Nov2023.
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(setter(into))]
#[table(name = "item")] //TODO: This should be renamed items
pub(crate) struct SurrealItem {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,

    #[cfg_attr(test, builder(default))]
    pub(crate) finished: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) responsibility: Responsibility,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: ItemType,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation,

    #[cfg_attr(test, builder(default))]
    pub(crate) permanence: Permanence,

    #[cfg_attr(test, builder(default))]
    pub(crate) staging: Staging,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,
}

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

impl SurrealItem {
    pub(crate) fn new(
        new_item: NewItem,
        smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,
    ) -> Self {
        SurrealItem {
            id: None,
            summary: new_item.summary,
            finished: new_item.finished,
            responsibility: new_item.responsibility,
            item_type: new_item.item_type,
            smaller_items_in_priority_order,
            notes_location: NotesLocation::default(),
            permanence: new_item.permanence,
            staging: new_item.staging,
        }
    }

    pub(crate) fn make_item<'a>(
        &'a self,
        requirements: &'a [SurrealRequiredCircumstance],
    ) -> Item<'a> {
        let my_requirements = requirements
            .iter()
            .filter(|x| {
                &x.required_for
                    == self
                        .id
                        .as_ref()
                        .expect("Item should already be in the database and have an id")
            })
            .collect();

        Item::new(self, my_requirements)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum ItemType {
    #[default]
    Undeclared,
    Simple,
    Action,
    Goal(GoalType),
    Motivation,
    PersonOrGroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum GoalType {
    #[default]
    NotSpecified,
    AspirationalHope,
    TangibleMilestone,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum ItemTypeOldVersion {
    #[default]
    Undeclared,
    Simple,
    ToDo, //TODO: Rename to Action
    Hope, //TODO: Rename to Goal (Hope, Milestone, or NotSpecified)
    Motivation,
    PersonOrGroup,
}

impl From<ItemTypeOldVersion> for ItemType {
    fn from(value: ItemTypeOldVersion) -> Self {
        match value {
            ItemTypeOldVersion::Undeclared => ItemType::Undeclared,
            ItemTypeOldVersion::Simple => ItemType::Simple,
            ItemTypeOldVersion::ToDo => ItemType::Action,
            ItemTypeOldVersion::Hope => ItemType::Goal(GoalType::NotSpecified),
            ItemTypeOldVersion::Motivation => ItemType::Motivation,
            ItemTypeOldVersion::PersonOrGroup => ItemType::PersonOrGroup,
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum Responsibility {
    #[default]
    ProactiveActionToTake,
    ReactiveBeAvailableToAct,
    WaitingFor, //TODO: This should not exist it should just be a TrackingToBeAwareOf that could be a Question or have some kind of automated way to track and watch and know
    TrackingToBeAwareOf,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum Permanence {
    Maintenance,
    Project,
    #[default]
    NotSet,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum Staging {
    #[default]
    NotSet,
    MentallyResident {
        last_worked_on: Datetime,
        work_on_again_before: Datetime,
    },
    OnDeck {
        began_waiting: Datetime,
        can_wait_until: Datetime,
    },
    Intension,
    Released,
}

impl PartialOrd for Staging {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Staging {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            Staging::NotSet => match other {
                Staging::NotSet => Ordering::Equal,
                _ => Ordering::Less,
            },
            Staging::MentallyResident { .. } => match other {
                Staging::NotSet => Ordering::Greater,
                Staging::MentallyResident { .. } => Ordering::Equal,
                _ => Ordering::Less,
            },
            Staging::OnDeck { .. } => match other {
                Staging::NotSet | Staging::MentallyResident { .. } => Ordering::Greater,
                Staging::OnDeck { .. } => Ordering::Equal,
                _ => Ordering::Less,
            },
            Staging::Intension => match other {
                Staging::Released => Ordering::Less,
                Staging::Intension => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Staging::Released => match other {
                Staging::Released => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealOrderedSubItem {
    SubItem {
        surreal_item_id: Thing,
    },
    Split {
        shared_priority: Vec<SurrealPriorityGoal>,
    },
}

//Each of these variants should be containing data but I don't want the data layer to get too far ahead of the prototype UI
//so I want to wait until I can try it out before working out these details so just this for now.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealPriorityGoal {
    AbsoluteInvocationCount,
    AbsoluteAmountOfTime,
    RelativePercentageOfTime,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum NotesLocation {
    #[default]
    None,
    OneNoteLink(String),
    WebLink(String),
}

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(Builder), builder(setter(into)))]
#[table(name = "item")] //TODO: This should be renamed items
pub(crate) struct SurrealItemOldVersion {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,

    #[cfg_attr(test, builder(default))]
    pub(crate) finished: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) responsibility: Responsibility,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: ItemTypeOldVersion,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation,

    #[cfg_attr(test, builder(default))]
    pub(crate) permanence: Permanence,

    #[cfg_attr(test, builder(default))]
    pub(crate) staging: Staging,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,
}

impl From<SurrealItemOldVersion> for SurrealItem {
    fn from(value: SurrealItemOldVersion) -> Self {
        SurrealItem {
            id: value.id,
            summary: value.summary,
            finished: value.finished,
            responsibility: value.responsibility,
            item_type: value.item_type.into(),
            notes_location: value.notes_location,
            permanence: value.permanence,
            staging: value.staging,
            smaller_items_in_priority_order: value.smaller_items_in_priority_order,
        }
    }
}
