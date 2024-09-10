use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SurrealProcessedText {
    pub(crate) id: Option<Thing>,
    pub(crate) text: String,
    pub(crate) when_written: Datetime,
    pub(crate) for_item: RecordId,
}

impl SurrealProcessedText {
    pub(crate) const TABLE_NAME: &'static str = "processed_text";
}
