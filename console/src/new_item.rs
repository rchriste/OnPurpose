use surrealdb::sql::Datetime;

use crate::{base_data::ItemType, surrealdb_layer::surreal_item::Responsibility};

pub(crate) struct NewItem {
    pub(crate) summary: String,
    pub(crate) finished: Option<Datetime>,
    pub(crate) responsibility: Responsibility,
    pub(crate) item_type: ItemType,
}
