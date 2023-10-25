use surrealdb::sql::Datetime;

use crate::{base_data::ItemType, surrealdb_layer::surreal_item::Responsibility};

pub(crate) struct NewItem {
    pub(crate) summary: String,
    pub(crate) finished: Option<Datetime>,
    pub(crate) responsibility: Responsibility,
    pub(crate) item_type: ItemType,
}

impl NewItem {
    pub(crate) fn new_person_or_group(summary: String) -> Self {
        NewItem {
            summary,
            finished: None,
            responsibility: Responsibility::ReactiveBeAvailableToAct,
            item_type: ItemType::PersonOrGroup,
        }
    }
}
