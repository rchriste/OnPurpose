use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Div, Mul, Sub},
};

use chrono::TimeDelta;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Duration, Thing},
};
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
    pub(crate) facing: Vec<Facing>,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: ItemType,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation,

    #[cfg_attr(test, builder(default))]
    pub(crate) permanence: Permanence,

    #[cfg_attr(test, builder(default))]
    pub(crate) lap: Option<SurrealLap>,

    #[cfg_attr(test, builder(default))]
    pub(crate) ready: SurrealReady,

    #[cfg_attr(test, builder(default))]
    pub(crate) importance_review: Option<SurrealImportanceReview>,

    #[cfg_attr(test, builder(default))]
    pub(crate) plan_review: Option<SurrealPlanReview>,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,

    #[cfg_attr(test, builder(default = "chrono::Utc::now().into()"))]
    pub(crate) created: Datetime,

    #[cfg_attr(test, builder(default))]
    pub(crate) scheduled: SurrealScheduled,

    #[cfg_attr(test, builder(default))]
    pub(crate) urgency_plan: SurrealUrgencyPlan,
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
            facing: new_item.facing,
            item_type: new_item.item_type,
            smaller_items_in_priority_order,
            notes_location: NotesLocation::default(),
            permanence: new_item.permanence,
            staging: new_item.staging,
            created: new_item.created.into(),
            scheduled: new_item.scheduled,
            urgency_plan: new_item.urgency_plan,
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum Facing {
    Others {
        how_well_defined: HowWellDefined,
        who: RecordId,
    },
    Myself(HowWellDefined),
    InternalOrSmaller,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum HowWellDefined {
    #[default]
    NotSet,
    WellDefined,
    RoughlyDefined,
    LooselyDefined,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum ItemType {
    #[default]
    Undeclared,
    Action,
    Goal(HowMuchIsInMyControl),
    IdeaOrThought,
    Motivation,
    PersonOrGroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum HowMuchIsInMyControl {
    #[default]
    NotSet,
    MostlyInMyControl,
    PartiallyInMyControl,
    LargelyOutOfMyControl,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum GoalType {
    #[default]
    NotSpecified,
    AspirationalHope,
    TangibleMilestone,
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
pub(crate) enum SurrealReady {
    #[default]
    Always,
    AfterDateTime(Datetime),
    DuringItem(RecordId),
    AfterItem(RecordId)
}

pub(crate) struct SurrealMentallyResident {
    pub(crate) last_worked_on: Datetime,
    pub(crate) save_state: SurrealSaveState,
    pub(crate) work_on_frequency: SurrealFrequency,
}

pub(crate) enum SurrealSaveState {
    Internal(String),
    //OneNote is where I hope to move this in the future or maybe inside the linked item
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealImportanceReview {
    pub(crate) last_reviewed: Datetime,
    pub(crate) when_ready: SurrealReady,
    pub(crate) review_frequency: SurrealFrequency,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealPlanReview {
    pub(crate) last_reviewed: Datetime,
    pub(crate) when_ready: SurrealReady,
    pub(crate) review_frequency: SurrealFrequency,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealFrequency {
    Scheduled{ range_start: Datetime, range_end: Datetime},
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum EnterListReasonOldVersion {
    DateTime(Datetime),
    HighestUncovered {
        earliest: Datetime,
        review_after: Datetime,
    },
}
//This is a newtype pattern for f32 that implements PartialEq and Eq
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct EqF32(f32);

impl Display for EqF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<f32> for EqF32 {
    fn from(value: f32) -> Self {
        EqF32(value)
    }
}

impl From<EqF32> for f32 {
    fn from(value: EqF32) -> Self {
        value.0
    }
}

impl PartialEq for EqF32 {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f32::EPSILON
    }
}

impl PartialEq<f32> for EqF32 {
    fn eq(&self, other: &f32) -> bool {
        (self.0 - *other).abs() < f32::EPSILON
    }
}

impl PartialOrd<f32> for EqF32 {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Eq for EqF32 {}

impl Div for EqF32 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        EqF32(self.0 / rhs.0)
    }
}

impl Div<f32> for EqF32 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        EqF32(self.0 / rhs)
    }
}

impl Div<&EqF32> for f32 {
    type Output = EqF32;

    fn div(self, rhs: &EqF32) -> Self::Output {
        EqF32(self / rhs.0)
    }
}

impl Mul<f32> for EqF32 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        EqF32(self.0 * rhs)
    }
}

impl Mul<&EqF32> for f32 {
    type Output = EqF32;

    fn mul(self, rhs: &EqF32) -> Self::Output {
        EqF32(self * rhs.0)
    }
}

impl Mul<EqF32> for f32 {
    type Output = EqF32;

    fn mul(self, rhs: EqF32) -> Self::Output {
        EqF32(self * rhs.0)
    }
}

impl Sub for EqF32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        EqF32(self.0 - rhs.0)
    }
}

impl Sub<f32> for EqF32 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        EqF32(self.0 - rhs)
    }
}

impl Sub<&EqF32> for f32 {
    type Output = EqF32;

    fn sub(self, rhs: &EqF32) -> Self::Output {
        EqF32(self - rhs.0)
    }
}

impl Sub<EqF32> for f32 {
    type Output = EqF32;

    fn sub(self, rhs: EqF32) -> Self::Output {
        EqF32(self - rhs.0)
    }
}

impl Mul<EqF32> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: EqF32) -> Self::Output {
        TimeDelta::seconds((self.num_seconds() as f32 * rhs.0) as i64)
    }
}

impl Mul<&EqF32> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: &EqF32) -> Self::Output {
        TimeDelta::seconds((self.num_seconds() as f32 * rhs.0) as i64)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum InRelationToRatioType {
    AmountOfTimeSpent { multiplier: EqF32 },
    IterationCount { multiplier: EqF32 },
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealStaging {
    #[default]
    NotSet,
    MentallyResident {
        enter_list: EnterListReason,
        lap: SurrealLap,
    },
    OnDeck {
        enter_list: EnterListReason,
        lap: SurrealLap,
    },
    Planned,
    ThinkingAbout,
    Released,
    InRelationTo {
        start: Datetime,
        other_item: RecordId,
        ratio: InRelationToRatioType,
    },
}

impl PartialOrd for SurrealStaging {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SurrealStaging {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            SurrealStaging::NotSet => match other {
                SurrealStaging::NotSet => Ordering::Equal,
                _ => Ordering::Less,
            },
            SurrealStaging::MentallyResident { .. } => match other {
                SurrealStaging::NotSet => Ordering::Greater,
                SurrealStaging::MentallyResident { .. } => Ordering::Equal,
                _ => Ordering::Less,
            },
            SurrealStaging::InRelationTo { .. } => match other {
                SurrealStaging::NotSet | SurrealStaging::MentallyResident { .. } => {
                    Ordering::Greater
                }
                SurrealStaging::InRelationTo { .. } => Ordering::Equal,
                _ => Ordering::Less,
            },
            SurrealStaging::OnDeck { .. } => match other {
                SurrealStaging::NotSet
                | SurrealStaging::MentallyResident { .. }
                | SurrealStaging::InRelationTo { .. } => Ordering::Greater,
                SurrealStaging::OnDeck { .. } => Ordering::Equal,
                _ => Ordering::Less,
            },
            SurrealStaging::Planned => match other {
                SurrealStaging::Released | SurrealStaging::ThinkingAbout => Ordering::Less,
                SurrealStaging::Planned => Ordering::Equal,
                _ => Ordering::Greater,
            },
            SurrealStaging::ThinkingAbout => match other {
                SurrealStaging::Released => Ordering::Less,
                SurrealStaging::ThinkingAbout => Ordering::Equal,
                _ => Ordering::Greater,
            },
            SurrealStaging::Released => match other {
                SurrealStaging::Released => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealLap {
    AlwaysTimer(Duration),
    ///The amount of time that should be logged or worked on before the lap count is 1.
    LoggedTimer(Duration),
    ///`stride` is the number of other items that should be worked on before the lap count is 1.
    ///In other words 1/stride * items_worked is the lap count.
    WorkedOnCounter {
        stride: u32,
    },
    InherentFromParent,
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealScheduled {
    #[default]
    NotScheduled,
    ScheduledExact {
        start: Datetime,
        duration: Duration,
        priority: SurrealScheduledPriority,
    },
    ScheduledRange {
        start_range: (Datetime, Datetime),
        duration: Duration,
        priority: SurrealScheduledPriority,
    },
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealScheduledPriority {
    Always,
    WhenRoutineIsActive,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealUrgencyPlan {
    WillEscalate{start: SurrealUrgency, trigger: Datetime, after_trigger: SurrealUrgency},
    Always(SurrealUrgency),
}

impl Default for SurrealUrgencyPlan {
    fn default() -> Self {
        SurrealUrgencyPlan::Always(SurrealUrgency::default())
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealUrgency {
    BeforeScheduledItems,
    BeforeRoutine,
    ElavatedInsideRoutine,
    #[default]
    Normal,
}


//derive Builder is only for tests, I tried adding it just for cfg_attr(test... but that
//gave me false errors in the editor (rust-analyzer) so I am just going to try including
//it always to see if that addresses these phantom errors. Nov2023.
#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(setter(into))]
#[table(name = "item")]
pub(crate) struct SurrealItemOldVersion {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,

    #[cfg_attr(test, builder(default))]
    pub(crate) finished: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) responsibility: Responsibility,

    #[cfg_attr(test, builder(default))]
    pub(crate) facing: Vec<Facing>,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: ItemType,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation,

    #[cfg_attr(test, builder(default))]
    pub(crate) permanence: Permanence,

    #[cfg_attr(test, builder(default))]
    pub(crate) staging: SurrealStaging,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,

    #[cfg_attr(test, builder(default = "chrono::Utc::now().into()"))]
    pub(crate) created: Datetime,
    //Touched and worked_on would be joined from separate tables so this does not need to be edited a lot for those purposes

    #[cfg_attr(test, builder(default))]
    pub(crate) scheduled: SurrealScheduled,
}

impl From<SurrealItemOldVersion> for SurrealItem {
    fn from(value: SurrealItemOldVersion) -> Self {
        SurrealItem {
            id: value.id,
            summary: value.summary,
            finished: value.finished,
            responsibility: value.responsibility,
            facing: value.facing,
            item_type: value.item_type,
            notes_location: value.notes_location,
            permanence: value.permanence,
            staging: value.staging,
            smaller_items_in_priority_order: value.smaller_items_in_priority_order,
            created: value.created,
            scheduled: value.scheduled,
            urgency_plan: SurrealUrgencyPlan::default(),
        }
    }
}
