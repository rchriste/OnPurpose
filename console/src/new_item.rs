use chrono::{DateTime, Utc};
use derive_builder::Builder;
use surrealdb::sql::Datetime;

use crate::surrealdb_layer::surreal_item::{
    Facing, ItemType, Permanence, Responsibility, SurrealScheduled, SurrealStaging, SurrealUrgencyPlan,
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
    pub(crate) facing: Vec<Facing>,

    #[builder(default)]
    pub(crate) item_type: ItemType,

    #[builder(default)]
    pub(crate) permanence: Permanence,

    #[builder(default)]
    pub(crate) staging: SurrealStaging,

    #[builder(default = "Utc::now()")]
    pub(crate) created: DateTime<Utc>,

    #[builder(default)]
    pub(crate) scheduled: SurrealScheduled,

    #[builder(default)]
    pub(crate) urgency_plan: SurrealUrgencyPlan,
}

impl NewItem {
    pub(crate) fn new(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            facing: Default::default(),
            item_type: ItemType::Undeclared,
            permanence: Permanence::default(),
            staging: SurrealStaging::default(),
            created: now,
            scheduled: SurrealScheduled::default(),
            urgency_plan: SurrealUrgencyPlan::default(),
        }
    }

    pub(crate) fn new_person_or_group(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            facing: Default::default(),
            item_type: ItemType::PersonOrGroup,
            permanence: Permanence::default(),
            staging: SurrealStaging::default(),
            created: now,
            scheduled: SurrealScheduled::default(),
            urgency_plan: SurrealUrgencyPlan::default(),
        }
    }
}
