use surrealdb::sql::Datetime;

use crate::{base_data::ItemType, surrealdb_layer::surreal_item::Responsibility};

pub struct NewItem {
    pub summary: String,
    pub finished: Option<Datetime>,
    pub responsibility: Responsibility,
    pub item_type: ItemType,
}
