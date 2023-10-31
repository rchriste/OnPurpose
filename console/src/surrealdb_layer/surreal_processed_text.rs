use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use surrealdb_extra::table::Table;

#[derive(PartialEq, Eq, Table, Serialize, Deserialize, Clone, Debug)]
#[table(name = "processed_text")]
pub(crate) struct SurrealProcessedText {
    pub(crate) id: Option<Thing>,
    pub(crate) text: String,
    pub(crate) when_written: Datetime,
    pub(crate) for_item: RecordId,
}
