use derive_builder::Builder;
use surrealdb::sql::Datetime;

use crate::surrealdb_layer::surreal_item::{ItemType, Permanence, Responsibility, Staging};

#[derive(Builder)]
#[builder(setter(into))]
pub(crate) struct NewItem {
    pub(crate) summary: String,

    #[builder(default)]
    pub(crate) finished: Option<Datetime>,

    #[builder(default)]
    pub(crate) responsibility: Responsibility,

    #[builder(default)]
    pub(crate) item_type: ItemType,

    #[builder(default)]
    pub(crate) permanence: Permanence,

    #[builder(default)]
    pub(crate) staging: Staging,
}

impl NewItem {
    pub(crate) fn new(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::default(),
            item_type: ItemType::Undeclared,
            permanence: Permanence::default(),
            staging: Staging::default(),
        }
    }

    pub(crate) fn new_person_or_group(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            item_type: ItemType::PersonOrGroup,
            permanence: Permanence::default(),
            staging: Staging::default(),
        }
    }
}
