use chrono::{DateTime, Utc};
use derive_builder::Builder;
use surrealdb::sql::Datetime;

use crate::surrealdb_layer::surreal_item::{Facing, ItemType, Permanence, Responsibility, Staging};

#[derive(Builder)]
#[builder(setter(into))]
pub(crate) struct NewItem {
    pub(crate) summary: String,

    #[builder(default)]
    pub(crate) finished: Option<Datetime>,

    #[builder(default)]
    pub(crate) responsibility: Responsibility,

    #[builder(default)]
    pub(crate) facing: Facing,

    #[builder(default)]
    pub(crate) item_type: ItemType,

    #[builder(default)]
    pub(crate) permanence: Permanence,

    #[builder(default)]
    pub(crate) staging: Staging,

    #[builder(default = "Utc::now()")]
    pub(crate) created: DateTime<Utc>,
}

impl NewItem {
    pub(crate) fn new(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            facing: Facing::default(),
            item_type: ItemType::Undeclared,
            permanence: Permanence::default(),
            staging: Staging::default(),
            created: now,
        }
    }

    pub(crate) fn new_person_or_group(summary: String, now: DateTime<Utc>) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            facing: Facing::default(),
            item_type: ItemType::PersonOrGroup,
            permanence: Permanence::default(),
            staging: Staging::default(),
            created: now,
        }
    }
}
