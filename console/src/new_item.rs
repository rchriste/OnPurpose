use chrono::{DateTime, Utc};
use derive_builder::Builder;
use surrealdb::sql::Datetime;

use crate::data_storage::surrealdb_layer::surreal_item::{
    Responsibility, SurrealDependency, SurrealFacing, SurrealFrequency, SurrealItemType,
    SurrealLap, SurrealReviewGuidance, SurrealUrgencyPlan,
};

#[derive(Builder)]
#[builder(setter(into))]
pub(crate) struct NewItem {
    pub(crate) summary: String,

    #[builder(default)]
    pub(crate) finished: Option<Datetime>,

    #[builder(default)]
    pub(crate) responsibility: Responsibility,

    #[builder(default)]
    pub(crate) facing: Vec<SurrealFacing>,

    #[builder(default)]
    pub(crate) item_type: SurrealItemType,

    #[builder(default = "Utc::now()")]
    pub(crate) created: DateTime<Utc>,

    #[builder(default)]
    pub(crate) urgency_plan: Option<SurrealUrgencyPlan>,

    #[builder(default)]
    pub(crate) lap: Option<SurrealLap>,

    #[builder(default)]
    pub(crate) dependencies: Vec<SurrealDependency>,

    #[builder(default)]
    pub(crate) last_reviewed: Option<DateTime<Utc>>,

    #[builder(default)]
    pub(crate) review_frequency: Option<SurrealFrequency>,

    #[builder(default)]
    pub(crate) review_guidance: Option<SurrealReviewGuidance>,
}

impl NewItem {
    pub(crate) fn new(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            facing: Default::default(),
            item_type: SurrealItemType::Undeclared,
            created: now,
            urgency_plan: None,
            lap: None,
            dependencies: Default::default(),
            last_reviewed: None,
            review_frequency: None,
            review_guidance: None,
        }
    }

    pub(crate) fn new_person_or_group(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            facing: Default::default(),
            item_type: SurrealItemType::PersonOrGroup,
            created: now,
            urgency_plan: None,
            lap: None,
            dependencies: Default::default(),
            last_reviewed: None,
            review_frequency: None,
            review_guidance: None,
        }
    }
}
