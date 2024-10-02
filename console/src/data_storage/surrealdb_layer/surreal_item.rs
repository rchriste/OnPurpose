use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Div, Mul, Sub},
};

use chrono::{DateTime, TimeDelta, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Duration, Thing},
};

use crate::{base_data::item::Item, new_item::NewItem};

use super::SurrealTrigger;

//derive Builder is only for tests, I tried adding it just for cfg_attr(test... but that
//gave me false errors in the editor (rust-analyzer) so I am just going to try including
//it always to see if that addresses these phantom errors. Nov2023.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(setter(into))]
pub(crate) struct SurrealItem {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,

    #[cfg_attr(test, builder(default = "1"))]
    pub(crate) version: u32,

    #[cfg_attr(test, builder(default))]
    pub(crate) finished: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) responsibility: Responsibility,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: SurrealItemType,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation,

    #[cfg_attr(test, builder(default))]
    pub(crate) lap: Option<SurrealLap>,

    #[cfg_attr(test, builder(default))]
    pub(crate) dependencies: Vec<SurrealDependency>,

    #[cfg_attr(test, builder(default))]
    pub(crate) last_reviewed: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) review_frequency: Option<SurrealFrequency>,

    #[cfg_attr(test, builder(default))]
    pub(crate) review_guidance: Option<SurrealReviewGuidance>,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,

    #[cfg_attr(test, builder(default = "chrono::Utc::now().into()"))]
    pub(crate) created: Datetime,

    #[cfg_attr(test, builder(default))]
    pub(crate) urgency_plan: Option<SurrealUrgencyPlan>,
}

impl From<SurrealItem> for Option<Thing> {
    fn from(value: SurrealItem) -> Self {
        value.id
    }
}

impl SurrealItem {
    pub(crate) const TABLE_NAME: &'static str = "item";

    pub(crate) fn new(
        new_item: NewItem,
        smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,
    ) -> Self {
        let last_reviewed = new_item.last_reviewed.map(|dt| dt.into());
        SurrealItem {
            id: None,
            version: 1,
            summary: new_item.summary,
            finished: new_item.finished,
            responsibility: new_item.responsibility,
            item_type: new_item.item_type,
            smaller_items_in_priority_order,
            notes_location: NotesLocation::default(),
            created: new_item.created.into(),
            urgency_plan: new_item.urgency_plan,
            lap: new_item.lap,
            dependencies: new_item.dependencies,
            last_reviewed,
            review_frequency: new_item.review_frequency,
            review_guidance: new_item.review_guidance,
        }
    }

    pub(crate) fn make_item<'a>(&'a self, now: &'a DateTime<Utc>) -> Item<'a> {
        Item::new(self, now)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealItemType {
    #[default]
    Undeclared,
    Action,
    Goal(SurrealHowMuchIsInMyControl),
    IdeaOrThought,
    /// Purpose behind the work
    Motivation(SurrealMotivationKind),
    PersonOrGroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealItemTypeOld {
    #[default]
    Undeclared,
    Action,
    Goal(SurrealHowMuchIsInMyControl),
    IdeaOrThought,
    Motivation,
    PersonOrGroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealMotivationKind {
    #[default]
    NotSet,
    CoreWork,
    NonCoreWork,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum ItemTypeOld {
    #[default]
    Undeclared,
    Action,
    Goal(SurrealHowMuchIsInMyControl),
    IdeaOrThought,
    Motivation,
    PersonOrGroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealHowMuchIsInMyControl {
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealDependency {
    AfterDateTime(Datetime),
    DuringItem(RecordId), //TODO: This should be removed as it is no longer used
    AfterItem(RecordId),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealItemReview {
    pub(crate) last_reviewed: Option<Datetime>,
    pub(crate) review_frequency: SurrealFrequency,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealReviewGuidance {
    AlwaysReviewChildrenWithThisItem,
    ReviewChildrenSeparately,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealFrequency {
    NoneReviewWithParent,
    Range {
        range_min: Duration,
        range_max: Duration,
    },
    Hourly,
    Daily,
    EveryFewDays,
    Weekly,
    BiMonthly,
    Monthly,
    Quarterly,
    SemiAnnually,
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
        enter_list: EnterListReasonOldVersion,
        lap: SurrealLap,
    },
    OnDeck {
        enter_list: EnterListReasonOldVersion,
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
    SubItem { surreal_item_id: Thing },
    //This could be expanded to state multiple items that are at the same priority meaning you would go with lap count or something else to determine which to work on first.
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum NotesLocation {
    #[default]
    None,
    OneNoteLink(String),
    WebLink(String),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealScheduled {
    Exact {
        start: Datetime,
        duration: Duration,
    },
    Range {
        start_range: (Datetime, Datetime),
        duration: Duration,
    },
}

impl PartialOrd for SurrealScheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SurrealScheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            SurrealScheduled::Exact { start, .. } => match other {
                SurrealScheduled::Exact {
                    start: other_start, ..
                } => start.cmp(other_start),
                SurrealScheduled::Range { start_range, .. } => start.cmp(&start_range.0),
            },
            SurrealScheduled::Range { start_range, .. } => match other {
                SurrealScheduled::Exact { start, .. } => start_range.0.cmp(start),
                SurrealScheduled::Range {
                    start_range: other_start_range,
                    ..
                } => start_range.0.cmp(&other_start_range.0),
            },
        }
    }
}

impl SurrealScheduled {
    pub(crate) fn get_earliest_start(&self) -> &Datetime {
        match self {
            SurrealScheduled::Exact { start, .. } => start,
            SurrealScheduled::Range { start_range, .. } => &(start_range.0),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealScheduledPriority {
    Always,
    WhenRoutineIsActive,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SurrealUrgencyPlan {
    //If any of the triggers, trigger then the urgency will escalate to the later urgency
    WillEscalate {
        initial: SurrealUrgency,
        triggers: Vec<SurrealTrigger>,
        later: SurrealUrgency,
    },
    StaysTheSame(SurrealUrgency),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, PartialOrd, Ord)]
pub(crate) enum SurrealUrgency {
    MoreUrgentThanAnythingIncludingScheduled,
    ScheduledAnyMode(SurrealScheduled),
    MoreUrgentThanMode,
    InTheModeScheduled(SurrealScheduled),
    InTheModeDefinitelyUrgent,
    InTheModeMaybeUrgent, //This is one of the things that map to PriorityLevel::RoutineReview
    InTheModeByImportance,
}

//derive Builder is only for tests, I tried adding it just for cfg_attr(test... but that
//gave me false errors in the editor (rust-analyzer) so I am just going to try including
//it always to see if that addresses these phantom errors. Nov2023.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(setter(into))]
pub(crate) struct SurrealItemOldVersion {
    pub(crate) id: Option<Thing>,
    pub(crate) summary: String,

    #[cfg_attr(test, builder(default))]
    pub(crate) finished: Option<Datetime>,

    #[cfg_attr(test, builder(default))]
    pub(crate) responsibility: Responsibility,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_type: SurrealItemType,

    #[cfg_attr(test, builder(default))]
    pub(crate) notes_location: NotesLocation, //TODO: I believe this is unused. Also in general I think a better design is to support this inside the general concept of capturing things so rather than having this hardcoded this should go away and be replaced by a capturing concept

    #[cfg_attr(test, builder(default))]
    pub(crate) lap: Option<SurrealLap>,

    #[cfg_attr(test, builder(default))]
    pub(crate) dependencies: Vec<SurrealDependency>,

    #[cfg_attr(test, builder(default))]
    pub(crate) item_review: Option<SurrealItemReview>,

    pub(crate) review_guidance: Option<SurrealReviewGuidance>,

    /// This is meant to be a list of the smaller or subitems of this item that further this item in an ordered list meaning that they should be done in order
    #[cfg_attr(test, builder(default))]
    pub(crate) smaller_items_in_priority_order: Vec<SurrealOrderedSubItem>,

    #[cfg_attr(test, builder(default = "chrono::Utc::now().into()"))]
    pub(crate) created: Datetime,

    #[cfg_attr(test, builder(default))]
    pub(crate) urgency_plan: Option<SurrealUrgencyPlan>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum SurrealScheduledOldVersion {
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

impl From<SurrealItemOldVersion> for SurrealItem {
    fn from(value: SurrealItemOldVersion) -> Self {
        let (last_reviewed, review_frequency) = match value.item_review {
            Some(item_review) => (
                item_review.last_reviewed,
                Some(item_review.review_frequency),
            ),
            None => (None, None),
        };

        SurrealItem {
            id: value.id,
            version: 1,
            summary: value.summary,
            finished: value.finished,
            responsibility: value.responsibility,
            item_type: value.item_type,
            notes_location: value.notes_location,
            lap: value.lap,
            smaller_items_in_priority_order: value.smaller_items_in_priority_order,
            created: value.created,
            urgency_plan: value.urgency_plan,
            dependencies: value.dependencies,
            review_guidance: value.review_guidance,
            last_reviewed,
            review_frequency,
        }
    }
}

impl SurrealItemOldVersion {
    pub(crate) const TABLE_NAME: &'static str = "item";
}
